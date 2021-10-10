use {
    serde::{
        self,
        Deserialize,
    },
    anyhow::{
        Result,
        Context,
    },
    tokio::{
        fs::File,
        io::AsyncReadExt,
    },
};


#[derive(Debug, Deserialize)]
pub struct Config {
    pub stream: Vec<Stream>,
}


#[derive(Debug, Deserialize)]
pub struct Stream {
    pub name: String,
    pub input: Type,
    pub output: Type,
}


#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Type {
    File { path: String },
    Udp { address: String, port: u16 }
}


pub async fn parse_config(path: &str) -> Result<Config> {
    let mut file = File::open(&path).await
        .with_context(|| format!("Failed to open configuration file \"{}\"", &path))?;

    let mut buf = vec![];
    file.read_to_end(&mut buf).await
        .with_context(|| format!("Failed to read configuration file \"{}\"", &path))?;

    let config: Config = serde_json::from_slice(&buf)
        .with_context(|| format!("Failed to parse configuration file \"{}\"", &path))?;

    Ok(config)
}
