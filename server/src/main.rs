use tokio::net::UdpSocket;
use std::sync::Arc;
use csv_updater::increment_packet_count;

struct Acknowledgement {
    sequence_number: i32,
}

async fn serialize_ack(ack: Acknowledgement) -> Vec<u8> {
    let mut byte_vector: Vec<u8> = Vec::new();
    byte_vector.extend_from_slice(&ack.sequence_number.to_ne_bytes());
    byte_vector
}

async fn read_from_proxy(sock: Arc<UdpSocket>, proxy_addr: &str) -> Result<(), Box<dyn std::error::Error>> {

    let mut recently_received_seq = 1;

    loop {
        let mut buf = [0; 1024];

        match sock.recv_from(&mut buf).await {
            Ok((len, _client_addr)) => {
                let sequence_number = &buf[..4];
                let sequence_number_int = i32::from_ne_bytes(sequence_number.try_into().unwrap());

                if sequence_number_int < recently_received_seq {
                    // print out for duplicate packet on the server
                    println!("Ignoring duplicate packet! Sequence Number was: {}. Expected: {}", sequence_number_int, recently_received_seq + 1);
                    continue;
                }
                let received_message = &buf[4..len];
                let received_message_string = String::from_utf8_lossy(received_message);
                println!("{:?} had sequence number: {:?}", received_message_string, sequence_number_int);
                increment_packet_count("Server", 0, 1, 0).unwrap();

                let ack = Acknowledgement {
                    sequence_number: sequence_number_int,
                };

                let _write_result = write_to_proxy(&sock, proxy_addr, ack).await;
                
                recently_received_seq = sequence_number_int;
            }
            Err(e) => {
                eprintln!("Socket error: {}", e);
            }
        }
    }  
}

async fn write_to_proxy(sock: &Arc<UdpSocket>, proxy_addr: &str, ack: Acknowledgement) -> Result<(), Box<dyn std::error::Error>> {
    // let server_response = "I recived the message, thank you!";
    let serialized_ack = serialize_ack(ack).await;
    sock.send_to(&serialized_ack, proxy_addr).await?;
    increment_packet_count("Server", 1, 0, 0).unwrap();
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock = Arc::new(UdpSocket::bind("127.0.0.1:9080").await?);
    let proxy_addr = "127.0.0.1:8080";
    
    let socket_clone1 = Arc::clone(&sock);
    tokio::spawn(async move {
        let _ = read_from_proxy(socket_clone1, proxy_addr).await;
    });

    loop {

    }
}