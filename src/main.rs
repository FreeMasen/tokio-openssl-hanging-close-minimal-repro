use std::pin::Pin;

use openssl::{
    pkey::PKey,
    ssl::{Ssl, SslAcceptor, SslConnector, SslMethod, SslVerifyMode},
    x509::X509,
};
use std::net::Ipv4Addr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpSocket,
};
use tokio_openssl::SslStream;

const CERT_BYTES: &[u8] = include_bytes!("../test.crt");
const KEY_BYTES: &[u8] = include_bytes!("../test.key");

#[tokio::main]
async fn main() {
    let port = server().await;
    client(port).await;
}

async fn client(port: u16) {
    let client = TcpSocket::new_v4().unwrap();
    let stream = client
        .connect((Ipv4Addr::UNSPECIFIED, port).into())
        .await
        .unwrap();
    let connector = build_connector();
    let ssl = Ssl::new(connector.context()).unwrap();
    let mut wrapped = SslStream::new(ssl, stream).unwrap();
    Pin::new(&mut wrapped).connect().await.unwrap();
    wrapped.write("Some bytes".as_bytes()).await.unwrap();
    read("CLIENT", &mut wrapped).await;
    read("CLIENT", &mut wrapped).await;
}

async fn server() -> u16 {
    let listener = TcpSocket::new_v4().unwrap();
    listener.bind((Ipv4Addr::UNSPECIFIED, 0).into()).unwrap();
    let addr = listener.local_addr().unwrap();
    let listener = listener.listen(0).unwrap();
    tokio::task::spawn(async move {
        let (stream, _addr) = listener.accept().await.unwrap();
        let acceptor = build_acceptor();
        let ssl = Ssl::new(acceptor.context()).unwrap();
        let mut wrapped = SslStream::new(ssl, stream).unwrap();
        Pin::new(&mut wrapped).accept().await.unwrap();
        read("SERVER", &mut wrapped).await;
        wrapped.write("Some reply".as_bytes()).await.unwrap();
        // the server is disconnecting but not cleanly (similar to loosing network access mid-stream)
        std::mem::forget(wrapped);
        return;
    });
    addr.port()
}

async fn read<A>(who: &str, reader: &mut A)
where
    A: AsyncReadExt + Unpin,
{
    let mut buf = vec![0u8; 4096];
    let len = reader.read(&mut buf).await.unwrap();
    println!("{who}");
    println!("{:?}", String::from_utf8_lossy(&buf[..len]));
}

pub fn build_acceptor() -> SslAcceptor {
    let mut builder = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls_server()).unwrap();
    builder.set_verify(SslVerifyMode::NONE);
    let cert = X509::from_pem(CERT_BYTES).unwrap();
    let pkey = PKey::private_key_from_pem(KEY_BYTES).unwrap();
    builder.set_certificate(&cert).unwrap();
    builder.set_private_key(&pkey).unwrap();
    builder.build()
}

pub fn build_connector() -> SslConnector {
    let mut builder = SslConnector::builder(SslMethod::tls_client()).unwrap();
    builder.set_verify(SslVerifyMode::NONE);
    builder.build()
}
