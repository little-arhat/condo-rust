#![deny(warnings)]
#![feature(collections)]
#![feature(convert)]

// packages
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate collections;
// ext
extern crate hyper;
extern crate argparse;

// internal mods
mod consul;
mod human_uri;

// traits
use std::io::{Read};
use std::error::Error;
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

fn error_description(e: &Error) -> String {
    match e.cause() {
        Some(inner) => format!("{}", inner),
        None => format!("{}", e)
    }
}

fn initialize_logging(level: log::LogLevelFilter) {
    use log4rs::{config,appender};
    let root = config::Root::builder(level).appender("stderr".to_string());
    let console = Box::new(appender::ConsoleAppender::builder().build());
    let config = config::Config::builder(root.build())
        .appender(config::Appender::builder("stderr".to_string(),
                                            console).build());
    log4rs::init_config(config.build().unwrap()).unwrap();
}

fn main() {
    let mut consul_endpoint = "127.0.0.1:8500".to_string();
    let consul_env = "CONSUL_AGENT";
    let consul_help = format!("Address of consul agent to query; can \
be set via {} env var; default: {}", consul_env, consul_endpoint);
    let mut opt_consul_key:Option<String> = None;
    let mut log_level = log::LogLevelFilter::Info;
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
        ap.refer(&mut log_level)
            .envvar("CONDO_LOG_LEVEL")
            .add_option(&["--loglevel"], argparse::Store,
                        "Set log level");
        ap.parse_args_or_exit();
    }
    initialize_logging(log_level);
    // opt_consul_key should not be None here, so unwrap safely
    let consul_key = opt_consul_key.unwrap();
    info!("Will watch for consul key: {}", consul_key);
    let consul:Consul = Consul::new(&consul_endpoint);
    loop {
        match consul.get_key(&consul_key, 1) {
            Err(e) => error!("Error while requesting key {}: {}", consul_key, error_description(&e)),
            Ok(mut res) => {
                let mut body = String::new();
                res.read_to_string(&mut body).unwrap();
                if res.status != hyper::Ok {
                    error!("HTTP Error: {}", res.status);
                } else {
                    info!("Response: {}", body);
                }
            }
        }
        sleep(10);
    }
}
