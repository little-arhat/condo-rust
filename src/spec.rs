
// ext libs
use serde_json;
use serde::{Deserialize, Deserializer, de};
// traits
use serde::de::Error;

#[derive(Deserialize, Debug, Clone)]
pub struct EnvVar {
    pub name: String,
    pub value: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Image {
    pub name: String,
    pub tag: String
}

#[derive(Debug, Clone)]
pub enum CheckMethod {
    Script(String),
    Http(String),
    HttpPath(String)
}

impl Deserialize for CheckMethod {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        deserializer.visit(CheckMethodEnumVisitor)
    }
}

struct CheckMethodEnumVisitor;

impl de::Visitor for CheckMethodEnumVisitor {
    type Value = CheckMethod;

    fn visit_seq<V>(&mut self, mut v: V) -> Result<Self::Value, V::Error>
        where V: de::SeqVisitor
    {
        let variants = vec!("Script", "Http", "HttpPath");
        let ret = match try!(v.visit::<String>()) {
            Some(ref m) if variants.contains(&m.as_str()) => {
                match try!(v.visit::<String>()) {
                    Some(s) => {
                        match m.as_str() {
                            "Script" => CheckMethod::Script(s),
                            "Http" => CheckMethod::Http(s),
                            "HttpPath" => CheckMethod::HttpPath(s),
                            _ => panic!("can't happen")
                        }
                    },
                    None => return Err(V::Error::type_mismatch(de::Type::String))
                }
            },
            Some(v) => return Err(V::Error::unknown_field(&v)),
            None => return Err(V::Error::length_mismatch(0))
        };
        try!(v.end());
        Ok(ret)
    }
}

impl Default for CheckMethod {
    fn default() -> CheckMethod { CheckMethod::Script("echo".to_string()) }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Check {
    pub method: CheckMethod,
    pub interval: u16,
    pub timeout: u16
}

#[derive(Deserialize, Debug, Clone)]
pub struct Service {
    pub name: String,
    pub port: u16,
    pub tags: Vec<String>,
    pub check: Check,
    #[serde(default)]
    pub udp: bool,
    pub host_port: Option<u16>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Discovery {
    pub service: String,
    pub env: String,
    #[serde(default)]
    pub multiple: bool,
    #[serde(skip_serializing_if_none, default)]
    pub tag: Option<String>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Volume {
    pub from: String,
    pub to: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Log {
    #[serde(rename="type")]
    pub log_type: String,
    #[serde(skip_serializing_if_none, default)]
    pub config: Option<serde_json::Value>
}


#[derive(Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
pub struct Spec {
    pub image: Image,
    pub cmd: Vec<String>,
    pub services: Vec<Service>,
    pub envs: Vec<EnvVar>,
    pub discoveries: Vec<Discovery>,
    #[serde(skip_serializing_if_none, default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if_none, default)]
    pub host: Option<String>,
    #[serde(skip_serializing_if_none, default)]
    pub user: Option<String>,
    #[serde(default)]
    pub privileged: bool,
    #[serde(skip_serializing_if_none, default)]
    pub network_mode: Option<String>,
    #[serde(default)]
    pub stop: Stop,
    pub kill_timeout: Option<u16>,
    #[serde(skip_serializing_if_none, default)]
    pub log: Option<Log>
}
