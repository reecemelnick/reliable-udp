use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex};
use csv_updater::{increment_packet_count, reset_csv};

use crate::Command;
use crate::check_for_ack;

#[derive(Clone)]
#[derive(Debug)]
pub struct Packet {
    pub sequence_number: i32,
    pub payload: Vec<u8>,
    pub retries: i32,
}


pub async fn serialize_packet(packet: &Packet) -> Vec<u8> {
    let mut byte_vector: Vec<u8> = Vec::new();
    byte_vector.extend_from_slice(&packet.sequence_number.to_ne_bytes());
    byte_vector.extend_from_slice(&packet.payload);
    byte_vector
}

pub async fn read_from_proxy(sock: Arc<UdpSocket>, tx: tokio::sync::mpsc::Sender<Command>) {
    loop {
        let mut buf = [0; 1024];
        match sock.recv_from(&mut buf).await {
            Ok((len, _addr)) => {
                let received_data = &buf[..len];
                let sequence_number_int = i32::from_ne_bytes(received_data.try_into().unwrap());
                println!("Got ack number: {}", sequence_number_int);
                let received_data = &buf[..len];

                if tx.send(Command::Ack(sequence_number_int)).await.is_err() {
                    println!("Client actor closed, stopping read_from_proxy");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving data: {:?}", e);
                continue;
            }
        }       
    }  
}

pub async fn send_and_wait(sock: &Arc<UdpSocket>, proxy_addr: &str, seq_number: i32, user_input: String, open_packets: Arc<Mutex<Vec<Packet>>>, tx: mpsc::Sender<Command>,) -> Result<(), Box<dyn std::error::Error>> {
    let trimmed_input = user_input.trim();

    let current_sequence_value = seq_number;

    let new_packet = Packet {
        sequence_number: current_sequence_value,
        payload: trimmed_input.as_bytes().to_vec(),
        retries: 0,
    };

    // turn every thing in packet struct to one big byte stream
    let serialized_packet = serialize_packet(&new_packet).await;

    // send to proxy
    sock.send_to(&serialized_packet, proxy_addr).await?;
    increment_packet_count("Client", 1, 0, 0).unwrap();

    // access shared packet vector
    let mut guard = open_packets.lock().await;
    guard.push(new_packet);
    println!("{:#?}", guard); // printing packet vector

    let sock_clone = Arc::clone(sock);
    let open_packets_clone = Arc::clone(&open_packets);
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let _ = check_for_ack(open_packets_clone, current_sequence_value, tx_clone).await;
    });

    Ok(())
}

pub async fn retransmit_packet(sock: &Arc<UdpSocket>, packet: &Packet , open_packets: &Arc<Mutex<Vec<Packet>>>, seq_number: i32) -> Result<(), Box<dyn std::error::Error>> {
    let serialized_packet = serialize_packet(&packet).await;
    let proxy_addr = "127.0.0.1:8080";
    sock.send_to(&serialized_packet, proxy_addr).await?;
    increment_packet_count("Client", 1, 0, 0).unwrap();
    println!("Packet resent!");
    Ok(())
}