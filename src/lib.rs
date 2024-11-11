pub mod resp;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
    data: Arc<Mutex<HashMap<String, Value>>>,
}

impl MemoryDb {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set(&mut self, key: String, value: Value, expiry: Option<u64>) {
        let mut data = self.data.lock().unwrap();
        data.insert(key.clone(), value);

        if let Some(expiry) = expiry {
            let data = self.data.clone();
            let key = key.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(expiry)).await;
                let mut data = data.lock().unwrap();
                data.remove(&key);
            });
        }
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }

    pub fn remove(&mut self, key: &str) {
        let mut data = self.data.lock().unwrap();
        data.remove(key);
    }
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
                "SET" => handle_set(db, args),
                "GET" => handle_get(db, args),
                "INCR" => handle_incr(db, args),
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

fn handle_set<'a>(db: &'a mut MemoryDb, args: Vec<Value>) -> Value {
    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
    let value = args.get(1).unwrap().clone();

    let expiry = match args.get(2) {
        Some(v) => match unpack_bulk_str(v.clone()) {
            Ok(s) if s == "px" => {
                if let Value::BulkString(v) = args.get(3).unwrap() {
                    Some(v.parse::<u64>().unwrap())
                } else {
                    None
                }
            }
            _ => None,
        },
        None => None,
    };

    db.set(key, value, expiry);
    Value::SimpleString("OK".to_string())
}

fn handle_get<'a>(db: &'a MemoryDb, args: Vec<Value>) -> Value {
    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
    match db.get(&key) {
        Some(Value::Integer(i) ) => Value::SimpleString(i.to_string()),
        Some(v) => v.clone(),
        None => Value::Null,
    }
}

fn handle_incr<'a>(db: &'a mut MemoryDb, args: Vec<Value>) -> Value {
    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
    match db.get(&key) {
        Some(Value::Integer(i)) => {
            db.set(key, Value::Integer(i + 1), None);
            Value::Integer(i + 1)
        }
        Some(Value::SimpleString(s)) if s.parse::<i64>().is_ok() => {
            let i = s.parse::<i64>().unwrap();
            db.set(key, Value::Integer(i + 1), None);
            Value::Integer(i + 1)
        }
        Some(Value::BulkString(s)) if s.parse::<i64>().is_ok() => {
            let i = s.parse::<i64>().unwrap();
            db.set(key, Value::Integer(i + 1), None);
            Value::Integer(i + 1)
        }
        _ => {
            db.set(key, Value::Integer(1), None);
            Value::Integer(1)
        }
    }
}
