// src/main.rs

mod generated_system;

fn main() {
    generated_system::initialize_system();
    
    // 示例：运行10秒
    std::thread::sleep(std::time::Duration::from_secs(10));
    println!("System running for 10 seconds, exiting...");
}