use {
    std::{
        pin::Pin,
        time::Duration,
    },

    tokio::{
        select,
        io::{
            AsyncReadExt,
            AsyncWriteExt,
        },
        time::sleep,
        signal::unix::{
            signal,
            SignalKind
        },
    },
    anyhow::Result,

    super::{
        config::{
            Type,
            Config,
            Stream,
            parse_config,
        },
        streams::{
            File,
            UdpStream,
            AsyncStream,
        },
    },
};


async fn syssig(kind: SignalKind) -> Result<()> {
    let mut stream = signal(kind)?;
    stream.recv().await;

    Ok(())
}


async fn make_stream(stream_type: &Type) -> Result<Pin<Box<dyn AsyncStream>>> {
    match stream_type {
        Type::File { path } => {
            Ok(Box::pin(File::open(&path).await?))
        },
        Type::Udp { address, port } => {
            Ok(Box::pin(UdpStream::new((address.as_str(), *port)).await?))
        },
    }
}


async fn play(stream: &Stream) -> Result<()> {
    let mut input = make_stream(&stream.input).await?;
    let mut output = make_stream(&stream.output).await?;

    let mut buf = vec![0; 4096];
    let mut input_offset = 0;

    loop {
        let offset = input.read(&mut buf).await.unwrap();

        input_offset += offset;
        dbg!(&input_offset);

        if output.write(&buf[ .. offset]).await.is_err() {
            continue
        }

        sleep(Duration::from_millis(50)).await;
    }
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
                _ = syssig(SignalKind::hangup()) => {
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
                _ = syssig(SignalKind::terminate()) => {
                    break;
                },
                _ = syssig(SignalKind::interrupt()) => {
                    break;
                },
                _ = play(&self.config.stream[0]) => {},
            }
        }
    }
}
