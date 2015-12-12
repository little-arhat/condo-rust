
// traits
use std::error::Error;
// std
use std::time::Duration;
use std::thread;
use std::io;
use core::marker::PhantomData;

use serde_json;
use serde_json::error;
use serde::de;

pub fn sleep(seconds: u64) {
    thread::sleep(Duration::new(seconds, 0));
}

pub fn error_details(e: &Error) -> String {
    match e.cause() {
        Some(inner) => {
            format!("{}", inner)
        },
        None => format!("{}", e)
    }
}

#[macro_export]
macro_rules! ignore_result{
    ( $( $x:expr ),* ) => {
        $(let _ = $x)*
    }
}

pub struct JSONStream<T, Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
          T: de::Deserialize
{
    deser: serde_json::de::Deserializer<Iter>,
    _marker: PhantomData<T>,
}

impl <T, Iter>JSONStream<T, Iter>
    where Iter:Iterator<Item=io::Result<u8>>,
          T: de::Deserialize {
    fn new(i: Iter) -> JSONStream<T, Iter> {
        JSONStream {
            deser: serde_json::de::Deserializer::new(i),
            _marker: PhantomData
        }
    }
}

impl <T, Iter>Iterator for JSONStream<T, Iter>
    where Iter:Iterator<Item=io::Result<u8>>,
          T: de::Deserialize {
    type Item = Result<T, error::Error>;
    fn next(&mut self) -> Option<Result<T, error::Error>> {
        match de::Deserialize::deserialize(&mut self.deser) {
            Ok(v) => Some(Ok(v)),
            Err(e) => {
                match e {
                    error::Error::SyntaxError(
                        error::ErrorCode::EOFWhileParsingValue, _, _)
                        => match self.deser.end() {
                            Ok(_) => None,
                            Err(e) => Some(Err(e))
                        },
                    _ => Some(Err(e))
                }
            }
        }
    }
}

pub fn iter_to_stream<T, Iter>(i: Iter) -> JSONStream<T, Iter>
    where Iter: Iterator<Item=io::Result<u8>>,
          T: de::Deserialize {
    JSONStream::new(i)
}
