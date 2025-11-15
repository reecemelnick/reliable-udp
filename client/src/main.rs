use tokio::net::UdpSocket;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use csv_updater::{increment_packet_count, reset_csv};

#[derive(Clone)]
#[derive(Debug)]
struct Packet {
    sequence_number: i32,
    payload: Vec<u8>,
}

async fn serialize_packet(packet: &Packet) -> Vec<u8> {
    let mut byte_vector: Vec<u8> = Vec::new();
    byte_vector.extend_from_slice(&packet.sequence_number.to_ne_bytes());
    byte_vector.extend_from_slice(&packet.payload);
    byte_vector
}

async fn read_from_proxy(sock: Arc<UdpSocket>, open_packets: Arc<Mutex<Vec<Packet>>>, seq_number: Arc<Mutex<i32>>) -> Result<(), Box<dyn std::error::Error>> {

    loop {
        let mut buf = [0; 1024];
        let (len, _addr) = sock.recv_from(&mut buf).await?;
        let received_data = &buf[..len];
        let sequence_number_int = i32::from_ne_bytes(received_data.try_into().unwrap());

        let seq_num_guard = seq_number.lock().await;
        if sequence_number_int != (*seq_num_guard - 1) {
            // print out to say that duplicate ack was found on client
            println!("Ignoring Duplicate ACK... (Unexpected Sequence Number)");
            increment_packet_count("Client", 0, 0, 1).unwrap();
            continue;
        }

        // --- could check my vector -- ignore if not in vector ?? 
        let mut open_packets_guard = open_packets.lock().await;
        let contains = open_packets_guard
            .iter()
            .any(|packet| packet.sequence_number == sequence_number_int);

        if contains {
            println!("Ack from server: {:?}", sequence_number_int);
            increment_packet_count("Client", 0, 1, 0).unwrap();
            open_packets_guard.retain(|packet| {
                packet.sequence_number != sequence_number_int
            });
        } else {
            println!("Ignoring Duplicate ACK... (Packet Already Removed from Active List)");
            increment_packet_count("Client", 0, 0, 1).unwrap();
        }
    }  
}

async fn send_and_wait(sock: &Arc<UdpSocket>, proxy_addr: &str, seq_number: Arc<Mutex<i32>>, user_input: String, open_packets: Arc<Mutex<Vec<Packet>>>) -> Result<(), Box<dyn std::error::Error>> {
    let trimmed_input = user_input.trim();

    let seq_num_guard = seq_number.lock().await;
    let current_sequence_value = *seq_num_guard;

    let new_packet = Packet {
        sequence_number: current_sequence_value,
        payload: trimmed_input.as_bytes().to_vec(),
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

    // clone data for 
    let socket_clone = Arc::clone(&sock);
    let open_packets_clone_for_spawn = Arc::clone(&open_packets);
    let seq_number_clone_for_spawn = Arc::clone(&seq_number);
    tokio::spawn(async move {
        let _ = check_for_ack(socket_clone, open_packets_clone_for_spawn, current_sequence_value).await;
    });

    Ok(())
}

async fn check_for_ack(sock: Arc<UdpSocket>, open_packets: Arc<Mutex<Vec<Packet>>>, seq_number: i32) -> Result<(), Box<dyn std::error::Error>> {
    let mut number_of_retries = 0;
    loop {    
        // this is the timeout to check for retransmission
        sleep(Duration::from_millis(5000)).await;

        let maybe_packet = {
            let open_packets_guard = open_packets.lock().await;
            open_packets_guard
                .iter()
                .find(|p| p.sequence_number == seq_number)
                .cloned()
        };

        match maybe_packet {
            Some(packet) => {
                println!("Retransmitting seq {}: {:?}", seq_number, packet);
                retransmit_packet(&sock, &packet, &open_packets, seq_number).await;
                number_of_retries += 1;

                if number_of_retries > 3 {
                    println!("Giving up on this message...");
                    break;
                }
            }
            None => {
                // ACK from server was successfully received
                break;
            }
        }
    }
    Ok(())
}

async fn retransmit_packet(sock: &Arc<UdpSocket>, packet: &Packet , open_packets: &Arc<Mutex<Vec<Packet>>>, seq_number: i32) -> Result<(), Box<dyn std::error::Error>> {
    let serialized_packet = serialize_packet(&packet).await;
    let proxy_addr = "127.0.0.1:8080";
    sock.send_to(&serialized_packet, proxy_addr).await?;
    increment_packet_count("Client", 1, 0, 0).unwrap();
    println!("Packet resent!");
    Ok(())
}

async fn read_input_from_user(sock: &Arc<UdpSocket>, proxy_addr: &str, seq_number: Arc<Mutex<i32>>, open_packets: Arc<Mutex<Vec<Packet>>>) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut user_input = String::new();
        println!("Please enter a message for the server:");
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        
        // clone shared data for function call
        let seq_number_clone = Arc::clone(&seq_number);
        let open_packets_clone = Arc::clone(&open_packets);
        let _ = send_and_wait(sock, proxy_addr, seq_number_clone, user_input, open_packets_clone).await;
        let mut guard = seq_number.lock().await;
        *guard += 1;
    } 
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    reset_csv();
    let next_sequence_number = Arc::new(Mutex::new(1));
    let sock = Arc::new(UdpSocket::bind("127.0.0.1:7080").await?);
    let proxy_addr = "127.0.0.1:8080";
    let open_packets: Arc<Mutex<Vec<Packet>>> = Arc::new(Mutex::new(Vec::new()));
    
    // clone shared data for asyncronous task
    let socket_clone = Arc::clone(&sock);
    let open_packets_clone_for_spawn = Arc::clone(&open_packets);
    let sequence_number_clone = Arc::clone(&next_sequence_number);
    tokio::spawn(async move {
        let _ = read_from_proxy(socket_clone, open_packets_clone_for_spawn, sequence_number_clone).await;
    });

    let _user_input = read_input_from_user(&sock, proxy_addr, Arc::clone(&next_sequence_number), Arc::clone(&open_packets)).await;

    Ok(())
 
}