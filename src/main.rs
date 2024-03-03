mod protos;
use protobuf::CodedInputStream;
use protobuf::CodedOutputStream;
use protobuf::Message;
use protobuf::SpecialFields;
use protos::protos::client;
use std::env;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    let argc = args.len();
    if argc == 1 {
        match server() {
            Ok(_) => return Ok(()),
            Err(_) => return Err("Binding Error!"),
        };
    }
    if args[1] != String::from("client") {
        return Err("Incorrect Argument!");
    }
    if argc != 4 {
        return Err("Incorrect Num of Args!");
    }

    let addr = match Ipv4Addr::from_str(&args[2]) {
        Ok(addr) => addr,
        Err(_) => return Err("Incorrect Address Format!"),
    };
    match client(addr, &args[3]) {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}

fn server() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8081")?;
    listener.set_nonblocking(true)?;
    println!("Server ip: {}", listener.local_addr()?);

    let (tx, rx) = mpsc::channel::<client::Client>();
    let (stream_sender, stream_receiver) = mpsc::channel::<TcpStream>();
    thread::spawn(move || {
        let stream_sender_1 = stream_sender.clone();
        for stream_result in listener.incoming() {
            let tx1 = tx.clone();
            match stream_result {
                Ok(stream) => {
                    let mut stream_clone = stream.try_clone().unwrap();
                    thread::spawn(move || match handle_client_stream(&mut stream_clone, tx1) {
                        Ok(_) => (),
                        Err(_) => (),
                    });
                    match stream_sender_1.send(stream) {
                        Ok(_) => (),
                        Err(_) => (),
                    };
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(_) => continue,
            }
        }
    });
    loop {
        match rx.recv() {
            Ok(received_client) => {
                for mut stream in stream_receiver.iter() {
                    let mut output_stream = CodedOutputStream::new(&mut stream);
                    received_client.write_length_delimited_to(&mut output_stream)?;
                    output_stream.flush()?;
                }
            }
            Err(_) => continue,
        };
    }
}

fn handle_client_stream(
    mut stream: &TcpStream,
    tx: mpsc::Sender<client::Client>,
) -> std::io::Result<()> {
    loop {
        let mut input_stream = CodedInputStream::new(&mut stream);
        let new_client: client::Client = match input_stream.read_message() {
            Ok(new_client) => new_client,
            Err(_) => return Ok(()),
        };
        println!("{} as {}:", new_client.ip_addr, new_client.username);
        println!("{}", new_client.payload);
        match tx.send(new_client) {
            Ok(_) => (),
            Err(_) => (),
        };
    }
}

fn client(addr: Ipv4Addr, port: &String) -> std::io::Result<()> {
    let port_num = match port.parse::<u16>() {
        Ok(port_num) => port_num,
        Err(_) => 0,
    };
    let mut username = String::new();
    println!("Please enter username: ");
    std::io::stdin().read_line(&mut username)?;
    let mut stream = TcpStream::connect(SocketAddr::new(IpAddr::V4(addr), port_num))?;
    let mut stream_clone = stream.try_clone()?;
    let username_clone = username.clone();

    println!("Connected to: {}", stream.peer_addr()?);
    let ip_addr = stream.local_addr()?.to_string();

    let mut output_stream = CodedOutputStream::new(&mut stream);
    thread::spawn(move || {
        let mut input_stream = CodedInputStream::new(&mut stream_clone);
        loop {
            let new_client: client::Client = match input_stream.read_message() {
                Ok(new_client) => new_client,
                Err(_) => continue,
            };
            if new_client.username == username_clone.trim_end() {
                continue;
            }
            println!("{} as {}:", new_client.ip_addr, new_client.username);
            println!("{}", new_client.payload);
        }
    });

    let mut out_payload = String::new();
    loop {
        std::io::stdin().read_line(&mut out_payload)?;
        if out_payload.trim_end() == "/exit" {
            break;
        }
        client::Client {
            ip_addr: ip_addr.to_string(),
            username: username.trim_end().to_string(),
            payload: out_payload.trim_end().to_string(),
            special_fields: SpecialFields::new(),
        }
        .write_length_delimited_to(&mut output_stream)?;
        output_stream.flush()?;
        out_payload.clear();
    }
    Ok(())
}
