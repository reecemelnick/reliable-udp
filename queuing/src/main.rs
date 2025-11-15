use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::spawn;
use tokio::time::{sleep, Duration};
use rand::Rng;

#[tokio::main]
async fn main() {

    let v = Arc::new(Mutex::new(Vec::<String>::new()));

    loop {
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        let user_input = user_input.trim().to_string();
        let shared_v = Arc::clone(&v);

        {
            let mut vec_gaurd = shared_v.lock().await;
            vec_gaurd.push(user_input.clone());
        }

        let mut rng = rand::thread_rng();
        let delay = rng.gen_range(1000..=5000);

        spawn(async move {
            // Perform some work in the new task
            sleep(Duration::from_millis(delay)).await;
            println!("Delay on {} was: {}", user_input, delay);

            let mut vec_guard = shared_v.lock().await;
            if let Some(pos) = vec_guard.iter().position(|x| *x == user_input) {
                vec_guard.remove(pos);
            }
            println!("Removed '{}', remaining: {:?}", user_input, *vec_guard);
        });
    }
}
