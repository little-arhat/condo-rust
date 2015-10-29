
// ext libs
use serde_json;
use serde::{Deserialize, Deserializer, de};
// traits
use serde::de::Error;

#[derive(Deserialize, Debug)]
pub struct EnvVar {
    name: String,
    value: String
}

#[derive(Deserialize, Debug)]
pub struct Image {
    name: String,
    tag: String
}

#[derive(Deserialize, Debug)]
pub enum CheckMethod {
    Script(String),
    Http(String),
    HttpPath(String)
}

#[derive(Deserialize, Debug)]
pub struct Check {
    method: CheckMethod,
    interval: u16,
    timeout: u16
}

#[derive(Deserialize, Debug)]
pub struct Service {
    name: String,
    port: u16,
    tags: Vec<String>,
    check: Check,
    udp: bool,
    host_port: Option<u16>
}

#[derive(Deserialize, Debug)]
pub struct Discovery {
    service: String,
    env: String,
    #[serde(default)]
    multiple: bool,
    #[serde(skip_serializing_if_none, default)]
    tag: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct Volume {
    from: String,
    to: String
}

#[derive(Deserialize, Debug)]
pub struct Log {
    #[serde(rename="type")]
    log_type: String,
    #[serde(skip_serializing_if_none, default)]
    config: Option<serde_json::Value>
}


#[derive(Debug)]
pub enum Stop {
    Before,
    AfterTimeout(u16)
}

// ocaml's yojson Enum parsing
impl Deserialize for Stop {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        deserializer.visit(StopEnumVisitor)
    }
}

struct StopEnumVisitor;

impl de::Visitor for StopEnumVisitor {
    type Value = Stop;

    fn visit_seq<V>(&mut self, mut v: V) -> Result<Self::Value, V::Error>
        where V: de::SeqVisitor
    {
        let ret = match try!(v.visit::<String>()) {
            Some(ref b) if b == "Before" => Stop::Before,
            Some(ref at) if at == "AfterTimeout"  => {
                match try!(v.visit::<u16>()) {
                    Some(t) => Stop::AfterTimeout(t),
                    _ => return Err(V::Error::length_mismatch(0))
                }
            },
            Some(v) => return Err(V::Error::unknown_field(&v)),
            None => return Err(V::Error::length_mismatch(0))
        };
        try!(v.end());
        Ok(ret)
    }
}

impl Default for Stop {
    fn default() -> Stop { Stop::AfterTimeout(10) }
}

#[derive(Deserialize, Debug)]
pub struct Spec {
    image: Image,
    cmd: Vec<String>,
    services: Vec<Service>,
    envs: Vec<EnvVar>,
    #[serde(skip_serializing_if_none, default)]
    name: Option<String>,
    #[serde(skip_serializing_if_none, default)]
    host: Option<String>,
    #[serde(skip_serializing_if_none, default)]
    user: Option<String>,
    #[serde(default)]
    privileged: bool,
    #[serde(skip_serializing_if_none, default)]
    network_mode: Option<String>,
    #[serde(default)]
    stop: Stop,
    kill_timeout: Option<u16>,
    #[serde(skip_serializing_if_none, default)]
    log: Option<Log>
}
