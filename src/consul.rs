
// ext libs
use hyper;
use hyper::{header};
// traits
use std::io::{Read};
use std::fmt;
use std::error;
use std::error::Error;
use std::convert::From;
use std::ops::Deref;
// std
use std::sync::mpsc;
use std::thread;
// internal
use human_uri::HumanURI;
use utils::*;


#[derive(Debug)]
pub enum ConsulError {
    HTTPError(String, String), // 404, 500, etc
    IOError(hyper::Error, String), // not resolved, etc
    DataError(String) // wrong data inside json
}

impl error::Error for ConsulError {
    fn description(&self) -> &str {
        match self {
            &ConsulError::HTTPError(ref msg, _) => msg,
            &ConsulError::IOError(_, ref s) => s,
            &ConsulError::DataError(ref msg) => msg
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &ConsulError::IOError(ref err, _) => Some(err),
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
            &ConsulError::IOError(_, ref s) =>
                write!(f, "Consul IO error: {}", s),
            &ConsulError::DataError(ref msg) => msg.fmt(f)
        }
    }
}

impl From<hyper::Error> for ConsulError {
    fn from(err: hyper::Error) -> ConsulError {
        let d = error_details(&err);
        ConsulError::IOError(err, d)
    }
}

header! {(XConsulIndex, "x-consul-index") => [i64] }

pub enum ConsulKeyResponse {
    NoNewContent,
    Key(String, i64)
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

    pub fn get_key<T:AsRef<str>>(&self, key: T, index: i64)
                                 -> Result<ConsulKeyResponse, ConsulError>
    {
        let url = self.endpoint.with_path("/v1/kv")
            .add_path(key)
            .with_query_params([("wait", "10s"), ("raw", "")].iter())
            .add_query_params([("index", index)].iter());
        debug!("Get {}...", url);
        let mut response = try!(self.client.get(url)
                                .header(header::Connection::close())
                                .send());
        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();
        if response.status != hyper::Ok {
            return Err(ConsulError::HTTPError(format!("{}", response.status),
                                              body));
        }
        match response.headers.get::<XConsulIndex>() {
            Some(new_index) if *new_index.deref() == index =>
                Ok(ConsulKeyResponse::NoNewContent),
            Some(new_index) =>
                Ok(ConsulKeyResponse::Key(body.to_owned(), *new_index.deref())),
            None =>
                Err(ConsulError::DataError("No Index Received".to_owned()))
        }
    }

    pub fn watch_key<T:AsRef<str> + Send>(self, key: T) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel();
        let thread_key = key.as_ref().to_owned();
        thread::spawn(move || {
            let mut index = 0;
            loop {
                match self.get_key(&thread_key, index) {
                    Err(e) => error!("Consul error:{}", e),
                    Ok(ConsulKeyResponse::NoNewContent) =>
                        debug!("No new content received..."),
                    Ok(ConsulKeyResponse::Key(spec, new_index)) => {
                        index = new_index;
                        ignore_result!(tx.send(spec));
                    }
                };
                sleep(5);
            }
        });

        rx
    }
}
