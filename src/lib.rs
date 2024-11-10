pub mod resp;
use std::collections::HashMap;

use anyhow::Result;
use resp::Value;
use tokio::net::{TcpListener, TcpStream};

#[derive(Clone, Debug)]
pub struct Server<'a> {
    addr: String,
    db: &'a MemoryDb,
}

#[derive(Clone, Debug, Default)]
pub struct MemoryDb {
    pub data: HashMap<String, Value>,
}

impl<'a> Server<'a> {
    pub fn new(addr: &str, db: &'a MemoryDb) -> Self {
        Self {
            addr: addr.to_string(),
            db,
        }
    }

    pub async fn run(self) {
        let listener = TcpListener::bind(&self.addr).await.unwrap();
        loop {
            let stream = listener.accept().await;
            match stream {
                Ok((stream, _)) => {
                    println!("Accepted connection from {:?}", stream.peer_addr().unwrap());

                    let mut db = self.db.clone();
                    tokio::spawn(async move {
                        handle_conn(&mut db, stream).await;
                    });
                }
                Err(e) => {
                    println!("Failed to accept connection: {:?}", e);
                }
            }
        }
    }
}

async fn handle_conn<'a>(db: &'a mut MemoryDb, stream: TcpStream) {
    let mut handler = resp::RespHandler::new(stream);

    println!("Starting read loop");

    loop {
        let value = match handler.read_value().await {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to read value: {:?}", e);
                break;
            }
        };
        println!("Got value {:?}", value);

        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();

            match command.as_str() {
                "PING" => Value::SimpleString("PONG".to_string()),
                "ECHO" => args.first().unwrap().clone(),
                "SET" => {
                    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
                    let value = args.get(1).unwrap().clone();
                    db.data.insert(key, value);
                    Value::SimpleString("OK".to_string())
                }
                "GET" => {
                    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
                    match db.data.get(&key) {
                        Some(v) => v.clone(),
                        None => Value::Null,
                    }
                }
                c => panic!("Cannot handle command {}", c),
            }
        } else {
            break;
        };

        println!("Sending value {:?}", response);

        handler.write_value(response).await.unwrap();
    }
}

// *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n

fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => Ok((
            unpack_bulk_str(a.first().unwrap().clone())?,
            a.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected command to be a bulk string")),
    }
}
