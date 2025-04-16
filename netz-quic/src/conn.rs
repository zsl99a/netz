use std::{
    io::{Error, ErrorKind},
    net::SocketAddr,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use netz_core::{
    codec::MessagePack,
    framed::{IoSerdeFramed, LengthDelimitedFramed},
    traits::{ConnAcceptor, ConnOpenner, IoAcceptor, IoOpenner},
};
use s2n_quic::{client::Connect, provider::tls, stream::BidirectionalStream, Client, Connection, Server};
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};

use crate::{CA_CERT_PEM, CLIENT_CERT_PEM, CLIENT_KEY_PEM, SERVER_CERT_PEM, SERVER_KEY_PEM};

/// QuicClient

#[derive(Debug, Clone)]
pub struct QuicClient {
    client: Client,
}

impl QuicClient {
    pub fn new(addr: SocketAddr) -> Result<Self, std::io::Error> {
        let tls = tls::default::Client::builder()
            .with_certificate(Path::new(CA_CERT_PEM))?
            .with_client_identity(Path::new(CLIENT_CERT_PEM), Path::new(CLIENT_KEY_PEM))?
            .build()?;

        let client = Client::builder()
            .with_tls(tls)
            .expect("Failed to create client.")
            .with_io(addr)?
            .start()
            .expect("Failed to start server.");

        Ok(Self { client })
    }
}

impl ConnOpenner for QuicClient {
    type Conn = QuicConnection;

    async fn open(&mut self, addr: SocketAddr) -> Result<Self::Conn, Error> {
        let mut conn = self.client.connect(Connect::new(addr).with_server_name("localhost")).await?;
        conn.keep_alive(true)?;
        Ok(QuicConnection::new(conn))
    }
}

/// QuicServer

pub struct QuicServer {
    server: Server,
}

impl QuicServer {
    pub fn new(addr: SocketAddr) -> Result<Self, std::io::Error> {
        let tls = tls::default::Server::builder()
            .with_trusted_certificate(Path::new(CA_CERT_PEM))?
            .with_certificate(Path::new(SERVER_CERT_PEM), Path::new(SERVER_KEY_PEM))?
            .with_client_authentication()?
            .build()?;

        let server = Server::builder()
            .with_tls(tls)
            .expect("Failed to create server.")
            .with_io(addr)?
            .start()
            .expect("Failed to start server.");

        Ok(Self { server })
    }
}

impl ConnAcceptor for QuicServer {
    type Conn = QuicConnection;

    async fn accept(&mut self) -> std::result::Result<Self::Conn, std::io::Error> {
        let conn = self
            .server
            .accept()
            .await
            .ok_or_else(|| Error::new(ErrorKind::Other, "accept connection failed"))?;

        Ok(QuicConnection::new(conn))
    }
}

/// QuicConnection

#[derive(Debug)]
pub struct QuicConnection {
    conn: Connection,
}

impl QuicConnection {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

impl IoOpenner for QuicConnection {
    type Io = QuicIo;

    async fn open(&mut self) -> Result<Self::Io, Error> {
        let io = self.conn.open_bidirectional_stream().await?;
        Ok(QuicIo::new(io))
    }
}

impl IoAcceptor for QuicConnection {
    type Io = QuicIo;

    async fn accept(&mut self) -> Result<Self::Io, Error> {
        let io = self
            .conn
            .accept_bidirectional_stream()
            .await?
            .ok_or_else(|| Error::new(ErrorKind::Other, "accept stream failed"))?;

        Ok(QuicIo::new(io))
    }
}

/// QuicIo

pub struct QuicIo {
    io: BidirectionalStream,
}

impl QuicIo {
    pub fn new(io: BidirectionalStream) -> Self {
        Self { io }
    }

    pub fn to_msgpack<O, I>(self) -> IoSerdeFramed<LengthDelimitedFramed<Self>, O, I, MessagePack<O, I>>
    where
        O: DeserializeOwned,
        I: Serialize,
    {
        IoSerdeFramed::new(LengthDelimitedFramed::new(self), MessagePack::<O, I>::default())
    }
}

impl AsyncRead for QuicIo {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<(), Error>> {
        self.io.read_buf(buf).boxed().poll_unpin(cx).map_ok(|_| {})
    }
}

impl AsyncWrite for QuicIo {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        self.io.write(buf).boxed().poll_unpin(cx)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.io.flush().boxed().poll_unpin(cx).map_err(|error| Error::new(ErrorKind::Other, error))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        self.io.close().boxed().poll_unpin(cx).map_err(|error| Error::new(ErrorKind::Other, error))
    }
}
