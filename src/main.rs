mod config;

use {
    std::{
        fs,
        path::Path,
    },

    anyhow::{
        Result,
        Context,
    },

    config::Config,
};


const DEFAULT_CONFIG_FILE: &str = "/etc/tsplay.conf";


fn path_validator(path: String) -> Result<(), String> {
    let file = Path::new(&path);
    match file.exists() {
        true => Ok(()),
        false => Err(format!("cannot access \"{}\": No such file", path))
    }
}


fn main() -> Result<()> {
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
            .validator(path_validator)
            .help("configuration file")
        ).get_matches();

    // Unwrap use, because there is a default value and a validator for the config argument.
    let path = args.value_of("config").unwrap();
    let data = fs::read(&path)
        .with_context(|| format!("Failed to read configuration file \"{}\"", &path))?;
    let config: Config = serde_json::from_slice(&data)
        .with_context(|| format!("Failed to parse configuration file \"{}\"", &path))?;

    dbg!(&config);

    Ok(())
}
