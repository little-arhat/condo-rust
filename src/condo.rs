
// ext libs
use serde_json;
// traits
// std
use std::sync::mpsc;
use std::thread;
// internal



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
            for spec in self.specs.iter() {
                info!("Spec received: {}", spec);
            }
        })
    }
}
