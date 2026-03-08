mod resp;
mod store;
mod types;

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use crate::resp::RespParser;
use crate::store::Store;
use crate::types::RespValue;

fn handle_client(stream: TcpStream, store: Arc<Store>) {
    let mut writer = stream.try_clone().unwrap();
    let mut parser = RespParser::new(stream);

    loop {
        let request = match parser.parse() {
            Ok(val) => val,
            Err(_) => break,
        };

        match request {
            RespValue::Array(args) => {
                let cmd_name = args[0].as_command().unwrap_or_default();

                match cmd_name.as_str() {
                    "PING" => writer.write_all(b"+PONG\r\n").unwrap(),
                    "GET" => {
                        if args.len() == 1 {
                            let response = RespValue::Error("No key provided".to_string());
                            writer.write_all(&response.serialize()).unwrap();
                            continue;
                        }
                        let key_name = args[1].as_command().unwrap_or_default();
                        let response = if let Some(value) = store.get(&key_name) {
                            RespValue::BulkString((*value).clone())
                        } else {
                            RespValue::Null
                        };
                        writer.write_all(&response.serialize()).unwrap();
                    },
                    "SET" => {
                        if args.len() >= 3 {
                            if let RespValue::BulkString(value_bytes) = &args[2] {
                                let key_name = args[1].as_command().unwrap_or_default();
                                store.set(key_name, value_bytes.clone());
                                writer.write_all(b"+OK\r\n").unwrap();
                            } 
                        }
                        else {
                            let response = RespValue::Error("ERR wrong number of arguments".to_string());
                            writer.write_all(&response.serialize()).unwrap();
                        }
                    }
                    _ => todo!(),
                }
            },
            _ => todo!(),
        }
    }
}

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:6379";
    let listener = TcpListener::bind(addr)?;
    println!("r > Listening in {}", addr);

    let store = Arc::new(Store::new());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store_cloned = Arc::clone(&store);
                thread::spawn(move || handle_client(stream, store_cloned));
            }
            Err(e) => {
                eprintln!("X > Error! {:?}", e);
            }
        }
    }

    println!("Hello, world!");

    Ok(())
}
