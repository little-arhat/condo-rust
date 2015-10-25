
// packages
extern crate hyper;

// ext libs
use hyper::{header};
// traits
use std::io::{Read};
use std::fmt;
use std::error;
use std::error::Error;
// std
use std::sync::mpsc;
use std::thread;
// internal
use human_uri::HumanURI;
use utils::*;

#[derive(Debug)]
pub enum ConsulError {
    HTTPError(String, String), // 404, 500, etc
    IOError(hyper::Error) // not resolved, etc
}

impl error::Error for ConsulError {
    fn description(&self) -> &str {
        match self {
            &ConsulError::HTTPError(ref msg, _) => msg.as_str(),
            &ConsulError::IOError(ref e) => error_description(e)
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &ConsulError::IOError(ref err) => Some(err as &error::Error),
            _ => None,
        }
    }
}

impl fmt::Display for ConsulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &ConsulError::HTTPError(ref msg, ref body) => {
                try!(msg.fmt(f));
                body.fmt(f)
            },
            &ConsulError::IOError(ref e) => e.description().fmt(f)
        }
    }
}

pub struct Consul {
    client: hyper::Client,
    endpoint: HumanURI
}

impl Consul {
    #[inline]
    pub fn new(raw_uri: &str) -> Consul {
        let endpoint = HumanURI::parse(raw_uri);
        Consul{
            client: hyper::Client::new(),
            endpoint: endpoint
        }
    }

    pub fn get_key<T:AsRef<str>>(&self, key: T, index: i32) -> Result<String, ConsulError>
    {
        let url = self.endpoint.with_path("/v1/kv")
            .add_path(key)
            .with_query_params([("wait", "10s")].iter())
            .add_query_params([("index", index)].iter());
        info!("Get {}...", url);
        let result = self.client.get(url)
            .header(header::Connection::close())
            .send();
        match result {
            Err(e) => Err(ConsulError::IOError(e)),
            Ok(mut res) => {
                let mut body = String::new();
                res.read_to_string(&mut body).unwrap();
                if res.status != hyper::Ok {
                    Err(ConsulError::HTTPError(
                        format!("Error response, code: {}", res.status),
                        body))
                } else {
                    Ok(body)
                }
            }
        }
    }

    pub fn watch_key<T:AsRef<str> + Send>(self, key: T) -> mpsc::Receiver<Result<String, ConsulError>> {
        let (tx, rx) = mpsc::channel();
        let thread_key = key.as_ref().to_owned();
        thread::spawn(move || {
            loop {
                ignore_result!(tx.send(self.get_key(&thread_key, 0)));
                sleep(10);
            }
        });

        rx
    }
}
