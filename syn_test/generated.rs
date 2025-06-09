#![allow(unused_imports)]
use tokio::sync::mpsc;
use std::time::Duration;
#[derive(Debug)]
pub struct SensorThread {
    input_port: tokio::sync::mpsc::Receiver<Data>,
}
impl SensorThread {
    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(
            std::time::Duration::from_millis(100u64),
        );
        loop {
            interval.tick().await;
            if let Some(data) = self.input_port.recv().await {
                println!("Processing: {:?}", data);
            }
        }
    }
}
