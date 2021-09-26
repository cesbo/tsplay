use {
    tokio::{
        select,
        signal::unix::{
            signal,
            SignalKind
        },
    },
    anyhow::Result,

    super::config::{
        Config,
        parse_config,
    }
};


async fn signal_coroutine(kind: SignalKind) -> Result<()> {
    let mut stream = signal(kind)?;
    stream.recv().await;

    Ok(())
}


pub struct Application {
    pub config: Config,
    config_path: String,
}

impl Application {
    pub async fn new<S: ToString>(path: S) -> Result<Self> {
        let config_path = path.to_string();
        let res = Self {
            config: parse_config(&config_path).await?,
            config_path,
        };

        Ok(res)
    }

    pub async fn run(&mut self) {
        loop {
            select! {
                _ = signal_coroutine(SignalKind::hangup()) => {
                    match parse_config(&self.config_path).await {
                        Ok(config) => {
                            dbg!(&config);
                            self.config = config;
                        },
                        Err(err) => {
                            dbg!(err);
                        }
                    }
                },
                _ = signal_coroutine(SignalKind::terminate()) => {
                    break;
                }
            }
        }
    }
}
