use std::{future::Future, io::Error, net::SocketAddr};

use tokio::io::{AsyncRead, AsyncWrite};

pub trait ConnOpenner {
    type Conn: IoAcceptor + IoOpenner + Send;

    fn open(&mut self, addr: SocketAddr) -> impl Future<Output = Result<Self::Conn, Error>> + Send;
}

pub trait ConnAcceptor {
    type Conn: IoAcceptor + IoOpenner + Send;

    fn accept(&mut self) -> impl Future<Output = Result<Self::Conn, Error>> + Send;
}

pub trait IoOpenner {
    type Io: AsyncRead + AsyncWrite + Send + Unpin;

    fn open(&mut self) -> impl Future<Output = Result<Self::Io, Error>> + Send;
}

pub trait IoAcceptor {
    type Io: AsyncRead + AsyncWrite + Send + Unpin;

    fn accept(&mut self) -> impl Future<Output = Result<Self::Io, Error>> + Send;
}
