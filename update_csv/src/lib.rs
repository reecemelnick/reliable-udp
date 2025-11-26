use bincode::ErrorKind;
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::OnceLock;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct MyData {
    values: (String, u32, u32, u32, u32),
}

static LOG_SERVER_ADDR: OnceLock<String> = OnceLock::new();

pub fn set_log_server(ip_addr: String, port: String) {
    let addr = format!("{}:{}", ip_addr, port);
    LOG_SERVER_ADDR.set(addr).unwrap();
}

pub fn update_csv(update_values: (String, u32, u32, u32, u32)) -> Result<(), Box<ErrorKind>> {
    let addr = LOG_SERVER_ADDR.get().expect("log server not set");

    let mut stream = TcpStream::connect(addr)?;
    let data = MyData {
        values: update_values,
    };
    let serialized = bincode::serialize(&data)?;
    let data_len = serialized.len() as u64;

    stream.write_all(&data_len.to_le_bytes())?;

    stream.write_all(&serialized)?;

    stream.flush()?;

    Ok(())
}
