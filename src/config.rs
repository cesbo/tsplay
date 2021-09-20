use {
    serde::{
        self,
        Deserialize,
    },
    anyhow::{
        Result,
        Context,
    },
    tokio_uring::fs::File,
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


pub async fn parse_config(path: &str) -> Result<Config> {
    let file = File::open(&path).await
        .with_context(|| format!("Failed to open configuration file \"{}\"", &path))?;

    let buf = vec![0; 4096];
    let (res, buf) = file.read_at(buf, 0).await;
    let offset = res
        .with_context(|| format!("Failed to read configuration file \"{}\"", &path))?;

    let config: Config = serde_json::from_slice(&buf[ .. offset])
        .with_context(|| format!("Failed to parse configuration file \"{}\"", &path))?;

    file.close().await
        .with_context(|| format!("Failed to close configuration file \"{}\"", &path))?;

    Ok(config)
}
