use prost::Message;
use proto::Client;
use std::env;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufStream};
use tokio::net::{TcpListener, TcpStream};
mod proto {
    tonic::include_proto!("client");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let argc = args.len();
    if argc == 1 {
        server().await?;
        return Ok(());
    }
    if args[1] != String::from("client") {
        panic!("Incorrect Argument!");
    }
    if argc != 4 {
        panic!("Incorrect Num of Args!");
    }

    let addr = match Ipv4Addr::from_str(&args[2]) {
        Ok(addr) => addr,
        Err(_) => panic!("Incorrect Address Format!"),
    };
    client(addr, &args[3]).await?;
    Ok(())
}

async fn server() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8081").await?;
    println!("Server ip: {}", listener.local_addr()?);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let buf_stream = BufStream::new(stream);
                tokio::task::spawn(async move { handle_client_stream(buf_stream).await })
            }
            Err(_) => continue,
        };
    }
}

async fn handle_client_stream(mut stream: BufStream<TcpStream>) -> std::io::Result<()> {
    loop {
        let mut buf = bytes::BytesMut::new();
        stream.read_buf(&mut buf).await?;
        let new_client = Client::decode_length_delimited(buf)?;
        println!("{} as {}:", new_client.ip_addr, new_client.username);
        println!("{}", new_client.payload);
    }
}

async fn client(addr: Ipv4Addr, port: &String) -> std::io::Result<()> {
    let port_num = match port.parse::<u16>() {
        Ok(port_num) => port_num,
        Err(_) => 0,
    };
    let mut username = String::new();
    println!("Please enter username: ");
    std::io::stdin().read_line(&mut username)?;
    let stream = TcpStream::connect(SocketAddr::new(IpAddr::V4(addr), port_num)).await?;

    println!("Connected to: {}", stream.peer_addr()?);
    let ip_addr = stream.local_addr()?.to_string();

    let mut payload = String::new();
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut buf = bytes::BytesMut::new();
    let mut buf_stream = BufStream::new(stream);
    loop {
        reader.read_line(&mut payload).await?;

        if payload.trim_end() == "/exit" {
            break;
        }
        Client {
            ip_addr: ip_addr.clone(),
            username: username.trim_end().to_string(),
            payload: payload.trim_end().to_string(),
        }
        .encode_length_delimited(&mut buf)?;
        buf_stream.write_buf(&mut buf).await?;
        buf_stream.flush().await?;
        buf.clear();
        payload.clear();
        continue;
    }
    Ok(())
}
