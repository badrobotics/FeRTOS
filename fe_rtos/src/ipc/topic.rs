extern crate alloc;

use crate::ipc::subscriber::Subscriber;
use alloc::vec::Vec;
use alloc::string::String;
pub(crate) struct Topic {
    pub(crate) name: String,
    pub(crate) data: Vec<Vec<u8>>,
    pub(crate) subscribers: Vec<Subscriber>,
}

impl Topic {
    pub(crate) fn new(name: &String) -> Topic {
        Topic {
            name: name.clone(),
            data: Vec::new(),
            subscribers: Vec::new(),
        }
    }
}