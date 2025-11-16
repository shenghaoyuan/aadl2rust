// use rmatest::node_aProcess;

// fn main() {
//     let process = node_aProcess::new();
//     process.start();

//     // 主线程阻塞
//     loop {
//         std::thread::sleep(std::time::Duration::from_secs(60));
//     }
// }

use rmstest::rmssysSystem;

fn main() {
    // 创建系统实例（包含进程和CPU绑定信息）
    let system = rmssysSystem::new();

    // 启动系统，系统会为每个进程分配CPU并启动线程
    system.run();

    // 主线程阻塞，保持程序运行
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
