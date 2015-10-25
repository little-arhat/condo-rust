
// traits
use std::error::Error;
// std
use std::time::Duration;
use std::thread;

pub fn sleep(seconds: u64) {
    thread::sleep(Duration::new(seconds, 0));
}

pub fn error_description(e: &Error) -> &str {
    match e.cause() {
        Some(inner) => inner.description(),
        None => e.description()
    }
}

#[macro_export]
macro_rules! ignore_result{
    ( $( $x:expr ),* ) => {
        $(let _ = $x)*
    }
}
