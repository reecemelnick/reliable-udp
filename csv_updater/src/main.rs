use std::net::{TcpListener};
use std::io::prelude::*;
use bincode::ErrorKind;
use serde::{Serialize, Deserialize};
use csv_updater::increment_packet_count;
use csv_updater::reset_csv;

#[derive(Serialize, Deserialize, Debug)]
struct MyData {
    values: (String, u32, u32, u32, u32),
}

fn handle_client() -> Result<(), Box<ErrorKind>> {
    let listener = TcpListener::bind("127.0.0.1:8000")?; // pass in IP

    let mut request = String::new(); 
    
    for stream in listener.incoming() {
        let mut stream = stream?;
        
        let mut len_bytes = [0u8; 8];
        stream.read_exact(&mut len_bytes)?;
        let data_len = u64::from_le_bytes(len_bytes);

        let mut buffer = vec![0u8; data_len as usize];
        stream.read_exact(&mut buffer)?;

        let received_data: MyData = bincode::deserialize(&buffer)?;
        println!("Received: {:?}", received_data);
        increment_packet_count(&received_data.values.0.to_string(), received_data.values.1, received_data.values.2, received_data.values.3, received_data.values.4);
    }
    Ok(())
}

fn main() {
    reset_csv();
    match handle_client() {
        Ok(res) => res,
        Err(err) => panic!("Failed to listen to connections: {}", err),
    };
}
