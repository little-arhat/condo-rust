
// ext libs
use hyper;
use hyper::{header};
// traits
use std::io::{Read};
use std::fmt;
use std::error;
use std::error::Error;
use std::convert::From;

// internal
use human_uri::HumanURI;
use utils::*;
use spec;

#[derive(Debug)]
pub enum DockerError {
    HTTPError(String, String),
    IOError(hyper::Error)
}

impl error::Error for DockerError {
    fn description(&self) -> &str {
        match self {
            &DockerError::HTTPError(ref msg, _) => msg.as_str(),
            &DockerError::IOError(ref e) => error_description(e),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &DockerError::IOError(ref err) => Some(err as &error::Error),
            _ => None,
        }
    }
}

impl fmt::Display for DockerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &DockerError::HTTPError(ref msg, ref body) => {
                try!(msg.fmt(f));
                body.fmt(f)
            },
            &DockerError::IOError(ref e) => e.description().fmt(f),
        }
    }
}

impl From<hyper::Error> for DockerError {
    fn from(err: hyper::Error) -> DockerError {
        DockerError::IOError(err)
    }
}


pub struct Docker {
    client: hyper::Client,
    endpoint: HumanURI
}

impl Docker {
    #[inline]
    pub fn new(raw_uri: &str) -> Docker {
        let endpoint = HumanURI::parse(raw_uri);
        Docker{
            client: hyper::Client::new(),
            endpoint: endpoint
        }
    }

    pub fn pull_image(&self, image: &spec::Image) -> Result<String, DockerError> {
        let url = self.endpoint.with_path("/images/create")
            .with_query_params([("fromImage", &image.name),
                                ("tag", &image.tag)].iter());
        debug!("POST {}...", url);
        let mut response = try!(self.client.post(url)
                                .header(header::Connection::close())
                                .send());
        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();
        if response.status != hyper::Ok {
            return Err(DockerError::HTTPError(
                format!("Error response, code: {}", response.status), body));
        }
        return Ok(body)
    }

}
