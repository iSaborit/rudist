mod resp;
mod store;
mod types;

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use crate::resp::{RespError, RespParser};
use crate::store::Store;
use crate::types::RespValue;

fn handle_client(stream: TcpStream, store: Arc<Store>) {
    let mut writer = stream.try_clone().unwrap();
    let mut parser = RespParser::new(stream);

    loop {
        let request = match parser.parse() {
            Ok(val) => val,
            Err(e) => match e {
                RespError::Protocol(e) => {
                    let _ = writer.write_all(format!("-ERR Protocol error: {e}\r\n").as_bytes());
                    return;
                }
                RespError::Io(e) => {
                    eprintln!("{e}");
                    return;
                }
                RespError::Eof => return,
            },
        };

        match request {
            RespValue::Array(args) => {
                if args.is_empty() {
                    let _ = writer.write_all(b"-ERR empty array\r\n");
                    continue;
                }
                let cmd_name = args[0].as_command().unwrap_or_default();

                match cmd_name.as_str() {
                    "PING" => {
                        let _ = writer.write_all(b"+PONG\r\n");
                        continue;
                    }
                    "GET" => {
                        let response: RespValue = if args.len() == 1 {
                            RespValue::Error(
                                "ERR wrong number of arguments for 'get' command".to_string(),
                            )
                        } else if args.len() == 2 {
                            let key_name = args[1].as_key().unwrap_or_default();
                            if let Some(value) = store.get(&key_name) {
                                RespValue::BulkString((*value).clone())
                            } else {
                                RespValue::Null
                            }
                        } else {
                            RespValue::Error(
                                "ERR wrong number of arguments for 'get' command".to_string(),
                            )
                        };
                        let _ = writer.write_all(&response.serialize());
                    }
                    "SET" => {
                        if args.len() == 3 {
                            if let RespValue::BulkString(value_bytes) = &args[2] {
                                let key_name = args[1].as_key().unwrap_or_default();
                                store.set(key_name, value_bytes.clone());
                                let _ = writer.write_all(b"+OK\r\n");
                            } else {
                                let response =
                                    RespValue::Error("ERR wrong type for 'set' value".to_string());
                                let _ = writer.write_all(&response.serialize());
                            }
                        } else {
                            let response =
                                RespValue::Error("ERR wrong number of arguments".to_string());
                            let _ = writer.write_all(&response.serialize());
                        }
                    }
                    _ => {
                        let response = RespValue::Error(format!("ERR unknown command: {cmd_name}"));
                        let _ = writer.write_all(&response.serialize());
                    }
                }
            }
            _ => {
                let response =
                    RespValue::Error(format!("ERR unknown command: {}", parser.get_command()));
                let _ = writer.write_all(&response.serialize());
            }
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
