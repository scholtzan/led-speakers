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
    sink: Rc<RefCell<AudioSink>>,
    name: String,
    context: Rc<RefCell<Context>>,
    mainloop: Rc<RefCell<Mainloop>>,
}


impl AudioStream {
    pub fn new(name: String, source_name: String) -> AudioStream {
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

        let sink = AudioSink::new(source_name.clone(), 0, source_name, None);

        AudioStream {
            sink: Rc::new(RefCell::new(sink)),
            name,
            context,
            mainloop
        }
    }

    pub fn start(&self) {
        self.context
            .borrow_mut()
            .connect(None, pulse::context::flags::NOFLAGS, None)
            .expect("Failed to connect context");
        self.mainloop.borrow_mut().lock();
        self.mainloop.borrow_mut().start().expect("Failed to start mainloop");
        
        let state_closure = || ReadyState::Context(self.context.borrow().get_state());
        if !self.is_ready(&state_closure) {
            return;
        }

        let op = {
            let ml_ref = Rc::clone(&self.mainloop);
            let sink_ref = Rc::clone(&self.sink);
            let name = sink_ref.borrow().source_name.clone();
            self.context.borrow_mut().introspect().get_sink_info_list(
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
                                    let sink = AudioSink::from_pa_sink_info(sink_info);
                                    *sink_ref.borrow_mut() = sink;
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
            self.mainloop.borrow_mut().wait();
        }

        self.mainloop.borrow_mut().unlock();
    }

    fn connect_stream(&self) -> Result<Arc<Mutex<Stream>>, String> {
        let sink_ref = Rc::clone(&self.sink);
        let spec = sink_ref.borrow().spec.clone().unwrap();
        let stream = Arc::new(Mutex::new(
            Stream::new(&mut self.context.borrow_mut(), "led-speaker", &spec, None)
                .expect("Failed to create new stream"),
        ));

        self.mainloop.borrow_mut().lock();
        let mainloop_ref = Rc::clone(&self.mainloop);

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
            Some(sink_ref.borrow().name.clone().as_str()),
            None,
            flags::START_UNMUTED & flags::START_CORKED,
        ).expect("Could not connect stream");
        let state_closure = || ReadyState::Stream(stream.clone().try_lock().unwrap().get_state());
        if !self.is_ready(&state_closure) {
            eprintln!("Error connecting stream.");
        }

        self.mainloop.borrow_mut().unlock();
        Ok(stream)
    }

    fn is_ready(&self, state_closure: &Fn() -> ReadyState) -> bool {
        loop {
            let mainloop = &self.mainloop;
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


