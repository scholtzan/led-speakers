use pulse::callbacks::ListResult;
use pulse::context::introspect::SourceInfo;
use pulse::context::Context;
use pulse::error::PAErr;
#[allow(unused_imports)]
use pulse::mainloop::api::Mainloop as MainloopTrait;
use pulse::mainloop::threaded::Mainloop;
use pulse::proplist::Proplist;
use pulse::sample::{Format, Spec};
use pulse::stream::{PeekResult, Stream};
use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::thread::JoinHandle;
use std::time;

use crate::buffer::Buffer;

/// Wrapper for PulseAudio context and stream states
enum State {
    Stream(pulse::stream::State),

    Context(pulse::context::State),
}

/// Representation of a PulseAudio source
#[derive(Clone, Debug)]
pub struct AudioSource {
    /// Name of the audio source
    name: String,

    /// PulseAudio source spec
    pub spec: Option<Spec>,

    /// Sampling rate
    pub rate: u32,

    /// Name of the monitoring sink
    pub sink_name: String,
}

impl AudioSource {
    /// Create a new `AudioSource` instane based on the PulseAudio source info.
    fn from_pa_source_info(source_info: &SourceInfo) -> AudioSource {
        /// get the audio source name
        let name = match &source_info.name {
            None => String::from("Unnamed audio source"),
            Some(Cow::Borrowed(inner_name)) => String::from(*inner_name),
            Some(Cow::Owned(inner_name)) => inner_name.clone(),
        };

        // get the audio sink name
        let sink_name = match &source_info.monitor_of_sink_name {
            None => String::from("Unnamed audio sink"),
            Some(Cow::Borrowed(inner_name)) => String::from(*inner_name),
            Some(Cow::Owned(inner_name)) => inner_name.clone(),
        };

        AudioSource {
            name: name,
            spec: Some(source_info.sample_spec.clone()),
            rate: source_info.sample_spec.rate,
            sink_name,
        }
    }
}

impl Default for AudioSource {
    fn default() -> Self {
        AudioSource {
            name: String::from("Audio Source"),
            spec: None,
            rate: 44100,
            sink_name: String::from("Audio Sink"),
        }
    }
}

/// Representation of the PulseAudio audio stream data
pub struct AudioStream {
    /// Identifier for the audio stream
    name: String,

    // Name of the audio sink
    sink_name: String,

    /// Handle for thread that is periodically fetching audio data
    handle: Option<JoinHandle<()>>,

    /// Whether audio stream has been stopped
    killed: Arc<AtomicBool>,

    /// Buffer audio data is written into
    pub buffer: Option<Arc<Buffer>>,

    /// Sampling rate of source
    pub rate: Arc<Mutex<u32>>,
}

impl AudioStream {
    /// Create a new `AudioStream` instance and connects to the provided audio sink.
    ///
    /// # Arguments
    ///
    /// * `name`: identifier for the created audio stream
    /// * `sink_name`: name of the audio sink to connect to
    pub fn new(name: String, sink_name: String) -> AudioStream {
        let mut audio_stream = AudioStream {
            name: name.clone(),
            sink_name: sink_name.clone(),
            killed: Arc::new(AtomicBool::from(false)),
            handle: None,
            buffer: None,
            rate: Arc::new(Mutex::new(0)),
        };

        // start reading audio from source and writing into buffer
        audio_stream.buffer = Some(audio_stream.start().expect("Could not create audio stream"));

        // create temporary context and main loop to connect to audio source
        let (mainloop, context) = Self::init(name);
        let source = AudioStream::connect_source(&mainloop, &context, sink_name);
        let spec = source.borrow().clone().unwrap().spec.clone();
        // store rate; required for audio processing
        audio_stream.rate = Arc::new(Mutex::new(spec.unwrap().rate));

        audio_stream
    }

    /// Initializes the audio stream.
    ///
    /// Creates a PulseAudio main loop and context that will be used to
    /// communicate with PulseAudio.
    fn init(name: String) -> (Rc<RefCell<Mainloop>>, Rc<RefCell<Context>>) {
        let mut proplist = Proplist::new().unwrap();
        proplist
            .set(
                pulse::proplist::properties::APPLICATION_NAME,
                &name.as_bytes(),
            )
            .unwrap();

        // create main loop and context
        let mainloop = Rc::new(RefCell::new(
            Mainloop::new().expect("Failed to create mainloop"),
        ));
        let context = Rc::new(RefCell::new(
            Context::new_with_proplist(mainloop.borrow().deref(), &name, &proplist)
                .expect("Failed to create context"),
        ));

        {
            let ml_ref = Rc::clone(&mainloop);
            let context_ref = Rc::clone(&context);
            // called whenever teh stream state changes
            context.borrow_mut().set_state_callback(Some(Box::new(move || {
                // callbacks are asynchronous
                // to convert them into synchronous ones, signals need to be sent to the main loop
                let state = unsafe { (*context_ref.as_ptr()).get_state() };
                match state {
                    pulse::context::State::Ready => unsafe {
                        // a signal needs to be sent to the main loop to indicate that the callback has been called
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

    /// Starts recording data from the audio sink and writing it into a buffer.
    ///
    /// A separate thread is created for periodically fetching new audio data and
    /// writing it into a buffer.
    fn start(&mut self) -> Result<Arc<Buffer>, String> {
        // members that need to be made available in thread
        let weak_killed: Weak<AtomicBool> = Arc::downgrade(&self.killed);
        let buffer = Arc::new(Buffer::new(30000)); // todo: configurable and make member
        let buf = buffer.clone();
        let rate = Arc::clone(&self.rate);
        let name = self.name.clone();
        let sink_name = self.sink_name.clone();

        self.handle = Some(thread::spawn(move || {
            let (mainloop, context) = Self::init(name);
            // connect to audio source
            let source = AudioStream::connect_source(&mainloop, &context, sink_name);
            let spec = source.borrow().clone().unwrap().spec.clone();

            // determine the sample rate
            *(rate.lock().unwrap()) = spec.unwrap().rate;

            // connect to audio stream
            let source_name = source.borrow().clone().unwrap().name.clone();
            let stream = Self::connect_stream(&mainloop, &context, spec, source_name)
                .expect("Error creating stream");
            let mut pa_stream = stream.lock().unwrap();

            mainloop.borrow_mut().lock();
            // resume paused playback of stream
            pa_stream.uncork(None);
            mainloop.borrow_mut().unlock();

            loop {
                let killed = weak_killed.upgrade();
                if killed.is_some() && !killed.unwrap().load(Ordering::Relaxed) {
                    mainloop.borrow_mut().lock();
                    // get number of bytes that can be read from stream
                    let available = pa_stream.readable_size();
                    mainloop.borrow_mut().unlock();

                    // wait until byte threshold is reached
                    if let Some(count) = available {
                        if count < 128 {
                            // todo: make configurable
                            thread::sleep(time::Duration::from_micros(200));
                            continue;
                        }
                    }

                    // read from the stream
                    let mut written = 0;
                    mainloop.borrow_mut().lock();
                    let peek = pa_stream.peek().expect("Could not peek PulseAudio stream");
                    match peek {
                        PeekResult::Empty => {
                            mainloop.borrow_mut().unlock();
                            thread::sleep(time::Duration::from_micros(200));
                            continue;
                        }
                        PeekResult::Hole(_) => {
                            // discard data in the stream buffer
                            pa_stream.discard().unwrap();
                            mainloop.borrow_mut().unlock();
                        }
                        PeekResult::Data(data) => {
                            let read = data.len();
                            let mut s: i32 = 100; // todo: make configurable

                            // try writing data to the buffer
                            // if buffer is full; retry in next iteration
                            while s > 0 {
                                // check if there is enough space left in the buffer
                                let buffer_avail = buf.reserve(data.len());
                                if buffer_avail > data.len() {
                                    buf.write(data);
                                    written = data.len();
                                }
                                s -= 1;

                                // done writing all the available data
                                if written >= read {
                                    break;
                                }
                            }

                            // discard any audio data left in the buffer
                            pa_stream
                                .discard()
                                .expect("Could not discard PulseAudio stream");
                            mainloop.borrow_mut().unlock();
                        }
                    }
                } else {
                    // reading audio has been stopped
                    let disconnect = Self::disconnect_stream(&mainloop, &stream);
                    match disconnect {
                        Ok(_) => {}
                        Err(_) => eprintln!("Could not disconnect stream"),
                    }
                    mainloop.borrow_mut().stop();
                    break;
                }
            }
        }));

        Ok(buffer.clone())
    }

    /// Connects to audio source that is monitoring the specified sink.
    fn connect_source(
        mainloop: &Rc<RefCell<Mainloop>>,
        context: &Rc<RefCell<Context>>,
        sink_name: String,
    ) -> Rc<RefCell<Option<AudioSource>>> {
        context
            .borrow_mut()
            .connect(None, pulse::context::FlagSet::NOFLAGS, None)
            .expect("Failed to connect context");
        mainloop.borrow_mut().lock();
        mainloop
            .borrow_mut()
            .start()
            .expect("Failed to start mainloop");

        let source = Rc::new(RefCell::new(None));

        let state_closure = || State::Context(context.borrow().get_state());
        if !Self::is_ready(&mainloop, &state_closure) {
            return source;
        }

        let op = {
            let ml_ref = Rc::clone(&mainloop);
            let source_ref = Rc::clone(&source);
            let name = sink_name.clone();

            // iterate through all existing sources and connect to source that
            // owns the configured sink
            context.borrow_mut().introspect().get_source_info_list(
                move |source_list: ListResult<&SourceInfo>| match source_list {
                    ListResult::Item(source_info) => {
                        if let Some(n) = &source_info.monitor_of_sink_name {
                            if name == n.clone().into_owned() {
                                let description: String = if let Some(d) = &source_info.description
                                {
                                    d.deref().to_owned()
                                } else {
                                    "".to_string()
                                };

                                eprintln!("index: {}", source_info.index);
                                eprintln!("name: {}", n);
                                eprintln!("description: {}", description);

                                // connect to source
                                *(source_ref.borrow_mut()) =
                                    Some(AudioSource::from_pa_source_info(source_info));
                            }
                        }
                    }
                    ListResult::End => unsafe {
                        (*ml_ref.as_ptr()).signal(false);
                    },
                    ListResult::Error => {
                        eprintln!("Listing devices failed");
                        unsafe {
                            (*ml_ref.as_ptr()).signal(false);
                        }
                    }
                },
            )
        };

        while op.get_state() == pulse::operation::State::Running {
            mainloop.borrow_mut().wait();
        }

        mainloop.borrow_mut().unlock();
        source
    }

    /// Connects the PulseAudio stream to the audio source and starts recording.
    fn connect_stream(
        mainloop: &Rc<RefCell<Mainloop>>,
        context: &Rc<RefCell<Context>>,
        spec: Option<Spec>,
        source_name: String,
    ) -> Result<Arc<Mutex<Stream>>, String> {
        // create new audio stream object based on specified spec
        let spec = spec.unwrap();
        let stream = Arc::new(Mutex::new(
            Stream::new(&mut context.borrow_mut(), "led  speaker", &spec, None)
                .expect("Failed to create new stream"),
        ));

        mainloop.borrow_mut().lock();
        let mainloop_ref = Rc::clone(&mainloop);

        {
            let weak_stream = Arc::downgrade(&stream);
            // define callback invoked when state of the stream changes
            stream
                .lock()
                .unwrap()
                .set_state_callback(Some(Box::new(move || match weak_stream.upgrade() {
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
                    }
                    None => {
                        eprintln!("Stream has been dropped");
                    }
                })));
        }

        {
            // connect stream to the configured source
            stream
                .lock()
                .unwrap()
                .connect_record(
                    Some(source_name.as_str()),
                    None,
                    pulse::stream::FlagSet::START_UNMUTED & pulse::stream::FlagSet::START_CORKED,
                )
                .expect("Could not connect stream");
        }

        // wait until stream has been successfully connected
        let state_closure = || State::Stream(stream.clone().try_lock().unwrap().get_state());
        if !Self::is_ready(&mainloop, &state_closure) {
            eprintln!("Error connecting stream.");
        }

        mainloop.borrow_mut().unlock();
        Ok(stream)
    }

    /// Disconnects the audio stream.
    fn disconnect_stream(
        mainloop: &Rc<RefCell<Mainloop>>,
        stream: &Arc<Mutex<Stream>>,
    ) -> Result<bool, PAErr> {
        mainloop.borrow_mut().lock();
        let mut s = stream.lock().unwrap();
        s.cork(None);
        s.flush(None);
        s.set_state_callback(None);
        s.disconnect()?;
        mainloop.borrow_mut().unlock();
        Ok(true)
    }

    /// Checks if the provided state is `Ready`.
    fn is_ready(mainloop: &Rc<RefCell<Mainloop>>, state_closure: &Fn() -> State) -> bool {
        loop {
            let mainloop = &mainloop;
            // check fi the stream of context are ready
            match state_closure() {
                State::Stream(state) => match state {
                    pulse::stream::State::Ready => {
                        return true;
                    }
                    pulse::stream::State::Terminated => {
                        eprintln!("Terminated");
                        mainloop.borrow_mut().unlock();
                        mainloop.borrow_mut().stop();
                        return false;
                    }
                    pulse::stream::State::Failed => {
                        eprintln!("Failed");
                        mainloop.borrow_mut().unlock();
                        mainloop.borrow_mut().stop();
                        return true;
                    }
                    _ => {
                        mainloop.borrow_mut().wait();
                    }
                },
                State::Context(state) => match state {
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
