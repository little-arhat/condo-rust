
// ext libs
use serde_json;
// traits
// std
use std::sync::mpsc;
use std::thread;
use std::fmt;
// internal
use spec::*;

#[derive(Clone)]
enum State {
    Start,
    WaitingForFirstStable{candidate: Spec},
    HasStable{stable: Spec, current: Spec}
}

impl State {
    fn candidate(&self) -> Spec {
        match self {
            &State::WaitingForFirstStable{ref candidate} => candidate.to_owned(),
            _ => panic!("Can't happen")
        }
    }
}

impl fmt::Debug for State {
    #[allow(unused_variables)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            &State::Start => "Start",
            &State::WaitingForFirstStable{ref candidate} => "WaitingForFirstStable",
            &State::HasStable{ref stable, ref current} => "HasStable"
        };
        write!(f, "{}", s)
    }
}

pub struct Dispatcher {
    // input: mpsc::Receiver<String>,
    state: State
        // ,
    // stable: Option<Spec>,
    // current: Option<Spec>,
    // candidate: Option<Spec>
}


impl Dispatcher {
    #[inline]
    pub fn new() -> Self {
        Dispatcher{
            // input: input,
            state: State::Start
            // stable: None,
            // current: None,
            // candidate: None
        }
    }

    pub fn start(mut self, input: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            for json_spec in input.iter() {
                let spec:Spec =
                    serde_json::from_str(&json_spec).unwrap();
                debug!("Received spec: {:?}", spec);
                match self.state {
                    State::Start => self.on_spec_received_no_stable(spec),
                    _ => panic!("can't happen")
                }
            }
        })
    }

    fn on_error_no_stable(&mut self) {
        info!("State transition from {:?}...", self.state);
        self.state = State::Start;
        info!("To {:?}!", self.state);
    }

    fn on_got_stable(&mut self) {
        info!("State transition from {:?}...", self.state);
        let candidate = self.state.candidate();
        self.state = State::HasStable{current: candidate.clone(),
                                      stable: candidate};
        info!("To {:?}!", self.state);
    }

    fn on_spec_received_no_stable(&mut self, spec: Spec) {
        info!("State transition from {:?}...", self.state);
        self.state = State::WaitingForFirstStable{candidate: spec};
        info!("To {:?}!", self.state);
        // start candidate, wait for result
        let result = true;
        if result {
            // Healthy
            self.on_got_stable();
        } else {
            // NotHealthy
            self.on_error_no_stable();
        }
    }


}
