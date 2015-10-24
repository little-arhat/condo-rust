#![deny(warnings)]
#![feature(collections)]
#![feature(convert)]

// packages
extern crate hyper;
extern crate argparse;
extern crate collections;

// internal mods
mod consul;
mod human_uri;

// traits
use std::io::{Read};
// std
// external
// interal
use consul::Consul;


macro_rules! ignore{
    ( $( $x:expr ),* ) => {
        $(let _ = $x)*
    }
}

fn sleep(seconds: u64) {
    std::thread::sleep(std::time::Duration::new(seconds, 0));
}

fn main() {
    let mut consul_endpoint = "127.0.0.1:8500".to_string();
    let consul_env = "CONSUL_AGENT";
    let consul_help = format!("Address of consul agent to query; can \
be set via {} env var; default: {}", consul_env, consul_endpoint);
    let mut opt_consul_key:Option<String> = None;
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Condo: watch for consul key and \
run docker container.");
        ap.add_option(&["-V", "--version"],
                      argparse::Print(env!("CARGO_PKG_VERSION").to_string()),
                      "Show version");
        ap.refer(&mut consul_endpoint)
            .envvar("CONSUL_AGENT")
            .add_option(&["--consul"], argparse::Store,
                        &consul_help);
        ap.refer(&mut opt_consul_key)
            .add_argument("consul_key", argparse::StoreOption,
                          "Consul key to watch")
            .required();
        ap.parse_args_or_exit();
    }
    // opt_consul_key should not be None here, so unwrap safely
    let consul_key = opt_consul_key.unwrap();
    let consul:Consul = Consul::new(consul_endpoint.as_str());
    consul.ping();
    loop {
        match consul.get_key(&consul_key, 1) {
            Err(e) => println!("Error while requesting key {}: {}", consul_key, e),
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
