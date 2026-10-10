mod cmd;
mod ctx;
mod resp;
mod store;
mod types;

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use crate::cmd::dispatch;
use crate::ctx::Ctx;
use crate::resp::{RespError, RespParser};
use crate::store::Store;
use crate::types::RespValue;

fn handle_client(stream: TcpStream, store: Arc<Store>, ctx: Arc<Ctx>) {
    let mut writer = stream.try_clone().unwrap();
    let mut parser = RespParser::new(stream);
    loop {
        let request = match parser.parse() {
            Ok(v) => v,
            Err(RespError::Eof) => return,
            Err(RespError::Io(e)) => {
                eprintln!("{e}");
                return;
            }
            Err(RespError::Protocol(e)) => {
                let _ = writer.write_all(format!("-ERR Protocol error: {e}\r\n").as_bytes());
                return;
            }
        };
        match request {
            RespValue::Array(args) => {
                let reply = dispatch(&args, &store, &ctx);
                let _ = writer.write_all(&reply.serialize());
            }
            _ => {
                let _ = writer.write_all(b"-ERR expected array\r\n");
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:6379";
    let listener = TcpListener::bind(addr)?;
    println!("r > Listening in {}", addr);

    let store = Arc::new(Store::new());

    let ctx = Arc::new(Ctx::new());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store_cloned = Arc::clone(&store);
                let ctx_clone = Arc::clone(&ctx);
                thread::spawn(move || handle_client(stream, store_cloned, ctx_clone));
            }
            Err(e) => {
                eprintln!("X > Error! {:?}", e);
            }
        }
    }

    println!("Hello, world!");

    Ok(())
}
