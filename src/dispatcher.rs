
// ext libs
// traits
// std
use std::sync::mpsc;
use std::thread;
use std::fmt;
// internal
use spec::*;
use event::*;

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

#[derive(Clone,Debug)]
enum State {
    Start,
    WaitingForFirstStable{candidate: Deploy},
    RunningStable{current: Deploy},
    WaitingForNewStable{last_stable: Deploy, candidate: Deploy},
    RunningStableWaitingForNew{current: Deploy, candidate: Deploy}
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

pub struct Dispatcher {
    state: State,
    send_events: mpsc::Sender<Event>,
    receive_events: mpsc::Receiver<Event>
}


impl Dispatcher {
    #[inline]
    pub fn new() -> Self {
        let (send_events, receive_events) = mpsc::channel();
        Dispatcher{
            state: State::Start,
            send_events: send_events,
            receive_events: receive_events
        }
    }

    pub fn start(self) -> (thread::JoinHandle<()>, mpsc::Sender<Event>) {
        let send_events = self.send_events.clone();
        let h = thread::spawn(move || self.listen_events());
        (h, send_events)
    }

    fn listen_events(mut self) {
        for event in self.receive_events.iter() {
            debug!("Current state: {}, received event: {:?}", self.state, event);
            self.state = match &self.state {
                &State::Start => match event {
                    Event::NewSpec(spec) => self.start_initial_deploy(spec),
                    _ => panic!("Invalid event")
                },
                &State::RunningStable{ref current} => match event {
                    Event::NewSpec(spec) => match &spec.stop {
                        // move stop dispatch inside
                        &Stop::Before =>
                            self.stop_current_and_start_deploy(current, spec),
                        &Stop::AfterTimeout(_) =>
                            self.start_new_deploy(current, spec)
                    },
                    _ => panic!("Invalid event")
                },
                &State::WaitingForFirstStable{ref candidate} => match event {
                    Event::NewSpec(_) => unimplemented!(),
                    Event::DeployFailed => State::Start,
                    Event::GotStable =>
                        State::RunningStable{current: candidate.to_owned()}
                },
                &State::WaitingForNewStable{ref last_stable, ref candidate} => match event {
                    Event::NewSpec(_) => unimplemented!(),
                    Event::DeployFailed =>
                        self.restore_last_stable(last_stable),
                    Event::GotStable =>
                        State::RunningStable{current: candidate.to_owned()}
                },
                &State::RunningStableWaitingForNew{ref current, ref candidate} => match event {
                    Event::NewSpec(_) => unimplemented!(),
                    Event::DeployFailed =>
                        State::RunningStable{current: current.to_owned()},
                    Event::GotStable =>
                        self.replace_old_stable(current, candidate)
                }
            };
            debug!("Transitioned to state: {}", self.state);
        }
    }

    fn start_initial_deploy(&self, init: Spec) -> State {
        let state = State::WaitingForFirstStable{
            candidate: Deploy::new(init)
        };
        // RUN DEPLOY
        state
    }

    fn stop_current_and_start_deploy(&self, current: &Deploy, new: Spec) -> State {
        // STOP CURRENT (TODO: actually stop just before new strt)
        let state = State::WaitingForNewStable{
            candidate: Deploy::new(new),
            last_stable: current.to_owned()
        };
        // RUN DEPLOY
        state
    }

    fn start_new_deploy(&self, current: &Deploy, new: Spec) -> State {
        let state = State::RunningStableWaitingForNew{
            current: current.to_owned(),
            candidate: Deploy::new(new)
        };
        // RUN DEPLOY OF candaidate
        state
    }

    fn restore_last_stable(&self, last_stable: &Deploy) -> State {
        let state = State::WaitingForFirstStable{
            candidate: last_stable.to_owned()
        };
        // RUN DEPLOY
        state
    }

    fn replace_old_stable(&self, current: &Deploy, new: &Deploy) -> State {
        // Schedule to stop old
        debug!("schedule to stop old: {:?}", current);
        let state = State::RunningStable{current: new.to_owned()};
        state
    }
}
