use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::net::UdpSocket;

use crate::Command;
use crate::state::ClientState;
use crate::packets::{process_ack, handle_timeout, handle_send_message};
use crate::forms::{FormSubmissionClean};

pub async fn client_actor(mut rx: mpsc::Receiver<Command>, tx: mpsc::Sender<Command>, socket: Arc<UdpSocket>, log_tx: mpsc::Sender<String>, config: FormSubmissionClean) {

    let mut state = ClientState::new(socket);
    state.proxy_addr = format!("{}:{}", config.target_ip, config.target_port);

    loop {
        match rx.recv().await {
            Some(Command::SendMessage(msg)) => {
                handle_send_message(msg, &mut state, tx.clone(), config.max_retries, config.timeout).await;
            }
            Some(Command::Ack(seq)) => {
                process_ack(seq, &state.in_flight, log_tx.clone()).await;
            }
            Some(Command::Timeout(seq)) => {
                handle_timeout(seq, &state, log_tx.clone(), state.proxy_addr.clone()).await;
            }   
            None => {
                println!("Got nothing");
            }
        }
    }

}