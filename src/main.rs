mod config;
mod streams;
mod application;

use {
    anyhow::Result,

    application::Application,
};


const DEFAULT_CONFIG_FILE: &str = "/etc/tsplay.conf";


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let mut app = Application::new(&path).await?;
    app.run().await;

    Ok(())
}
