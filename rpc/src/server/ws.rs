use super::Peer;
use super::Error;
use tokio::net::TcpListener;
use tokio::net::ToSocketAddrs;

pub async fn run(peer: impl Peer, address: impl ToSocketAddrs) -> Result<(), Error> {
    let listener = TcpListener::bind(address).await?;

    loop {
        let (stream, address) = listener.accept().await?;
        tokio::spawn(async move {

        });

    }
}