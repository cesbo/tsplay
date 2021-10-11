use {
    tokio::{
        select,
        fs::File,
        io::{
            AsyncReadExt,
            AsyncWriteExt,
        },
        signal::unix::{
            signal,
            SignalKind
        },
    },
    anyhow::Result,

    super::{
        udp::UdpStream,
        config::{
            Type,
            Config,
            Stream,
            parse_config,
        },
    }
};


async fn signal_coroutine(kind: SignalKind) -> Result<()> {
    let mut stream = signal(kind)?;
    stream.recv().await;

    Ok(())
}


pub async fn play(stream: &Stream) -> Result<()> {
    match stream {
        Stream { name: _, input: Type::File { path }, output: Type::Udp { address, port } } => {
            let mut buf = vec![0; 4096];

            let mut input = File::open(&path).await.unwrap();
            let mut input_offset = 0;

            let mut output = UdpStream::new((address.as_str(), *port)).await.unwrap();

            loop {
                let offset = input.read(&mut buf).await.unwrap();

                input_offset += offset;
                dbg!(&input_offset);

                if let Err(_) = output.write(&buf[ .. offset]).await {
                    continue
                }
            }
        },
        _ => {}
    }

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
                },
                _ = play(&self.config.stream[0]) => {},
            }
        }
    }
}
