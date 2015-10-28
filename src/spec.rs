
// ext libs
use serde_json;

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
    #[serde(skip_serializing_if_none)]
    tag: Option<None>
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
    #[serde(skip_serializing_if_none)]
    config: Option<serde_json::Value>
}

#[derive(Deserialize, Debug)]
pub enum Stop {
    Before,
    AfterTimeout(u16)
}

#[derive(Deserialize, Debug)]
pub struct Spec {
    image: Image,
    cmd: Vec<String>,
    services: Vec<Service>,
    envs: Vec<EnvVar>,
    #[serde(skip_serializing_if_none)]
    name: Option<String>,
    #[serde(skip_serializing_if_none)]
    host: Option<String>,
    #[serde(skip_serializing_if_none)]
    user: Option<String>,
    #[serde(default)]
    privileged: bool,
    #[serde(skip_serializing_if_none)]
    network_mode: Option<String>,
    #[serde(default)]
    stop: Stop,
    kill_timeout: Option<u16>,
    #[serde(skip_serializing_if_none)]
    log: Option<Log>
}
