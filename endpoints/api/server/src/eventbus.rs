#![allow(dead_code)]

use tokio::{
    sync::broadcast,
    time::{Duration, sleep},
};

/// تجربة الـ event bus القديمة — متسيبة هنا لحد ما تتحول لكريت لوحدها.
pub async fn start() {
    println!("Started...");

    let (tx, mut rx) = broadcast::channel(100);
    let tx1 = tx.clone();

    tokio::spawn(async move {
        let mut counter = 1;
        loop {
            let message = format!("User {counter} created");
            if tx1.send(message).is_err() {
                break;
            }
            counter += 1;
            sleep(Duration::from_secs(1)).await;
        }
    });

    tokio::spawn(async move {
        while let Ok(message) = rx.recv().await {
            println!("Received event: {message}");
        }
    });
}
