use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};

use anyhow::Result;
use futures::future;
use netz_core::traits::{ConnAcceptor, ConnOpenner, IoAcceptor, IoOpenner};
use netz_quic::{QuicClient, QuicServer};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_line_number(true).init();

    let client = QuicClient::new(SocketAddr::from(([0, 0, 0, 0], 0)))?;
    let mut server = QuicServer::new(SocketAddr::from(([0, 0, 0, 0], 12345)))?;

    let peers = Arc::new(Mutex::new(BTreeMap::<SocketAddr, String>::new()));

    {
        let peers = peers.clone();

        tokio::spawn(async move {
            while let Ok(mut conn) = server.accept().await {
                let peers = peers.clone();

                tokio::spawn(async move {
                    let nz = conn.open().await?;

                    while let Ok(mut stream) = conn.accept().await {
                        tokio::spawn(async move {});
                    }

                    Result::<()>::Ok(())
                });
            }
        });
    }

    tokio::spawn(async move {
        todo!();
    });

    future::pending::<Result<()>>().await
}
