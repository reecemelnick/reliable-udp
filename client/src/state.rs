use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

use crate::network::Packet;

pub struct ClientState {
    pub next_seq: i32,
    pub in_flight: Arc<Mutex<Vec<Packet>>>,
    pub sock: Arc<UdpSocket>,
    pub proxy_addr: String,
}

impl ClientState {
    pub fn new(socket: Arc<UdpSocket>) -> ClientState {
        ClientState {
            next_seq: 1,
            in_flight: Arc::new(Mutex::new(Vec::new())),
            sock: socket,
            proxy_addr: "127.0.0.1:8080".to_string(),
        }
    }
}