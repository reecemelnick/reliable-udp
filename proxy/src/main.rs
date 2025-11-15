use tokio::net::UdpSocket;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::spawn;
use tokio::time::{sleep, Duration};
use std::io;
use rand::Rng;
use csv_updater::increment_packet_count;

#[derive(Debug)]
struct Packet {
    sequence_number: i32,
    payload: Vec<u8>,
}


async fn foward_to_server(sock: &UdpSocket, data: &[u8]) -> io::Result<()> {
    let server_addr = "127.0.0.1:9080";

    // try to send to server
    let len = sock.send_to(data, server_addr).await?;
    println!("{:?} bytes sent to server", len);
    increment_packet_count("Proxy", 1, 0, 0).unwrap();
    Ok(())
    
}

async fn foward_to_client(sock: &UdpSocket, data: &[u8]) -> io::Result<()> {
    let client_addr = "127.0.0.1:7080";

    let len = sock.send_to(data, client_addr).await?;
    println!("{:?} bytes sent to client", len);
    increment_packet_count("Proxy", 1, 0, 0).unwrap();
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // read from client and server at same time
    // if from client forward to server
    // if from server forward to client
    let client_v = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
    // let server_v = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
    let sock = Arc::new(UdpSocket::bind("127.0.0.1:8080").await?);

    loop {
        let mut buf = [0; 1024];
        let (len, sender_addr) = sock.recv_from(&mut buf).await?;
        increment_packet_count("Proxy", 0, 1, 0).unwrap();
        let received_data = buf[..len].to_vec();

        let port_num = sender_addr.port();
        

        let shared_v = Arc::clone(&client_v);

        {
            let mut vec_gaurd = shared_v.lock().await;
            vec_gaurd.push(received_data.clone());
        }

        let mut rng = rand::thread_rng();
        let delay = rng.gen_range(3000..=5000);
        
        let value = received_data.clone();
        let sock_clone = sock.clone();
        spawn(async move {
            // Perform some work in the new task
            sleep(Duration::from_millis(delay)).await;
            println!("Delay on {:?} was: {}", value, delay);

            // at this point order is important - possibly only for sending to server? 
            
            if port_num == 7080 {
                if chopping_block() {
                    let _ = foward_to_server(&sock_clone, &received_data).await;
                }
            } else if port_num == 9080 {
                if chopping_block() {
                    let _ = foward_to_client(&sock_clone, &received_data).await;
                }
            }
    
            let mut vec_guard = shared_v.lock().await;
            if let Some(pos) = vec_guard.iter().position(|x| *x == value) {
                vec_guard.remove(pos);
            }
            println!("Removed '{:?}', remaining: {:?}", value, *vec_guard);
        });

    }

}

fn chopping_block() -> bool {
    let mut rng = rand::thread_rng();
    let percent = rng.gen_range(50..=120);   
    if percent < 50 {
        println!("PACKET DROPPED");
        return false;
    } 
    true
}