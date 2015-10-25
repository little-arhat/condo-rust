
// packages
extern crate hyper;

// ext libs
use hyper::{header};
use hyper::client::Response;
// internal
use human_uri::HumanURI;

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
}
