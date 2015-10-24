#![deny(warnings)]
#![feature(collections)]
#![feature(convert)]

// packages
extern crate hyper;
// internal mods
mod consul;
mod human_uri;

// traits
use std::io::{Read,Write};
// std
use std::env;
use std::process::exit;
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
