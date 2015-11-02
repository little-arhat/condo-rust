
// ext libs
use serde_json;
// traits
// std
use std::sync::mpsc;
use std::thread;
use std::fmt;
// internal
use spec::*;

#[derive(Clone, Debug)]
struct Deploy {
    spec: Spec
}

impl Deploy {
    fn new(spec: Spec) -> Self {
        Deploy{
            spec: spec
        }
    }
}

#[derive(Clone, Debug)]
enum Event {
    NewSpec(Spec)
}

#[derive(Clone,Debug)]
enum State {
    Start,
    WaitingForFirstStable{candidate: Deploy},
    RunningStable{current: Deploy},
    WaitingForNewStable{last_stable: Deploy, candidate: Deploy},
    RunningStableWaitingForNew{current: Deploy, candidate: Deploy}
}

impl State {
    fn candidate(self) -> Deploy {
        match self {
            State::WaitingForFirstStable{candidate} => candidate,
            State::WaitingForNewStable{candidate, ..} => candidate,
            State::RunningStableWaitingForNew{candidate, ..} => candidate,
            _ => panic!("Can't happen")
        }
    }

    fn current(self) -> Deploy {
        match self {
            State::RunningStable{current} => current,
            State::RunningStableWaitingForNew{current, ..} => current,
            _ => panic!("Can't happen")
        }
    }

    fn stable(self) -> Deploy {
        match self {
            State::RunningStable{current} => current,
            State::WaitingForNewStable{last_stable, ..} => last_stable,
            State::RunningStableWaitingForNew{current, ..} => current,
            _ => panic!("Can't happen")
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            &State::Start => "Start",
            &State::WaitingForFirstStable{..} => "WaitingForFirstStable",
            &State::WaitingForNewStable{..} => "WaitingForNewStable",
            &State::RunningStable{..} => "RunningStable",
            &State::RunningStableWaitingForNew{..} => "RunningStableWaitingForNew"
        };
        write!(f, "{}", s)
    }
}

fn spec_parser(jsons: mpsc::Receiver<String>, events: mpsc::Sender<Event>) {
    for json_spec in jsons.iter() {
        debug!("Received json spec: {}", json_spec);
        let res_spec:serde_json::Result<Spec> = serde_json::from_str(&json_spec);
        match res_spec {
            Ok(spec) => {
                ignore_result!(events.send(Event::NewSpec(spec)));
            },
            Err(e) => {
                warn!("Error while parsing spec: {}, ignore...", e);
            }
        }
    }
}

pub struct Dispatcher {
    state: State
}


impl Dispatcher {
    #[inline]
    pub fn new() -> Self {
        Dispatcher{
            state: State::Start
        }
    }

    pub fn start(mut self, input: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let (send_events, receive_events) = mpsc::channel();
            thread::spawn(move || spec_parser(input, send_events.clone()));

            for event in receive_events.iter() {
                debug!("Received event: {:?}", event);
                match self.state {
                    State::Start => match event {
                        Event::NewSpec(spec) => {
                            self = self.on_spec_received_no_stable(spec)
                        }
                    },
                    State::RunningStable{..} => match event {
                        Event::NewSpec(spec) => {
                            self = self.on_spec_received(spec)
                        }
                    },
                    _ => panic!("can't happen")
                }
            }
        })
    }

    fn start_transition(&self) {
        info!("State transition from {}...", self.state);
    }

    fn end_transition(&self) {
        info!("To {}!", self.state);
    }

    fn on_error_no_stable(mut self) -> Self {
        self.start_transition();
        self.state = State::Start;
        self.end_transition();
        self
    }

    fn on_error_restore_stable(mut self) -> Self {
        self.start_transition();
        // restore last stable
        let stable = self.state.stable();
        // run stable
        self.state = State::RunningStable{current: stable};
        self.end_transition();
        self
    }

    fn on_error_discard_candidate(mut self) -> Self {
        self.start_transition();
        // run stable
        self.state = State::RunningStable{current: self.state.stable()};
        self.end_transition();
        self
    }

    fn on_got_stable(mut self) -> Self {
        self.start_transition();
        self.state = State::RunningStable{current: self.state.candidate()};
        self.end_transition();
        self
    }

    fn on_spec_received(mut self, spec: Spec) -> Self {
        self.start_transition();
        match spec.stop {
            Stop::Before => {
                // stop current
                let current = self.state.current();
                self.state = State::WaitingForNewStable{
                    candidate: Deploy::new(spec),
                    last_stable: current
                };
                self.end_transition();
                // start candidate, wait for results
                let result = true;
                if result {
                    // healthy
                    self.on_got_stable()
                } else {
                    // eror
                    self.on_error_restore_stable()
                }
            },
            Stop::AfterTimeout(_) => {
                // start candidate
                self.state = State::RunningStableWaitingForNew{
                    current: self.state.current(),
                    candidate: Deploy::new(spec)
                };
                self.end_transition();
                // start candidate, wait for results
                let result = true;
                if result {
                    // healthy
                    self.on_got_stable()
                } else {
                    // error
                    self.on_error_discard_candidate()
                }
            }
        }
    }

    fn on_spec_received_no_stable(mut self, spec: Spec) -> Self {
        self.start_transition();
        self.state = State::WaitingForFirstStable{candidate: Deploy::new(spec)};
        self.end_transition();
        let result = true;
        if result {
            // Healthy
            self.on_got_stable()
        } else {
            // NotHealthy
            self.on_error_no_stable()
        }
    }

}
