
// ext libs
use serde_json;
// traits
// std
use std::sync::mpsc;
use std::thread;
// internal
use spec::*;

pub struct Condo {
    specs: mpsc::Receiver<String>
}


impl Condo {
    #[inline]
    pub fn new(input: mpsc::Receiver<String>) -> Self {
        Condo{
            specs: input
        }
    }

    pub fn start(self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            for json_spec in self.specs.iter() {
                let spec:Spec =
                    serde_json::from_str(&json_spec).unwrap();
                info!("Received spec: {:?}", spec);
            }
        })
    }
}
