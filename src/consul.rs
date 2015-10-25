
// packages
extern crate hyper;

// ext libs
use hyper::{header};
use hyper::client::Response;
// traits
use std::io::{Read};
// std
use std::sync::mpsc;
use std::thread;
// internal
use human_uri::HumanURI;
use utils::*;


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

    pub fn get_key<T:AsRef<str>>(&self, key: T, index: i32) -> hyper::Result<Response>
    {
        let url = self.endpoint
            .with_path("/v1/kv")
            .add_path(key)
            .with_query_params([("wait", "10s")].iter())
            .add_query_params([("index", index)].iter());
        info!("Get {}...", url);
        self.client.get(url)
            .header(header::Connection::close())
            .send()
    }

    pub fn watch_key<T:AsRef<str> + Send>(self, key: T) -> mpsc::Receiver<Result<String, String>> {
        let (tx, rx) = mpsc::channel();
        let thread_key = key.as_ref().to_owned();
        thread::spawn(move || {
            loop {
                match self.get_key(&thread_key, 0) {
                    Err(e) => {
                        ignore_result!(tx.send(Err(error_description(&e))));
                    },
                    Ok(mut res) => {
                        if res.status != hyper::Ok {
                            let e = Err(format!("HTTP Error: {}", res.status));
                            ignore_result!(tx.send(e));
                        } else {
                            let mut body = String::new();
                            res.read_to_string(&mut body).unwrap();
                            ignore_result!(tx.send(Ok(body)));
                        }
                    }
                };
                sleep(10);
            }
        });

        rx
    }
}
