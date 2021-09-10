use {
    std::path::Path,

    clap,
};


const DEFAULT_CONFIG_FILE: &str = "/etc/tsplay.conf";


fn path_validator(path: String) -> Result<(), String> {
    let file = Path::new(&path);
    match file.exists() {
        true => Ok(()),
        false => Err(format!("cannot access '{}': No such file", path))
    }
}


fn main() {
    let args = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Play (stream) TS packets.")
        .arg(clap::Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("Enable debug"))
        .arg(clap::Arg::with_name("config")
            .takes_value(true)
            .value_name("CONFIG")
            .default_value(DEFAULT_CONFIG_FILE)
            .validator(path_validator)
            .help("config file")
        ).get_matches();

    dbg!(args);
}
