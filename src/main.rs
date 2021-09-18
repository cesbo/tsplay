mod config;

use {
    anyhow::{
        Result,
        Context,
    },
    tokio_uring::fs::File,

    config::Config,
};


const DEFAULT_CONFIG_FILE: &str = "/etc/tsplay.conf";


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Play (stream) TS packets.")
        .arg(clap::Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("enable debug"))
        .arg(clap::Arg::with_name("config")
            .takes_value(true)
            .value_name("CONFIG")
            .default_value(DEFAULT_CONFIG_FILE)
            .help("configuration file")
        ).get_matches();

    // Unwrap use, because there is a default value and a validator for the config argument.
    let path = args.value_of("config").unwrap();

    tokio_uring::start(async {
        let file = File::open(&path).await
            .with_context(|| format!("Failed to open configuration file \"{}\"", &path))?;

        let buf = vec![0; 4096];
        let (res, buf) = file.read_at(buf, 0).await;
        let offset = res
            .with_context(|| format!("Failed to read configuration file \"{}\"", &path))?;

        let config: Config = serde_json::from_slice(&buf[ .. offset])
            .with_context(|| format!("Failed to parse configuration file \"{}\"", &path))?;

        dbg!(&config);

        Ok(())
    })
}
