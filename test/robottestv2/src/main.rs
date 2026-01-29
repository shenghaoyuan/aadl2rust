use robottest::{System,robotSystem};

pub fn boot<S: System>() {
    let system = S::new();
    system.run();

    // 主线程保持运行，防止退出
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
fn main(){
    boot::<robotSystem>();
}

// use rmatest::node_aProcess;

// fn main() {
//     let process = node_aProcess::new();
//     process.start();

//     // 主线程阻塞
//     loop {
//         std::thread::sleep(std::time::Duration::from_secs(60));
//     }
// }
// use robottest::robotSystem;

// fn main() {
//     // 创建系统实例（包含进程和CPU绑定信息）
//     let system = robotSystem::new();

//     // 启动系统，系统会为每个进程分配CPU并启动线程
//     system.run();

//     // 主线程阻塞，保持程序运行
//     loop {
//         std::thread::sleep(std::time::Duration::from_secs(60));
//     }
// }
