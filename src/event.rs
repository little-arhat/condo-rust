
// internal
use spec::*;

// TODO: change to Rc<Spec>
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Event {
    NewSpec(Spec),
    DeployFailed,
    GotStable
}
