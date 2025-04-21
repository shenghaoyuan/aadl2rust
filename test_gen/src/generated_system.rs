// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-04-21T17:26:17.669269200+08:00

#![allow(unused_imports)]
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

// === 子程序实现 ===
// 子程序: PingPong.Send
mod sender_spg {
    use std::sync::atomic::{AtomicI32, Ordering};

    static COUNT: AtomicI32 = AtomicI32::new(0);

    // 从Java转换的Rust实现
    // 原始Java代码:
    /*
public class PingPong {
    public static int count &#x3D; 0;
    public static void Send(int val) {
        System.out.println(&quot;[PING] &quot; + count);
        count &#x3D; count + 1;
        val &#x3D; count;
    }
    public static void Receive(int val) {
        if (val !&#x3D; 0) {
            System.out.println(&quot;[PONG] &quot; + val);
        }
    }
}*/

    pub fn send(val: &mut i32) {
        let new_count = COUNT.fetch_add(1, Ordering::SeqCst);
        println!("[PING] {}", new_count + 1);
        *val = new_count + 1;
    }

}
// 子程序: PingPong.Receive
mod receiver_spg {
    use std::sync::atomic::{AtomicI32, Ordering};


    // 从Java转换的Rust实现
    // 原始Java代码:
    /*
public class PingPong {
    public static int count &#x3D; 0;
    public static void Send(int val) {
        System.out.println(&quot;[PING] &quot; + count);
        count &#x3D; count + 1;
        val &#x3D; count;
    }
    public static void Receive(int val) {
        if (val !&#x3D; 0) {
            System.out.println(&quot;[PONG] &quot; + val);
        }
    }
}*/


    pub fn receive(val: i32) {
        if val != 0 {
            println!("[PONG] {}", val);
        }
    }
}

// === 线程定义 ===
/// 线程: the_sender
/// 周期: 2000ms
/// 优先级: 5
struct TheSenderThread {
    id: u32,
    sender: Option<mpsc::Sender<i32>>,  // 改为i32类型
    receiver: Option<mpsc::Receiver<i32>>,
}

impl TheSenderThread {
    fn new() -> Self {
        Self {
            id: 0,
            sender: None,
            receiver: None,
        }
    }
    
    fn run(&mut self) {
        let period = Duration::from_millis(2000);
        let exec_time = Duration::from_millis(1);
        
        loop {
            let start = Instant::now();
            
            // 线程执行逻辑
            if let Some(sender) = &self.sender {
                let mut val = 0;
                sender_spg::send(&mut val);
                sender.send(val).unwrap();
            }
            
            thread::sleep(exec_time);
            let elapsed = start.elapsed();
            if elapsed < period {
                thread::sleep(period - elapsed);
            }
        }
    }
}
/// 线程: the_receiver
/// 周期: 1000ms
/// 优先级: 10
struct TheReceiverThread {
    id: u32,
    sender: Option<mpsc::Sender<i32>>,  // 改为i32类型
    receiver: Option<mpsc::Receiver<i32>>,
}

impl TheReceiverThread {
    fn new() -> Self {
        Self {
            id: 1,
            sender: None,
            receiver: None,
        }
    }
    
    fn run(&mut self) {
        let period = Duration::from_millis(1000);
        let exec_time = Duration::from_millis(1);
        
        loop {
            let start = Instant::now();
            
            // 线程执行逻辑
            if let Some(receiver) = &self.receiver {
                let val = receiver.recv().unwrap();
                receiver_spg::receive(val);
            }
            
            thread::sleep(exec_time);
            let elapsed = start.elapsed();
            if elapsed < period {
                thread::sleep(period - elapsed);
            }
        }
    }
}

// === 系统初始化 ===
pub fn initialize_system() {
    // 创建线程实例
    let mut the_sender = TheSenderThread::new();
    let mut the_receiver = TheReceiverThread::new();
    
    // 建立连接
    let (the_sender_sender, the_receiver_receiver) = mpsc::channel();
    the_sender.sender = Some(the_sender_sender);
    the_receiver.receiver = Some(the_receiver_receiver);
    
    // 启动线程
    thread::Builder::new()
        .name("the_sender".to_string())
        .stack_size(40000)
        .spawn(move || {
            the_sender.run();
        }).unwrap();
    thread::Builder::new()
        .name("the_receiver".to_string())
        .stack_size(40000)
        .spawn(move || {
            the_receiver.run();
        }).unwrap();
}