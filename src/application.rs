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
        ts::{
            TsPacket,
            TS_PACKET_SIZE,
        },
        es::PesPacket,
        misc::offset_calc,
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

    let mut buf = [0; 1024 * TS_PACKET_SIZE];
    let mut ts_cnt = 0;

    loop {
        let mut r_offset = 0;
        loop {
            let offset = input.read(&mut buf[r_offset.. ]).await.unwrap();
            r_offset += offset;
            if offset == 0 {
                break
            }
        }

        let mut pts_vec = Vec::new();

        let mut cnt = 0;
        while cnt < r_offset {
            match TsPacket::new(&buf[cnt ..r_offset]) {
                Ok(ts) => {
                    ts_cnt += 1;
                    cnt += TS_PACKET_SIZE;
                    if ! (ts.is_pusi() & ts.is_payload()) {
                        continue
                    }

                    if ts.is_pes() {
                        let pes = PesPacket::from(ts);
                        if pes.is_syntax_spec() && pes.is_pts() {
                            if let Some(pts) = pes.get_pts() {
                                pts_vec.push(pts);
                            };
                        }
                    }
                }
                Err(_) => cnt += 1
            }
        }

        dbg!(&pts_vec);

        let mut w_offset = 0;
        loop {
            let (start, end) = offset_calc(w_offset, r_offset);
            let offset = output.write(&mut buf[start .. end]).await.unwrap();
            w_offset += offset;
            if offset == 0 {
                break
            }
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
