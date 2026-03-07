mod types;
mod resp;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    while match stream.read(&mut buffer) {
        Ok(n) if n > 0 => {
            let response = "+PONG\r\n";
            stream.write_all(response.as_bytes()).unwrap();
            true
        }
        _ => false,
    } {}
}

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:6379";
    let listener = TcpListener::bind(addr)?;
    println!("r > Listening in {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => {
                eprintln!("X > Error! {:?}", e);
            }
        }
    }

    println!("Hello, world!");

    Ok(())
}
