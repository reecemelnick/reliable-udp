use tokio::net::UdpSocket;
// use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use csv_updater::{increment_packet_count, reset_csv};
use tokio::sync::mpsc::{self, Receiver, Sender};
use std::collections::HashMap;
use tokio::io::{self, AsyncBufReadExt, BufReader};

mod actor;
mod packets;
mod state;
mod network;

use actor::client_actor;
use network::read_from_proxy;
use network::Packet;

enum Command {
    SendMessage(String),
    Ack(i32),
    Timeout(i32),
}

async fn check_for_ack(open_packets: Arc<Mutex<Vec<Packet>>>, seq: i32, tx: mpsc::Sender<Command>) -> Result<(), Box<dyn std::error::Error>> {
    let mut number_of_retries = 0;
    loop {    
        // this is the timeout to check for retransmission
        sleep(Duration::from_millis(2000)).await;

        let mut remove_packet = false;

        {
            let mut guard = open_packets.lock().await;
            if let Some(packet) = guard.iter_mut().find(|p| p.sequence_number == seq) {
                if packet.retries >= 3 {
                    println!("Giving up on seq {}", seq);
                    remove_packet = true;
                } else {
                    println!("Timeout for seq {}, retransmitting...", seq);
                    packet.retries += 1;
                    let _ = tx.send(Command::Timeout(seq)).await;
                }
            } else {
                break; // ACK received
            }
        }
        
        // flags set to true we remove the packet from in flight list
        if remove_packet {
            let mut guard = open_packets.lock().await;
            guard.retain(|p| p.sequence_number != seq);
            break;
        }
    }
    Ok(())
}

async fn read_input_from_user(tx: tokio::sync::mpsc::Sender<Command>) {
    // tokios aysnc input reader 
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    // read one line and send it to the client actor
    while let Ok(Some(line)) = lines.next_line().await {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if tx.send(Command::SendMessage(trimmed.to_string())).await.is_err() {
            break; // channel closed
        }
    }
}

// single source of truth state handler

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    reset_csv();
    let sock = Arc::new(UdpSocket::bind("127.0.0.1:7080").await?);

    let (tx, rx) = mpsc::channel(100);

    let tx_actor = tx.clone();
    let sock_actor = sock.clone();
    tokio::spawn(async move {
        client_actor(rx, tx_actor, sock_actor).await;
    });

    let tx_input = tx.clone();
    tokio::spawn(async move {
        read_input_from_user(tx_input).await;
    });

    let tx_proxy = tx.clone();
    let sock_proxy = sock.clone();
    tokio::spawn(async move {
        read_from_proxy(sock_proxy, tx_proxy).await;
    });

    loop {

    }

    Ok(())
 
}
