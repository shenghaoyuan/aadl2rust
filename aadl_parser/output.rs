// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-06-19 19:07:08

#![allow(unused_imports)]
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Initialize proc.impl
pub  fn init_proc() -> () {
    let sender = SenderThread::new();
    let receiver = ReceiverThread::new();
    // Connect SubcomponentPort { subcomponent: "the_sender", port: "p" } to SubcomponentPort { subcomponent: "the_receiver", port: "p" };
    thread::spawn.unwrap(|sender| sender.run());
}

/// AADL Thread: sender
#[derive(Debug, Clone)]
pub struct senderThread {
    /// Port: p Out
    pub p: mpsc::Sender<i32>,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, /// AADL属性: Dispatch_Protocol
    pub period: i64, /// AADL属性: Period
    pub priority: i64, /// AADL属性: Priority
    pub data_size: i64, /// AADL属性: Data_Size
    pub stack_size: i64, /// AADL属性: Stack_Size
    pub code_size: i64, /// AADL属性: Code_Size
}

impl senderThread {
    /// 创建组件并初始化AADL属性
    pub fn new(p: mpsc::Sender<i32>) -> Self {
        Self {
            p,
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
    /// Thread execution entry point
    /// Period: 2000ms
    pub  async fn run(self: &mut  senderThread) -> () {
        let interval: tokio::time::Interval = tokio::time::interval(Duration::from_millis(2000));
        loop interval.tick();
        _.await;
        ;
    }
    
}

/// Initialize sender.impl
pub  fn init_sender() -> () {
    let sender = SenderThread::new();
    let receiver = ReceiverThread::new();
    thread::spawn.unwrap(|sender| sender.run());
}

/// Port handler for result
/// Direction: Out
pub  async fn handle_result(port: mpsc::Sender<()>) -> () {
    // Handle port: result;
}

/// AADL Thread: receiver
#[derive(Debug, Clone)]
pub struct receiverThread {
    /// Port: p In
    pub p: mpsc::Receiver<i32>,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, /// AADL属性: Dispatch_Protocol
    pub period: i64, /// AADL属性: Period
    pub priority: i64, /// AADL属性: Priority
    pub data_size: i64, /// AADL属性: Data_Size
    pub stack_size: i64, /// AADL属性: Stack_Size
    pub code_size: i64, /// AADL属性: Code_Size
}

impl receiverThread {
    /// 创建组件并初始化AADL属性
    pub fn new(p: mpsc::Receiver<i32>) -> Self {
        Self {
            p,
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
    /// Thread execution entry point
    /// Period: 1000ms
    pub  async fn run(self: &mut  receiverThread) -> () {
        let interval: tokio::time::Interval = tokio::time::interval(Duration::from_millis(1000));
        loop interval.tick();
        _.await;
        ;
    }
    
}

/// Initialize receiver.impl
pub  fn init_receiver() -> () {
    let sender = SenderThread::new();
    let receiver = ReceiverThread::new();
    thread::spawn.unwrap(|sender| sender.run());
}

/// Port handler for input
/// Direction: In
pub  async fn handle_input(port: mpsc::Receiver<()>) -> () {
    // Handle port: input;
}

/// AADL Data Type: Integer
pub type Integer = ();

