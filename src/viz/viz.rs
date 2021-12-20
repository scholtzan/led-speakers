use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use dyn_clone::DynClone;


#[typetag::serde]
pub trait Viz: DynClone + Sync + Send {
    fn get_name(&self) -> &str;
    fn get_pretty_name(&self) -> &str;
    fn update(&mut self);
}


// todo: move threading into viz runner
// todo: move start stop into viz runner
pub struct VizRunner {
    pub viz: Arc<Mutex<Box<dyn Viz>>>,
    pub 
    
    is_stopped: Arc<AtomicBool>
}

impl VizRunner {
    pub fn start(&self) {
        let stopped = self.is_stopped.clone();
        let viz = Arc::clone(&self.viz);

        let handle = thread::spawn(move || {
            while !stopped.load(Ordering::Relaxed) {
                viz.lock().unwrap().update();
            }
        });
    }

    pub fn stop(&mut self) {
        self.is_stopped = Arc::new(AtomicBool::from(false));
    }
}

