use {
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    }
};

mod udp;
pub use udp::UdpStream;

mod file;
pub use tokio::fs::File;


pub trait AsyncStream: AsyncRead + AsyncWrite {}
