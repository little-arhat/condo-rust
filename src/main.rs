#![deny(warnings)]

#![feature(collections)]
#![feature(convert)]
extern crate collections;

extern crate hyper;
extern crate url;

// traits
use std::io::{Read,Write};
use std::clone::Clone;
use std::borrow::Borrow;
use std::fmt;
use std::string::ToString;

// libs
use std::env;
use std::process::exit;

// ext libs
use hyper::client::Response;
use hyper::{header};

macro_rules! ignore{
    ( $( $x:expr ),* ) => {
        $(let _ = $x)*
    }
}

struct HumanURI {
    // TODO: store path components and all that separately
    url: hyper::Url
}

impl HumanURI {
    fn wrap(url: hyper::Url) -> Self {
        HumanURI{
            url: url
        }
    }

    fn parse(raw_uri: &str) -> Self {
        let url = if raw_uri.starts_with("http://") || raw_uri.starts_with("https://") {
            hyper::Url::parse(raw_uri)
        } else {
            hyper::Url::parse(&format!("http://{}", raw_uri))
        }.unwrap();
        Self::wrap(url)
    }

    // fn path_components(&self) -> Vec<&str> {
    //     self.url.path().unwrap().iter()
    //         .filter(|s| !s.is_empty())
    //         .map(|s| s.as_str())
    //         .collect::<Vec<_>>()
    // }

    fn with_query_params<'a, K, V, I>(&self, params: I) -> Self
        where I: Iterator<Item=&'a (K, V)>,
              K: 'a + AsRef<str>,
              V: 'a + ToString
    {
        let mut new_url = self.url.clone();
        // Save params to Vec, to keep created Strings for V there
        let sparams = params.map(|pair| {
            let ref k = pair.0;
            let ref v = pair.1;
            (k, v.to_string())
        }).collect::<Vec<_>>();
        // Pass references to passed K and created String for V to
        // setter
        new_url.set_query_from_pairs(sparams.iter()
                                            .map(|&(ref k, ref v)|
                                                 (k.as_ref(), v.as_str())));
        Self::wrap(new_url)
    }

    fn add_query_params<'a, K, V, I>(&self, params: I) -> Self
        where I: Iterator<Item=&'a (K, V)>,
              K: 'a + AsRef<str>,
              V: 'a + ToString
    {
        let mut new_url = self.url.clone();
        // Save params to Vec, to keep created Strings for V there
        let sparams = params.map(|pair| {
            let ref k = pair.0;
            let ref v = pair.1;
            (k, v.to_string())
        }).collect::<Vec<_>>();
        // Extract current query
        let current_query = match self.url.query_pairs() {
            Some(cq) => cq,
            None => vec!()
        };
        // Create Iter<&str, &str> from current query params
        let current_i = current_query.iter().map(|&(ref k, ref v)|
                                                 (k.as_str(), v.as_str()));
        // Create Iter from passed K and Strings created from V
        let sparams_i = sparams.iter().map(|&(ref k, ref v)|
                                           (k.as_ref(), v.as_str()));
        // Chain current query with receieved params
        let new_query = current_i.chain(sparams_i);
        new_url.set_query_from_pairs(new_query);
        Self::wrap(new_url)
    }

    /// Returns new uri, by appending path components
    fn with_path_components<E, I>(&self, paths: I) -> Self
        where I: Iterator<Item=E>,
              E: Borrow<str>
    {
        let mut new_url = self.url.clone();
        // Protect from "borrow of `new_url` occurs here"
        {
            // we will unwrap, because we want to use this for http urls only
            let mut path_components = new_url.path_mut().unwrap();
            path_components.clear();
            path_components.extend(paths.map(|s| s.borrow().to_string()));
        }
        Self::wrap(new_url)
    }

    fn add_path_components<E, I>(&self, paths: I) -> Self
        where I: Iterator<Item=E>,
              E: Borrow<str>
    {
        let mut new_url = self.url.clone();
        {
            // we will unwrap, because we want to use this for http urls only
            let mut path_components = new_url.path_mut().unwrap();
            path_components.extend(paths.map(|s| s.borrow().to_string()));
        }
        Self::wrap(new_url)
    }

    fn add_path(&self, path: &str) -> Self {
        self.add_path_components(path.trim_left_matches('/').split("/"))
    }

    fn with_path(&self, path: &str) -> Self {
        self.with_path_components(path.trim_left_matches('/').split("/"))
    }

}

impl hyper::client::IntoUrl for HumanURI {
    fn into_url(self) -> Result<hyper::Url, url::ParseError> {
        Ok(self.url)
    }
}

impl fmt::Display for HumanURI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.url.fmt(f)
    }
}

struct Consul {
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

    pub fn ping(&self) {
        println!("Will use consul on {}...", self.endpoint);
    }

    pub fn get_key(&self, key: &str, index: i32) -> hyper::Result<Response> {
        let url = self.endpoint
            .with_path("/v1/kv")
            .add_path(key)
            .with_query_params([("wait", "10s")].iter())
            .add_query_params([("index", index)].iter());
        println!("Get {}...", url);
        self.client.get(url)
            .header(header::Connection::close())
            .send()
    }
}

fn sleep(seconds: u64) {
    std::thread::sleep(std::time::Duration::new(seconds, 0));
}

fn print_usage(to: &mut Write) {
    let u = "Usage:\n\tCONSUL_AGENT=localhost:8500 ./condo consul/key\n";
    ignore!(writeln!(to, "{}", u));
}

fn error_and_usage(msg: &str) {
    let mut se = std::io::stderr();
    ignore!(writeln!(se, "{}", msg));
    print_usage(&mut se);
    exit(1);
}

fn env_var_or_exit(key: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => {
            error_and_usage(&format!("Provide {} environment variable", key));
            "".to_string() // not reached
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        return error_and_usage("No key provided!");
    }
    if args[1] == "--help" {
        print_usage(&mut std::io::stdout());
        return exit(0);
    }
    let ref key = args[1];
    println!("Will watch for {} key...", key);
    let consul:Consul = Consul::new(&env_var_or_exit("CONSUL_AGENT"));
    consul.ping();
    loop {
        match consul.get_key(key, 1) {
            Err(e) => println!("Error while requesting key {}: {}", key, e),
            Ok(mut res) => {
                let mut body = String::new();
                res.read_to_string(&mut body).unwrap();
                if res.status != hyper::Ok {
                    println!("HTTP Error: {}", res.status);
                } else {
                    println!("Response: {}", body);
                }
            }
        }
        sleep(5);
    }
}
