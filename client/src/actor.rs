use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::net::UdpSocket;

use crate::Command;
use crate::state::ClientState;
use crate::packets::{process_ack, handle_timeout, handle_send_message};

pub async fn client_actor(mut rx: mpsc::Receiver<Command>, tx: mpsc::Sender<Command>, socket: Arc<UdpSocket>) {

    let mut state = ClientState::new(socket);
    
    loop {
        match rx.recv().await {
            Some(Command::SendMessage(msg)) => {
                handle_send_message(msg, &mut state, tx.clone()).await;
            }
            Some(Command::Ack(seq)) => {
                process_ack(seq, &state.in_flight).await;
            }
            Some(Command::Timeout(seq)) => {
                handle_timeout(seq, &state).await;
            }   
            None => {
                println!("Got nothing");
            }
        }
    }

}