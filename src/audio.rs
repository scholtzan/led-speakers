use pulse::callbacks::ListResult;
use pulse::context::introspect::SinkInfo;
use pulse::context::Context;
use pulse::error::PAErr;
#[allow(unused_imports)]
use pulse::mainloop::api::Mainloop as MainloopTrait;
use pulse::mainloop::threaded::Mainloop;
use pulse::proplist::Proplist;
use pulse::sample::{Format, Spec};
use pulse::stream::flags;
use pulse::stream::{PeekResult, Stream};
use std::borrow::Cow;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time;

use crate::buffer::Buffer;


enum ReadyState {
    Stream(pulse::stream::State),
    Context(pulse::context::State),
}


struct AudioContext {
    context: Rc<RefCell<Context>>,
    mainloop: Rc<RefCell<Mainloop>>,
}


#[derive(Clone, Debug)]
pub struct AudioSink {
    name: String,
    index: u32,
    source_name: String,
    spec: Option<Spec>
}

impl AudioSink {
    fn from_pa_sink_info(sink_info: &SinkInfo) -> AudioSink {
        let name = match &sink_info.name {
            None => String::from("Unnamed audio source"),
            Some(Cow::Borrowed(inner_name)) => String::from(*inner_name),
            Some(Cow::Owned(inner_name)) => inner_name.clone(),
        };

        let source_name = match &sink_info.monitor_source_name {
            None => String::from("Unnamed audio source"),
            Some(Cow::Borrowed(inner_name)) => String::from(*inner_name),
            Some(Cow::Owned(inner_name)) => inner_name.clone(),
        };

        AudioSink {
            name: name,
            index: sink_info.index,
            source_name: source_name,
            spec: Some(sink_info.sample_spec.clone()),
        }
    }

    fn new(name: String, index: u32, source_name: String, spec: Option<Spec>) -> AudioSink {
        AudioSink {
            name,
            index,
            source_name,
            spec,
        }
    }
}

impl Default for AudioSink {
    fn default() -> Self {
        AudioSink {
            name: String::from("Audio Source"),
            index: 0,
            source_name: String::from("Audio Source"),
            spec: None,
        }
    }
}


pub struct AudioStream {
    name: String,
    source_name: String,
    handle: Option<JoinHandle<()>>,
    killed: Arc<AtomicBool>,
    buffer: Option<Arc<Buffer>>
}


impl AudioStream {
    pub fn new(name: String, source_name: String) -> AudioStream {
        let mut audio_stream = AudioStream {
            name,
            source_name,
            killed: Arc::new(AtomicBool::from(false)),
            handle: None,
            buffer: None
        };

        audio_stream.buffer = Some(audio_stream.start().expect("Could not create audio stream"));

        audio_stream
    }

    fn init(name: String) -> (Rc<RefCell<Mainloop>>, Rc<RefCell<Context>>) {
        let mut proplist = Proplist::new().unwrap();
        proplist.set(pulse::proplist::properties::APPLICATION_NAME, &name.as_bytes()).unwrap();

        let mainloop = Rc::new(RefCell::new(Mainloop::new().expect("Failed to create mainloop")));
        let context = Rc::new(RefCell::new(
            Context::new_with_proplist(mainloop.borrow().deref(), &name, &proplist)
                .expect("Failed to create context"),
        ));

        {
            let ml_ref = Rc::clone(&mainloop);
            let context_ref = Rc::clone(&context);
            context.borrow_mut().set_state_callback(Some(Box::new(move || {
                let state = unsafe { (*context_ref.as_ptr()).get_state() };
                match state {
                    pulse::context::State::Ready => unsafe {
                        (*ml_ref.as_ptr()).signal(false);
                    },
                    pulse::context::State::Failed | pulse::context::State::Terminated => unsafe {
                        eprintln!("Failed to connect to PulseAudio");
                        (*ml_ref.as_ptr()).signal(false);
                    },
                    _ => {}
                }
            })));
        }

        (mainloop, context)
    }

    fn start(&mut self) -> Result<Arc<Buffer>, String> {
        let weak_killed: Weak<AtomicBool> = Arc::downgrade(&self.killed);
        let buffer = Arc::new(Buffer::new(30000)); // todo: configurable and make member
        let buf = buffer.clone();
        let name = self.name.clone();
        let source_name = self.source_name.clone();

        self.handle = Some(thread::spawn(move || {
            let (mainloop, context) = Self::init(name);
            let sink = Self::connect_sink(&mainloop, &context, source_name);
            let mut stream = Self::connect_stream(&mainloop, &context, sink).expect("Error creating stream");
            let mut pa_stream = stream.lock().unwrap();
            mainloop.borrow_mut().lock();
            pa_stream.uncork(None);
            mainloop.borrow_mut().unlock();

            loop {
                let killed = weak_killed.upgrade();
                if killed.is_some() && !killed.unwrap().load(Ordering::Relaxed) {
                    mainloop.borrow_mut().lock();
                    let available = pa_stream.readable_size();

                    if let Some(count) = available {
                        if count < 128 { // todo: make configurable
                            thread::sleep(time::Duration::from_micros(200));
                            continue;
                        }
                    }

                    let mut written = 0;
                    let peek = pa_stream.peek().expect("Could not peek PulseAudio stream");
                    match peek {
                        PeekResult::Empty => {
                            mainloop.borrow_mut().unlock();
                            thread::sleep(time::Duration::from_micros(200));
                            continue;
                        },
                        PeekResult::Hole(size) => {
                            pa_stream.discard().unwrap();
                            mainloop.borrow_mut().unlock();
                        },
                        PeekResult::Data(data) => {
                            let read = data.len();
                            let mut s: i32 = 100; // todo: make configurable
                            while s > 0 {
                                let buffer_avail = buf.reserve(data.len());
                                if buffer_avail > data.len() {
                                    buf.write(data);
                                    written = data.len();
                                }
                                s -= 1;
                                if written >= read {
                                    break;
                                }
                            }

                            pa_stream.discard().expect("Could not discard PulseAudio stream");
                            mainloop.borrow_mut().unlock();
                        }
                    }
                } else {
                    let disconnect = Self::disconnect_stream(&mainloop, &stream);
                    match disconnect {
                        Ok(_) => {},
                        Err(error) => eprintln!("Could not disconnect stream"),
                    }
                    mainloop.borrow_mut().stop();
                    break;
                }
            }
        }));

        Ok(buffer.clone())
    }

    fn connect_sink(mainloop: &Rc<RefCell<Mainloop>>, context: &Rc<RefCell<Context>>, source_name: String) -> Rc<RefCell<Option<AudioSink>>> {
        context
            .borrow_mut()
            .connect(None, pulse::context::flags::NOFLAGS, None)
            .expect("Failed to connect context");
        mainloop.borrow_mut().lock();
        mainloop.borrow_mut().start().expect("Failed to start mainloop");

        let mut sink = Rc::new(RefCell::new(None));
        
        let state_closure = || ReadyState::Context(context.borrow().get_state());
        if !Self::is_ready(&mainloop, &state_closure) {
            eprintln!("Not ready");
            return sink;
        }

        let op = {
            let mut sink_ref = Rc::clone(&sink);
            let ml_ref = Rc::clone(&mainloop);
            let name = source_name.clone();
            context.borrow_mut().introspect().get_sink_info_list(
                move |sink_list: ListResult<&SinkInfo>| {
                    match sink_list {
                        ListResult::Item(sink_info) => {
                            if let Some(n) = &sink_info.name {
                                if name == n.clone().into_owned()  {
                                    let description: String = if let Some(d) = &sink_info.description {
                                        d.deref().to_owned()
                                    } else {
                                        "".to_string()
                                    };

                                    eprintln!("index: {}", sink_info.index);
                                    eprintln!("name: {}", n);
                                    eprintln!("description: {}", description);
                                    sink_ref = Rc::new(RefCell::new(Some(AudioSink::from_pa_sink_info(sink_info))));
                                }
                            } else {
                                eprintln!("Nameless device at index: {}", sink_info.index);
                            }
                        }
                        ListResult::End => {
                            unsafe {
                                (*ml_ref.as_ptr()).signal(false);
                            }
                        }
                        ListResult::Error => {
                            eprintln!("Listing devices failed");
                            unsafe {
                                (*ml_ref.as_ptr()).signal(false);
                            }
                        }
                    }
                },
            )
        };

        while op.get_state() == pulse::operation::State::Running {
            mainloop.borrow_mut().wait();
        }

        mainloop.borrow_mut().unlock();
        sink
    }

    fn connect_stream(mainloop: &Rc<RefCell<Mainloop>>, context: &Rc<RefCell<Context>>, sink: Rc<RefCell<Option<AudioSink>>>) -> Result<Arc<Mutex<Stream>>, String> {
        let sink_ref = Rc::clone(&sink);
        let spec = sink_ref.borrow().clone().unwrap().spec.unwrap();
        let stream = Arc::new(Mutex::new(
            Stream::new(&mut context.borrow_mut(), "led-speaker", &spec, None)
                .expect("Failed to create new stream"),
        ));

        mainloop.borrow_mut().lock();
        let mainloop_ref = Rc::clone(&mainloop);

        {
            let weak_stream = Arc::downgrade(&stream);
            stream.lock().unwrap().set_state_callback(Some(Box::new(move || {
                match weak_stream.upgrade() {
                    Some(stream) => {
                        if let Ok(res) = stream.try_lock() {
                            let state = res.get_state();
                            match state {
                                pulse::stream::State::Ready
                                | pulse::stream::State::Failed
                                | pulse::stream::State::Terminated => unsafe {
                                    (*mainloop_ref.as_ptr()).signal(false);
                                },
                                _ => {}
                            }
                        } else {
                            eprintln!("PulseAudio state callback for locked stream.");
                        }
                    },
                    None => {
                        eprintln!("Stream has been dropped");
                    }
                }
            })));
        }

        stream.lock().unwrap().connect_record(
            Some(sink_ref.borrow().clone().unwrap().name.as_str()),
            None,
            flags::START_UNMUTED & flags::START_CORKED,
        ).expect("Could not connect stream");
        let state_closure = || ReadyState::Stream(stream.clone().try_lock().unwrap().get_state());
        if !Self::is_ready(&mainloop, &state_closure) {
            eprintln!("Error connecting stream.");
        }

        mainloop.borrow_mut().unlock();
        Ok(stream)
    }

    fn disconnect_stream(mainloop: &Rc<RefCell<Mainloop>>, stream: &Arc<Mutex<Stream>>) -> Result<bool, PAErr> {
        mainloop.borrow_mut().lock();
        let mut s = stream.lock().unwrap();
        s.cork(None);
        s.flush(None);
        s.set_state_callback(None);
        s.disconnect()?;
        mainloop.borrow_mut().unlock();
        Ok(true)
    }

    fn is_ready(mainloop: &Rc<RefCell<Mainloop>>, state_closure: &Fn() -> ReadyState) -> bool {
        loop {
            let mainloop = &mainloop;
            match state_closure() {
                ReadyState::Stream(state) => match state {
                    pulse::stream::State::Ready => {
                        return true;
                    }
                    pulse::stream::State::Failed | pulse::stream::State::Terminated => {
                        mainloop.borrow_mut().unlock();
                        mainloop.borrow_mut().stop();
                        return false;
                    }
                    _ => {
                        mainloop.borrow_mut().wait();
                    }
                },
                ReadyState::Context(state) => match state {
                    pulse::context::State::Ready => {
                        return true;
                    }
                    pulse::context::State::Failed | pulse::context::State::Terminated => {
                        mainloop.borrow_mut().unlock();
                        mainloop.borrow_mut().stop();
                        return false;
                    }
                    _ => {
                        mainloop.borrow_mut().wait();
                    }
                },
            }
        }
    }
}
