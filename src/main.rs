#![deny(warnings)]
#![feature(collections)]
#![feature(convert)]

// packages
#[macro_use] extern crate log;
extern crate log4rs;
extern crate collections;
// ext
#[macro_use] extern crate hyper;
extern crate argparse;
extern crate serde;
extern crate serde_json;

// internal mods
#[macro_use] mod utils;
mod human_uri;
mod consul;
mod condo;

// traits
// std
// external
// interal



fn initialize_logging(level: log::LogLevelFilter) {
    use log4rs::{config,appender};
    let root = config::Root::builder(log::LogLevelFilter::Error)
        .appender("stderr".to_string());
    let logger = config::Logger::builder("condo_r".to_string(), level)
        .additive(true);
    let console = Box::new(appender::ConsoleAppender::builder().build());
    let config = config::Config::builder(root.build())
        .appender(config::Appender::builder("stderr".to_string(),
                                            console).build())
        .logger(logger.build());
    log4rs::init_config(config.build().unwrap()).unwrap();
}

fn main() {
    let mut consul_endpoint = "127.0.0.1:8500".to_string();
    let consul_env = "CONSUL_AGENT";
    let consul_help = format!("Address of consul agent to query; can \
be set via {} env var; default: {}", consul_env, consul_endpoint);
    let mut opt_consul_key:Option<String> = None;
    let mut log_level = log::LogLevelFilter::Debug;
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
    let consul = consul::Consul::new(&consul_endpoint);
    let specs = consul.watch_key(&consul_key);
    let condo = condo::Condo::new(specs);
    let handle = condo.start();
    let r = handle.join();
    match r {
        Err(_) => error!("Error!"),
        Ok(_) => info!("Done!")
    }
}
