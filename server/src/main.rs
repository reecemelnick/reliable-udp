use tokio::net::UdpSocket;
use std::sync::Arc;
use tokio::sync::mpsc;
use std::net::SocketAddr;
use update_csv::update_csv;
use update_csv::set_log_server;

mod gui;
mod forms;

use crate::gui::App;
use crate::forms::{FormSubmission};

struct Acknowledgement {
    sequence_number: i32,
}

async fn serialize_ack(ack: Acknowledgement) -> Vec<u8> {
    let mut byte_vector: Vec<u8> = Vec::new();
    byte_vector.extend_from_slice(&ack.sequence_number.to_ne_bytes());
    byte_vector
}

async fn read_from_proxy(sock: Arc<UdpSocket>, log_tx: mpsc::Sender<String>) -> Result<(), Box<dyn std::error::Error>> {

    let mut expected_sequence_number = 1;
    let mut recently_received_seq = 1;

    loop {

        // MESSAGE CAN ONLY PRINT ONCE!!!

        let mut buf = [0; 1024];
        let mut printed = false;

        match sock.recv_from(&mut buf).await {
            Ok((len, proxy_addr)) => {
                let sequence_number = &buf[..4];
                let sequence_number_int = i32::from_ne_bytes(sequence_number.try_into().unwrap());

                if sequence_number_int < recently_received_seq {
                    // print out for duplicate packet on the server
                    println!("Ignoring duplicate packet! Sequence Number was: {}. Expected: {} or {}", sequence_number_int, recently_received_seq, recently_received_seq + 1);
                    let _ = log_tx.send(format!("Ignoring duplicate packet! Sequence Number was: {}. Expected: {} or {}", sequence_number_int, recently_received_seq, recently_received_seq + 1)).await;
                    // increment_packet_count("Server", 0, 0, 1, 0).unwrap();
                    update_csv(("Server".to_string(), 0, 0, 1, 0));
                    continue;
                }

                if sequence_number_int > (expected_sequence_number - 1) {
                    let received_message = &buf[4..len];
                    let received_message_string = String::from_utf8_lossy(received_message);
                    println!("{:?} had sequence number: {:?}", received_message_string, sequence_number_int);
                    let _ = log_tx.send(format!("{:?} had sequence number: {:?}", received_message_string, sequence_number_int)).await;
                }

                // increment_packet_count("Server", 0, 1, 0, 0).unwrap();
                update_csv(("Server".to_string(), 0, 1, 0, 0));

                let ack = Acknowledgement {
                    sequence_number: sequence_number_int,
                };

                let _write_result = write_to_proxy(&sock, &proxy_addr, ack).await;
                
                expected_sequence_number = sequence_number_int + 1;
                recently_received_seq = sequence_number_int;
            
            }
            Err(e) => {
                eprintln!("Socket error: {}", e);
            }
        }
    }  
}

async fn write_to_proxy(sock: &Arc<UdpSocket>, proxy_addr: &SocketAddr, ack: Acknowledgement) -> Result<(), Box<dyn std::error::Error>> {
    let serialized_ack = serialize_ack(ack).await;
    sock.send_to(&serialized_ack, proxy_addr).await?;
    // increment_packet_count("Server", 1, 0, 0, 0).unwrap();
    update_csv(("Server".to_string(), 1, 0, 0, 0));
    Ok(())
}

#[tokio::main]
async fn main() {

    let (form_tx, form_rx) = mpsc::channel::<FormSubmission>(128);
    let (log_tx, log_rx) = mpsc::channel::<String>(128);

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Proxy logic on secondary thread so gui can be on main thread which is required
    std::thread::spawn(move || {
        rt.block_on(async {
            start_udp_server(form_rx, log_tx.clone()).await.unwrap();
        });
    });

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_resizable(true)
            .with_inner_size([520.0, 840.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Server",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(form_tx, log_rx)))),
    );
}

async fn start_udp_server(mut rx: mpsc::Receiver<FormSubmission>, log_tx: mpsc::Sender<String>) -> Result<(), Box<dyn std::error::Error>> {

    let mut server_config: Option<FormSubmission> = None;
    // Take config from gui before starting
    match rx.recv().await {
        Some(msg) => {
            server_config = Some(msg);
        }
        None => {
            println!("Nothing here");
        }
    }

    let config = server_config
        .expect("Expected FormSubmission from GUI")
        .with_defaults();

    println!("Server Config: {:#?}", config);

    set_log_server(config.log_ip.to_string(), config.log_port.to_string());

    let sock = Arc::new(UdpSocket::bind(format!("{}:{}", config.listen_ip, config.listen_port)).await?);

    let socket_clone1 = Arc::clone(&sock);
    let log_tx_clone = log_tx.clone();
    tokio::spawn(async move {
        let _ = read_from_proxy(socket_clone1, log_tx_clone).await;
    });

    loop {

    }
}
