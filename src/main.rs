#![feature(collections)]
#![feature(convert)]
extern crate collections;

extern crate hyper;

// traits
use std::io::{Read,Write};
use std::clone::Clone;
use std::fmt::Display;
use std::string::ToString;

// libs
use std::env;
use std::process::exit;

// ext libs
use hyper::client::Response;
use hyper::{header, Url};

macro_rules! ignore{
    ( $( $x:expr ),* ) => {
        $(let _ = $x)*
    }
}

trait HumanURI:hyper::client::IntoUrl + Sized + Display {
    fn parse(&str) -> Self;
    // TODO: https://github.com/rust-lang/rfcs/pull/1305
    fn path_components(&self) -> Vec<&str>;
    fn with_path_components<E, I>(&self, I) -> Self
        where I: Iterator<Item=E>,
              E: ToString;
    // fn with_query<(&self, )
    // defaults
    fn full_path(&self) -> String {
        let p = self.path_components().join("/");
        if p.starts_with("/") {
            p
        } else {
            format!("/{}", p)
        }
    }

    fn add_path_components<E, I>(&self, pc: I) -> Self
        where I: Iterator<Item=E>,
              E: ToString
    {
        // XXX: horrible! find a way to do this better!
        // XXX: problems: can't .to_string().as_str() due to lifetimes
        let pcs = pc.map(|s| s.to_string()).collect::<Vec<_>>();
        let mut tmp = self.path_components();
        tmp.extend(pcs.iter().map(|s| s.as_str()));
        self.with_path_components(tmp.iter())
    }

    fn add_path(&self, path: &str) -> Self {
        self.add_path_components(path.trim_left_matches('/').split("/"))
    }

    fn with_path(&self, path: &str) -> Self {
        self.with_path_components(path.trim_left_matches('/').split("/"))
    }
}

impl HumanURI for Url {
    fn parse(raw_uri: &str) -> Self {
        if raw_uri.starts_with("http://") || raw_uri.starts_with("https://") {
            Url::parse(raw_uri)
        } else {
            Url::parse(&format!("http://{}", raw_uri))
        }.unwrap()
    }

    fn path_components(&self) -> Vec<&str> {
        self.path().unwrap().iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
    }

    /// Returns new uri, by appending path components
    fn with_path_components<E, I>(&self, paths: I) -> Self
        where I: Iterator<Item=E>,
              E: ToString
    {
        let mut new_url = self.clone();
        // Protect from "borrow of `new_url` occurs here"
        {
            // we will unwrap, because we want to use this for http urls only
            let mut path_components = new_url.path_mut().unwrap();
            path_components.clear();
            path_components.extend(paths.map(|s| s.to_string()));
        }
        new_url
    }
}

struct Consul<T:HumanURI> {
    client: hyper::Client,
    endpoint: T
}

impl <T:HumanURI> Consul<T> {
    #[inline]
    pub fn new(raw_uri: &str) -> Consul<T> {
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
        let query = if index > 0 {
            format!("?wait=10s&index={}", index)
        } else {
            "".to_string()
        };
        let url = self.endpoint.with_path("/v1/kv").add_path(key);
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
    let consul:Consul<Url> = Consul::new(&env_var_or_exit("CONSUL_AGENT"));
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
