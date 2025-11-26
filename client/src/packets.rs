use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use update_csv::update_csv;

use crate::network::Packet;
use crate::network::send_and_wait;
use crate::state::ClientState;
use crate::Command;

pub async fn handle_send_message(msg: String, state: &mut ClientState, tx: mpsc::Sender<Command>, max_retries: i32, timeout: usize) {
    let sequence_number = state.next_seq;
    state.next_seq += 1;

    let sock = Arc::clone(&state.sock);
    let in_flight_packets = Arc::clone(&state.in_flight);
    let proxy_addr = state.proxy_addr.clone();
    let tx_for_resend = tx.clone();

    tokio::spawn(async move {
        let _ = send_and_wait(&sock, &proxy_addr, sequence_number, msg, in_flight_packets, tx_for_resend, max_retries, timeout).await;
    });
}

pub async fn process_ack(seq: i32, in_flight: &Arc<Mutex<Vec<Packet>>>, log_tx: mpsc::Sender<String>) {
    let mut guard = in_flight.lock().await;

    if guard.iter().any(|p| p.sequence_number == seq) {
        println!("Received ACK for seq {}", seq);
        let _ = log_tx.send(format!("Got ACK number: {}", seq)).await;
        // increment_packet_count("Client", 0, 1, 0, 0).unwrap();
        update_csv(("Client".to_string(), 0, 1, 0, 0));

        guard.retain(|p| p.sequence_number != seq);
    } else {
        println!("Duplicate ACK");
        let _ = log_tx.send(format!("Duplicate ACK: {}. Dropping...", seq)).await;
        // increment_packet_count("Client", 0, 0, 1, 0).unwrap();
        update_csv(("Client".to_string(), 0, 0, 1, 0));
    }
}

pub async fn handle_timeout(seq: i32, state: &ClientState, log_tx: mpsc::Sender<String>, proxy_addr: String) {

    let _ = log_tx.send(format!("Timeout for Packet {}, retransmitting...", seq)).await;

    let in_flight_packets = Arc::clone(&state.in_flight);
    let sock = Arc::clone(&state.sock);

    // Look up the packet in the in-flight list
    if let Some(packet) = {
        let guard = in_flight_packets.lock().await;
        guard.iter().find(|p| p.sequence_number == seq).cloned()
    } {
        
        tokio::spawn(async move {
            crate::network::retransmit_packet(&sock, &packet, proxy_addr).await.unwrap();
        });

    } else {
        // No retransmit needed
    }
}