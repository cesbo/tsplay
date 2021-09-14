use serde::{
    self,
    Deserialize,
};


#[derive(Debug, Deserialize)]
pub struct Config {
    stream: Vec<Stream>,
}


#[derive(Debug, Deserialize)]
struct Stream {
    name: String,
    input: Type,
    output: Type,
}


#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Type {
    File { path: String },
    Udp { address: String, port: u16 }
}
