use tokio::net::UdpSocket;
use std::sync::Arc;
use tokio::sync::Mutex;
// use eframe::IconData;

use tokio::time::{sleep, Duration};
use std::io;
use rand::Rng;
// use csv_updater::increment_packet_count;
use update_csv::update_csv;
use update_csv::set_log_server;
use tokio::sync::mpsc;
use std::net::SocketAddr;

mod gui;
mod forms;
use crate::gui::App;
use crate::forms::{FormSubmission};

async fn foward_to_server(sock: &UdpSocket, data: &[u8], log_tx: mpsc::Sender<String>, server_addr: String) -> io::Result<()> {
    let _ = log_tx.send(format!("Forwarding {:?} to {}", data, server_addr)).await;
    let _len = sock.send_to(data, server_addr).await?;
    let _ =  update_csv(("Proxy".to_string(), 1, 0, 0, 0));
    Ok(())
    
}

async fn foward_to_client(sock: &UdpSocket, data: &[u8], log_tx: mpsc::Sender<String>, client_addr: SocketAddr) -> io::Result<()> {
    let _ = log_tx.send(format!("Fowarding {:?} to {}", data, client_addr)).await;
    let _len = sock.send_to(data, client_addr).await?;
    let _ = update_csv(("Proxy".to_string(), 1, 0, 0, 0));
    Ok(())
}

#[tokio::main]
async fn main() {

    let (tx, rx) = mpsc::channel::<FormSubmission>(128);
    let (log_tx, log_rx) = mpsc::channel::<String>(128);

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Proxy logic on secondary thread so gui can be on main thread which is required
    std::thread::spawn(move || {
        rt.block_on(async {
            start_udp_proxy(rx, log_tx.clone()).await.unwrap();
        });
    });
    
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_resizable(true)
            .with_inner_size([520.0, 840.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Proxy Server",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(tx, log_rx)))),
    );
}

async fn start_udp_proxy(mut rx: mpsc::Receiver<FormSubmission>, log_tx: mpsc::Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
    
    let mut proxy_config: Option<FormSubmission> = None;
    // Take config from gui before starting
    match rx.recv().await {
        Some(msg) => {
            proxy_config = Some(msg);
        }
        None => {
            println!("No config set");
        }
    }

    let config = proxy_config
        .expect("Expected FormSubmission from GUI")
        .with_defaults();

    println!("Proxy Config: {:#?}", config);

    set_log_server(config.log_ip.to_string(), config.log_port.to_string());

    let client_v = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
    let proxy_addr = format!("{}:{}", config.listen_ip, config.listen_port);
    let server_addr = format!("{}:{}", config.target_ip, config.target_port);
    let mut client_addr: Option<SocketAddr> = None;
    let sock = Arc::new(UdpSocket::bind(proxy_addr).await?);


    loop {
        let mut buf = [0; 1024];
        let (len, sender_addr) = sock.recv_from(&mut buf).await?;
        if client_addr == None {
            client_addr = Some(sender_addr);
        }

        let received_data = buf[..len].to_vec();
        let port_num = sender_addr.port();

        let _ = log_tx.send(format!("Received packet {:?} from {}", received_data, sender_addr)).await;
        let _ = update_csv(("Proxy".to_string(), 0, 1, 0, 0));

        let shared_v = client_v.clone();
        {
            let mut guard = shared_v.lock().await;
            guard.push(received_data.clone());
        }

        let sock_clone = sock.clone();
        let log_tx_clone = log_tx.clone();
        let server_addr_clone = server_addr.clone();
        tokio::spawn(async move {
            let value = received_data.clone();

            if port_num != config.target_port {
                if chopping_block(config.client_drop) {
                    if !delay_decider(config.client_delay) {
                        let _ = log_tx_clone.send(format!("Delaying packet from {}", sender_addr)).await;
                        sleep(Duration::from_millis(random_delay(config.client_delay_time_min, config.client_delay_time_max).await.try_into().unwrap())).await;
                    } 
                    let _ = foward_to_server(&sock_clone, &received_data, log_tx_clone, server_addr_clone).await;
                } else {
                    let _ = log_tx_clone.send(format!("Dropping packet from {}", sender_addr)).await;
                }
            } else if port_num == config.target_port {
                if chopping_block(config.server_drop) {
                    if !delay_decider(config.server_delay) {
                        let _ = log_tx_clone.send(format!("Delaying packet from {}", sender_addr)).await;
                        sleep(Duration::from_millis(random_delay(config.server_delay_time_min, config.server_delay_time_max).await.try_into().unwrap())).await;
                    }
                    let _ = foward_to_client(&sock_clone, &received_data, log_tx_clone, client_addr.unwrap()).await;
                } else {
                    let _ = log_tx_clone.send(format!("Dropping packet from {}", sender_addr)).await;
                }
            }

            let mut guard = shared_v.lock().await;
            if let Some(pos) = guard.iter().position(|x| *x == value) {
                guard.remove(pos);
            }
        });
    }
}

async fn random_delay(min: usize, max: usize) -> usize {
    let mut rng = rand::thread_rng();
    let delay = rng.gen_range(min..=max);
    delay
}

fn delay_decider(delay_chance: i32) -> bool {
    let mut rng = rand::thread_rng();
    let percent = rng.gen_range(0..100);   
    if percent < delay_chance {
        println!("Delaying packet...");
        return false;
    } 
    true
}

fn chopping_block(drop_chance: i32) -> bool {
    let mut rng = rand::thread_rng();
    let percent = rng.gen_range(0..100);   
    if percent < drop_chance {
        let _ = update_csv(("Proxy".to_string(), 0, 0, 0, 1));
        println!("Dropping packet...");
        return false;
    } 
    true
}