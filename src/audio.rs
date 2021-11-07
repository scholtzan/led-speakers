use pulse::callbacks::ListResult;
use pulse::context::introspect::SourceInfo;
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


struct AudioContext {
    context: Rc<RefCell<Context>>,
    mainloop: Rc<RefCell<Mainloop>>,
}


#[derive(Clone, Debug)]
pub struct AudioSource {
    name: String,
    index: u32,
    pub rate: u32,
    channels: u8,
    sample_format: Format,
}

impl AudioSource {
    fn from_pa_source_info(source_info: &SourceInfo) -> AudioSource {
        let name = match &source_info.name {
            None => String::from("Unnamed audio source"),
            Some(Cow::Borrowed(inner_name)) => String::from(*inner_name),
            Some(Cow::Owned(inner_name)) => inner_name.clone(),
        };

        AudioSource {
            name: name,
            index: source_info.index,
            rate: source_info.sample_spec.rate,
            sample_format: source_info.sample_spec.format,
            channels: source_info.sample_spec.channels,
        }
    }

    pub fn byte_rate(&self) -> u64 {
        self.channels as u64 * self.rate as u64 * (self.sample_format.size()) as u64
    }

    fn sample_bytes(&self) -> u32 {
        self.channels as u32 * self.sample_format.size() as u32
    }
}

impl Default for AudioSource {
    fn default() -> Self {
        AudioSource {
            name: String::from("Audio Source"),
            index: 0,
            rate: 44100,
            sample_format: Format::S16le,
            channels: 2,
        }
    }
}


// pub struct AudioStream {
//     source: AudioSource
// }


// impl AudioStream {
//     pub fn new() -> AudioStream {

//     }
// }


