
// ext libs
use hyper;
use hyper::{header};
use serde_json;
// traits
use std::io::{Read};
use std::fmt;
use std::error;
use std::error::Error;
use std::convert::From;
// std
use std::sync::mpsc;
use std::thread;
// internal
use human_uri::HumanURI;
use utils::*;


#[derive(Debug)]
pub enum ConsulError {
    HTTPError(String, String), // 404, 500, etc
    IOError(hyper::Error), // not resolved, etc
    ParseError(serde_json::Error), // wrong json
    DataError(String, String) // wrong data inside json
}

impl error::Error for ConsulError {
    fn description(&self) -> &str {
        match self {
            &ConsulError::HTTPError(ref msg, _) => msg.as_str(),
            &ConsulError::IOError(ref e) => error_description(e),
            &ConsulError::ParseError(ref e) => error_description(e),
            &ConsulError::DataError(ref msg, _) => msg.as_str()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &ConsulError::IOError(ref err) => Some(err as &error::Error),
            &ConsulError::ParseError(ref err) => Some(err as &error::Error),
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
            &ConsulError::IOError(ref e) => e.description().fmt(f),
            &ConsulError::ParseError(ref e) => e.description().fmt(f),
            &ConsulError::DataError(ref msg, ref value) => {
                try!(msg.fmt(f));
                value.fmt(f)
            }
        }
    }
}

impl From<hyper::Error> for ConsulError {
    fn from(err: hyper::Error) -> ConsulError {
        ConsulError::IOError(err)
    }
}

impl From<serde_json::Error> for ConsulError {
    fn from(err: serde_json::Error) -> ConsulError {
        ConsulError::ParseError(err)
    }
}

pub struct Consul {
    client: hyper::Client,
    endpoint: HumanURI
}

fn some_or_error<T>(arg: Option<T>,
                    msg: String, details: String) -> Result<T, ConsulError>
{
    match arg {
        Some(a) => Ok(a),
        None => Err(ConsulError::DataError(msg, details))
    }
}

fn extract_consul_value(body: &str) -> Result<String, ConsulError> {
    let parsed:serde_json::Value = try!(serde_json::from_str(body));
    // Looks like boilerplate
    let inner = match parsed.as_array() {
        Some(a) if a.len() == 1 => {
            a[0].to_owned() // Don't like this clone
        },
        _ =>
            return Err(ConsulError::DataError("Expected 1-element array!".to_string(),
                                              format!("{:?}", parsed)))
    };
    let obj = try!(some_or_error(inner.as_object(),
                                 "Expected object!".to_string(),
                                 format!("{:?}", inner)));
    let value = try!(some_or_error(obj.get("Value"),
                                   "No key named \"Value\" found!".to_string(),
                                   format!("{:?}", obj)));
    let encoded = try!(some_or_error(value.as_string(),
                                     "Expected \"Value\" to be string!".to_string(),
                                     format!("{:?}", value))).to_owned();
    info!("{:?}", encoded);
    Ok(encoded)
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
        let mut response = try!(self.client.get(url)
                                .header(header::Connection::close())
                                .send());
        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();
        if response.status != hyper::Ok {
            Err(ConsulError::HTTPError(
                format!("Error response, code: {}", response.status), body))
        } else {
            Ok(try!(extract_consul_value(&body)))
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
