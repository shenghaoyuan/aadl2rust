// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-06-23 21:08:43

#![allow(unused_imports)]
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

// Process implementation: proc
// Auto-generated from AADL
#[derive(Debug)]
pub struct procProcess {
    // Subcomponent: the_sender
    #[allow(dead_code)]
    pub the_sender: senderThread,
    // Subcomponent: the_receiver
    #[allow(dead_code)]
    pub the_receiver: receiverThread,
}

impl procProcess {
    // Creates a new process instance
    pub fn new() -> Self {
        let mut the_sender: senderThread = senderThread::new();
        let mut the_receiver: receiverThread = receiverThread::new();
        let channel = mpsc::channel();
        // bulid connection: 
            the_sender.p = Some(channel.0);
        // bulid connection: 
            the_receiver.p = Some(channel.1);
        Self { the_sender, the_receiver };
    }
    
    // Starts all threads in the process
    pub fn start(self: &mut  Self) -> () {
        thread::Builder::new()
            .name("the_sender".to_string())
            .stack_size(self.the_sender.stack_size as usize)
            .spawn(move || { self.the_sender.run() }).unwrap();
        thread::Builder::new()
            .name("the_receiver".to_string())
            .stack_size(self.the_receiver.stack_size as usize)
            .spawn(move || { self.the_receiver.run() }).unwrap();
    }
    
}

// AADL Thread: sender
#[derive(Debug, Clone)]
pub struct senderThread {
    // Port: p Out
    pub p: Option<mpsc::Sender<i32>>,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: i64, // AADL属性: Period
    pub priority: i64, // AADL属性: Priority
    pub data_size: i64, // AADL属性: Data_Size
    pub stack_size: i64, // AADL属性: Stack_Size
    pub code_size: i64, // AADL属性: Code_Size
}

impl senderThread {
    // 创建组件并初始化AADL属性
    pub fn new() -> Self {
        Self {
            p: None,
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
    // Period: 2000ms
    pub async fn run(self: &mut  senderThread) -> () {
        let interval: tokio::time::Interval = tokio::time::interval(Duration::from_millis(2000));
        loop interval.tick();
        _.await;
        ;
    }
    
}

// Port handler for result
// Direction: Out
pub async fn handle_result(port: Option<mpsc::Sender<()>>) -> () {
    // Handle port: result;
}

// AADL Thread: receiver
#[derive(Debug, Clone)]
pub struct receiverThread {
    // Port: p In
    pub p: Option<mpsc::Receiver<i32>>,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: i64, // AADL属性: Period
    pub priority: i64, // AADL属性: Priority
    pub data_size: i64, // AADL属性: Data_Size
    pub stack_size: i64, // AADL属性: Stack_Size
    pub code_size: i64, // AADL属性: Code_Size
}

impl receiverThread {
    // 创建组件并初始化AADL属性
    pub fn new() -> Self {
        Self {
            p: None,
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
    // Period: 1000ms
    pub async fn run(self: &mut  receiverThread) -> () {
        let interval: tokio::time::Interval = tokio::time::interval(Duration::from_millis(1000));
        loop interval.tick();
        _.await;
        ;
    }
    
}

// Port handler for input
// Direction: In
pub async fn handle_input(port: Option<mpsc::Receiver<()>>) -> () {
    // Handle port: input;
}

// AADL Data Type: Integer
pub type Integer = ();

