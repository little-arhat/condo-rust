#![feature(collections)]
extern crate collections;

extern crate hyper;

// traits
use std::io::{Read,Write};
use std::clone::Clone;
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


struct Consul {
    client: hyper::Client,
    endpoint: Url
}



fn url_parse(raw_uri: &str) -> Url {
    if raw_uri.starts_with("http://") || raw_uri.starts_with("https://") {
        Url::parse(raw_uri)
    } else {
        Url::parse(&("http://".to_string() + raw_uri))
    }.unwrap()
}

fn url_append_paths<E:ToString, I:Iterator<Item=E>>(url: &Url, paths: I) -> Url
{
    let mut new_url = url.clone();
    // Protect from "borrow of `new_url` occurs here"
    {
        // we will unwrap, because we want to use this for http urls only
        let mut path_components = new_url.path_mut().unwrap();
        // Empty path encoded as [""]
        if path_components.len() == 1 && &path_components[0] == "" {
            ignore!(path_components.pop());
        }
        path_components.extend(paths.map(|s| s.to_string()));
    }
    new_url
}

impl Consul {
    #[inline]
    pub fn new(raw_uri: &str) -> Consul {
        let endpoint = url_parse(raw_uri);
        let endpoint = url_append_paths(&endpoint, ["trulalala"].iter());
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
        let url = format!("{}/v1/kv/{}{}", self.endpoint, key, query);
        self.client.get(&url)
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
    let consul = Consul::new(&env_var_or_exit("CONSUL_AGENT"));
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
