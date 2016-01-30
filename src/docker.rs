
// ext libs
use hyper;
use hyper::{header};
use serde_json;
use serde_json::JSONStream;
// traits
use std::io::{Read};
use std::fmt;
use std::error;
use std::convert::From;
// internal
use human_uri::HumanURI;
use utils::*;
use spec;

#[derive(Debug)]
pub enum DockerError {
    HTTPError(String, String), // invalid status, etc
    IOError(hyper::Error, String), // no connection, etc
    ProtocolError(Option<serde_json::error::Error>, String), // invalid json/header, etc
    RequestError(String), // error processing request: invalid params, etc
}

impl error::Error for DockerError {
    fn description(&self) -> &str {
        match self {
            &DockerError::HTTPError(ref msg, _) => msg,
            &DockerError::IOError(_, ref s) => s,
            &DockerError::ProtocolError(_, ref s) => s,
            &DockerError::RequestError(ref s) => s,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &DockerError::IOError(ref err, _) => Some(err),
            &DockerError::ProtocolError(ref maybe_err, _) =>
                match maybe_err {
                    &Some(ref err) => Some(err),
                    &None => None
                },
            _ => None,
        }
    }
}

impl fmt::Display for DockerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &DockerError::HTTPError(ref msg, _) =>
                write!(f, "Docker HTTP Error: {}", msg),
            &DockerError::IOError(_, ref s) =>
                write!(f, "Docker IO error: {}", s),
            &DockerError::ProtocolError(_, ref s) =>
                write!(f, "Docker protocol error: {}", s),
            &DockerError::RequestError(ref s) =>
                write!(f, "Docker request error: {}", s),
        }
    }
}

impl From<hyper::Error> for DockerError {
    fn from(err: hyper::Error) -> DockerError {
        let d = error_details(&err);
        DockerError::IOError(err, d)
    }
}

impl From<serde_json::error::Error> for DockerError {
    fn from(err: serde_json::error::Error) -> DockerError {
        let d = error_details(&err);
        DockerError::ProtocolError(Some(err), d)
    }
}


pub struct Docker {
    client: hyper::Client,
    endpoint: HumanURI
}

impl Docker {
    #[inline]
    pub fn new(raw_uri: &str) -> Docker {
        // TODO: custom ssl certificates
        let endpoint = HumanURI::parse(raw_uri);
        Docker{
            client: hyper::Client::new(),
            endpoint: endpoint
        }
    }

    pub fn pull_image(&self, image: &spec::Image) -> Result<(), DockerError> {
        // TODO: ADD X-Registry-Auth header
        let url = self.endpoint.with_path("/images/create")
            .with_query_params([("fromImage", &image.name),
                                ("tag", &image.tag)].iter());
        debug!("POST {}...", url);
        let mut response = try!(self.client.post(url)
                                .header(header::Connection::close())
                            .send());
        if response.status != hyper::Ok {
            let mut body = String::new();
            response.read_to_string(&mut body).unwrap();
            return Err(DockerError::HTTPError(format!("{}", response.status),
                                              body));
        }
        let progress:JSONStream<serde_json::Value, _> = JSONStream::new(response.bytes());
        for msg in progress {
            let msg = try!(msg); // unwrap it
            info!("status: {:?}", msg);
            match msg.lookup("error") {
                Some(error) => match error {
                    &serde_json::Value::String(ref s) =>
                        return Err(DockerError::RequestError(s.clone())),
                    _ => return Err(DockerError::ProtocolError(
                        None, "Invalid error value".to_string()))
                },
                None => ()
            };
        };
        let x = try!(self.receive_image_id(image));
        info!("{:}", x);
        return Ok(());
    }

    fn receive_image_id(&self, image: &spec::Image) -> Result<String, DockerError> {
        let url = self.endpoint
            .with_path("images")
            .add_path(format!("{}:{}", &image.name, &image.tag))
            .add_path("json");
        debug!("GET {}...", url);
        let mut response = try!(self.client.get(url)
                            .header(header::Connection::close())
                            .send());
        if response.status != hyper::Ok {
            let mut body = String::new();
            response.read_to_string(&mut body).unwrap();
            return Err(DockerError::HTTPError(format!("{}", response.status),
                                              body));
        }
        let result:serde_json::Value = try!(serde_json::from_reader(response));
        match result.lookup("Id") {
            Some(id) => match id {
                &serde_json::Value::String(ref s) => Ok(s.clone()),
                _ => Err(DockerError::ProtocolError(None,
                                                    "Invalid id value".to_string()))
            },
            None => Err(DockerError::ProtocolError(None,
                                                   "No id value found".to_string()))
        }
    }

}
