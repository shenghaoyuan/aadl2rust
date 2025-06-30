use pingpongcallc::aProcess;

fn main() {
    let process = aProcess::new();
    process.start();

    // 主线程阻塞
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}