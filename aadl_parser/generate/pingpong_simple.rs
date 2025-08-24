// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-08-23 18:18:05

#![allow(unused_imports)]
use std::sync::{mpsc, Arc};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use libc::{
    pthread_self, sched_param, pthread_setschedparam, SCHED_FIFO,
    cpu_set_t, CPU_SET, CPU_ZERO, sched_setaffinity,
};
include!(concat!(env!("OUT_DIR"), "/aadl_c_bindings.rs"));

// ---------------- cpu ----------------
fn set_thread_affinity(cpu: isize) {
    unsafe {
        let mut cpuset: cpu_set_t = std::mem::zeroed();
        CPU_ZERO(&mut cpuset);
        CPU_SET(cpu as usize, &mut cpuset);
        sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset);
    }
}

// AADL System: root
#[derive(Debug)]
pub struct rootSystem {
    // 子组件进程（the_proc : process proc）
    #[allow(dead_code)]
    pub the_proc: procProcess,
}

impl rootSystem {
    // Creates a new system instance
    pub fn new() -> Self {
        let mut the_proc: procProcess = procProcess::new(0);
        return Self { the_proc }  //显式return;
    }
    
    // Runs the system, starts all processes
    pub fn run(self: Self) -> () {
        self.the_proc.start();;
    }
    
}

// AADL Process: proc
#[derive(Debug)]
pub struct procProcess {
    // 进程 CPU ID
    pub cpu_id: isize,
    // 子组件线程（the_sender : thread sender）
    #[allow(dead_code)]
    pub the_sender: senderThread,
    // 子组件线程（the_receiver : thread receiver）
    #[allow(dead_code)]
    pub the_receiver: receiverThread,
}

impl procProcess {
    // Creates a new process instance
    pub fn new(cpu_id: isize) -> Self {
        let mut the_sender: senderThread = senderThread::new(cpu_id);
        let mut the_receiver: receiverThread = receiverThread::new(cpu_id);
        let channel = mpsc::channel();
        // build connection: 
            the_sender.p = Some(channel.0);
        // build connection: 
            the_receiver.p = Some(channel.1);
        return Self { the_sender, the_receiver, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    pub fn start(self: Self) -> () {
        thread::Builder::new()
            .name("the_sender".to_string())
            .spawn(move || { self.the_sender.run() }).unwrap();
        thread::Builder::new()
            .name("the_receiver".to_string())
            .spawn(move || { self.the_receiver.run() }).unwrap();
        std::thread::spawn(|| {
            loop {
                let data = self.the_sender.p.recv();
                // build connection: 
                    self.the_receiver.p = data;
            };
        });
    }
    
}

// AADL Thread: sender
#[derive(Debug)]
pub struct senderThread {
    // Port: p Out
    pub p: Option<mpsc::Sender<i32>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub priority: u64, // AADL属性: Priority
    pub data_size: u64, // AADL属性: Data_Size
    pub stack_size: u64, // AADL属性: Stack_Size
    pub code_size: u64, // AADL属性: Code_Size
}

impl senderThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            p: None,
            cpu_id: cpu_id,
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 2000, // AADL属性: Period
            priority: 5, // AADL属性: Priority
            data_size: 40000, // AADL属性: Data_Size
            stack_size: 40000, // AADL属性: Stack_Size
            code_size: 40, // AADL属性: Code_Size
        }
    }
}
impl senderThread {
    // Thread execution entry point
    // Period: None ms
    pub fn run(mut self) -> () {
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(2000);
        loop {
            let start = Instant::now();
            {
                // --- 调用序列（等价 AADL 的 Wrapper）---
            // c();
                // c;
                if let Some(sender) = &self.p {
                    let mut val = 0;
                    sender_spg::send(&mut val);
                    sender.send(val).unwrap();
                };
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

pub mod sender_spg {
    // Auto-generated from AADL subprogram: sender_spg
    // C binding to: PingPong.Send
    // source_files: ""
    use super::{PingPong.Send};
    // Wrapper for C function PingPong.Send
    // Original AADL port: result
    pub fn send(result: &mut i32) -> () {
        unsafe { PingPong.Send(result);
         };
    }
    
}

// AADL Thread: receiver
#[derive(Debug)]
pub struct receiverThread {
    // Port: p In
    pub p: Option<mpsc::Receiver<i32>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub priority: u64, // AADL属性: Priority
    pub data_size: u64, // AADL属性: Data_Size
    pub stack_size: u64, // AADL属性: Stack_Size
    pub code_size: u64, // AADL属性: Code_Size
}

impl receiverThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            p: None,
            cpu_id: cpu_id,
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 1000, // AADL属性: Period
            priority: 10, // AADL属性: Priority
            data_size: 40000, // AADL属性: Data_Size
            stack_size: 40000, // AADL属性: Stack_Size
            code_size: 40, // AADL属性: Code_Size
        }
    }
}
impl receiverThread {
    // Thread execution entry point
    // Period: None ms
    pub fn run(mut self) -> () {
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(2000);
        loop {
            let start = Instant::now();
            {
                // --- 调用序列（等价 AADL 的 Wrapper）---
            // c();
                // c;
                if let Some(receiver) = &self.p {
                    match receiver.try_recv() {
                        Ok(val) => {
                            // 收到消息 → 调用处理函数
                            receiver_spg::receive(val);
                        },
                        Err(mpsc::TryRecvError::Empty) => {
                            // 没有消息，不阻塞，直接跳过
                        },
                        Err(mpsc::TryRecvError::Disconnected) => {
                            // 通道已关闭
                            eprintln!("channel closed");
                        },
                    };
                };
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

pub mod receiver_spg {
    // Auto-generated from AADL subprogram: receiver_spg
    // C binding to: PingPong.Receive
    // source_files: ""
    use super::{PingPong.Receive};
    // Wrapper for C function PingPong.Receive
    // Original AADL port: input
    pub fn receive(input: i32) -> () {
        unsafe { PingPong.Receive(input);
         };
    }
    
}

