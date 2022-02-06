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
        mpeg::{
            pid,
            Pat,
            Pmt,
            StreamType,
        },
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


/// 90clocks = 1ms
pub const PTS_CLOCK_MS: u64 = 90;
pub const PTS_NONE: u64 = 1 << 33;
pub const PTS_MAX: u64 = PTS_NONE - 1;

/// Converts PTS to milliseconds
#[inline]
pub fn pts_to_ms(pts: u64) -> u64 { pts / PTS_CLOCK_MS }


/// Returns difference between previous PTS and current PTS
#[inline]
pub fn pts_delta(last_pts: u64, current_pts: u64) -> u64 {
    if current_pts >= last_pts {
        current_pts - last_pts
    } else {
        current_pts + PTS_MAX - last_pts
    }
}


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

    let mut buf = [0; 2048 * TS_PACKET_SIZE];

    let mut pmt_pids = Vec::new();
    let mut multimedia_pid = pid::NONE;

    let mut pts_first = 0;
    let mut pts_last = 0;

    loop {
        let mut r_offset = 0;
        let mut w_offset = 0;
        let mut c_offset = 0;

        let mut pts_vec = Vec::new();

        loop {
            let offset = input.read(&mut buf[r_offset .. ]).await.unwrap();
            r_offset += offset;
            if offset == 0 {
                break
            }
        }


        while c_offset < r_offset {
            match TsPacket::new(&buf[c_offset.. r_offset]) {
                Ok(ts) => {
                    c_offset += TS_PACKET_SIZE;
                    let ts_pid = ts.get_pid();

                    if multimedia_pid == pid::NONE {
                        if ts_pid == pid::PAT && pmt_pids.is_empty() {
                            let pat = Pat::new(&ts).unwrap();
                            pmt_pids.extend(
                                pat.items.iter().map(|item| item.program_map_pid)
                            );
                        }

                        if pmt_pids.contains(&ts_pid) {
                            let pmt = Pmt::new(&ts).unwrap();
                            for item in pmt.items {
                                if item.get_stream_type() == StreamType::Video {
                                    multimedia_pid = item.elementary_pid;
                                    pmt_pids.clear();
                                    break
                                }
                            }
                        }
                    }

                    if ts_pid != multimedia_pid || ! (ts.is_pusi() && ts.is_payload()) || ! ts.is_pes() {
                        continue
                    }

                    let pes = PesPacket::from(ts);
                    if ! (pes.is_syntax_spec() && pes.is_pts()) {
                        continue
                    }

                    if let Some(pts) = pes.get_pts() {
                        pts_vec.push(pts);

                        if pts_first == 0 {
                            pts_first = pts;
                            pts_last = pts;
                        }

                        if pts > pts_last {
                            pts_last = pts;
                            continue
                        }

                        if pts < pts_last {
                            let delta = pts_to_ms(pts_delta(pts_first, pts));
                            pts_first = pts_last;
                            pts_last = pts;

                            loop {
                                let (start, end) = offset_calc(w_offset, c_offset);
                                let diff = end - start;
                                if diff < 7 * TS_PACKET_SIZE {
                                    break
                                }
                                let offset = output.write(&mut buf[start .. end]).await.unwrap();
                                w_offset += offset;
                                if offset == 0 {
                                    break
                                }
                            }

                            sleep(Duration::from_millis(10)).await;
                        }
                    };
                }
                Err(_) => c_offset += 1
            }
        }
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
