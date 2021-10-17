use {
    std::{
        io,
        pin::Pin,
        net::Ipv4Addr,
        task::{
            Poll,
            Context,
        },
    },

    tokio::{
        io::{
            ReadBuf,
            AsyncRead,
            AsyncWrite,
        },
        net::{
            UdpSocket,
            ToSocketAddrs,
        },
    },

    super::AsyncStream,
};


pub struct UdpStream {
    inner: UdpSocket,
}

impl UdpStream {
    pub async fn new<A: ToSocketAddrs>(addr: A) -> anyhow::Result<Self> {
        let inner = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
        inner.connect(addr).await?;

        Ok(Self { inner })
    }
}

impl AsyncRead for UdpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.inner.poll_recv(cx, buf)
    }
}

impl AsyncWrite for UdpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.inner.poll_send(cx, buf)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl AsyncStream for UdpStream {}
