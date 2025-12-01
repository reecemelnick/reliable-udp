use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use update_csv::set_log_server;
use tokio::net::UdpSocket;

mod actor;
mod packets;
mod state;
mod network;
mod gui;
mod forms;

use crate::gui::App;
use crate::forms::{FormSubmission};

use actor::client_actor;
use network::read_from_proxy;
use network::Packet;

enum Command {
    SendMessage(String),
    Ack(i32),
    Timeout(i32),
}

async fn check_for_ack(open_packets: Arc<Mutex<Vec<Packet>>>, seq: i32, tx: mpsc::Sender<Command>, max_retries: i32, timeout: usize) -> Result<(), Box<dyn std::error::Error>> {
    loop {    
        // this is the timeout to check for retransmission
        sleep(Duration::from_millis(timeout.try_into().unwrap())).await;

        {
            let mut guard = open_packets.lock().await;
            if let Some(packet) = guard.iter_mut().find(|p| p.sequence_number == seq) {
                if packet.retries >= max_retries {
                    println!("Giving up on packet {}", seq);
                    std::process::exit(0);
                } else {
                    println!("Timeout for seq {}, retransmitting...", seq);
                    packet.retries += 1;
                    let _ = tx.send(Command::Timeout(seq)).await;
                }
            } else {
                break; // ACK received
            }
        }
    }
    Ok(())
}

async fn read_input_from_user(tx: tokio::sync::mpsc::Sender<Command>, mut input_rx: mpsc::Receiver<String>) {

    while let Some(line) = input_rx.recv().await {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if tx.send(Command::SendMessage(trimmed.to_string())).await.is_err() {
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    let (form_tx, form_rx) = mpsc::channel::<FormSubmission>(128);
    let (log_tx, log_rx) = mpsc::channel::<String>(128);
    let (input_tx, input_rx) = mpsc::channel::<String>(128);

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Proxy logic on secondary thread so gui can be on main thread which is required
    std::thread::spawn(move || {
        rt.block_on(async {
            start_udp_client(form_rx, log_tx.clone(), input_rx).await.unwrap();
        });
    });

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_resizable(true)
            .with_inner_size([520.0, 840.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Client",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(form_tx, log_rx, input_tx)))),
    );
}

async fn start_udp_client(mut rx: mpsc::Receiver<FormSubmission>, log_tx: mpsc::Sender<String>, input_rx: mpsc::Receiver<String>) -> Result<(), Box<dyn std::error::Error>> {
    
    let mut client_config: Option<FormSubmission> = None;
    // Take config from gui before starting
    match rx.recv().await {
        Some(msg) => {
            client_config = Some(msg);
        }
        None => {
            println!("Nothing here");
        }
    }

    let config = client_config
        .expect("Expected FormSubmission from GUI")
        .with_defaults();

    println!("Client Config: {:#?}", config);

    set_log_server(config.log_ip.to_string(), config.log_port.to_string());
    let client_addr = format!("{}:{}", config.listen_ip.to_string(), config.listen_port.to_string());
    
    let sock = Arc::new(UdpSocket::bind(client_addr).await?);

    let (tx, rx) = mpsc::channel(100); // for the actor

    let tx_actor = tx.clone();
    let sock_actor = sock.clone();
    let log_tx_clone = log_tx.clone();
    tokio::spawn(async move {
        client_actor(rx, tx_actor, sock_actor, log_tx_clone, config).await;
    });

    let tx_input = tx.clone();
    tokio::spawn(async move {
        read_input_from_user(tx_input, input_rx).await;
    });

    let tx_proxy = tx.clone();
    let sock_proxy = sock.clone();
    tokio::spawn(async move {
        read_from_proxy(sock_proxy, tx_proxy).await;
    });

    loop {

    }
}