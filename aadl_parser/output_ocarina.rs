// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-06-24 20:25:12

#![allow(unused_imports)]
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

// Process implementation: A
// Auto-generated from AADL
#[derive(Debug)]
pub struct aProcess {
    // Subcomponent: Pinger
    #[allow(dead_code)]
    pub pinger: pThread,
    // Subcomponent: Ping_Me
    #[allow(dead_code)]
    pub ping_me: qThread,
}

impl aProcess {
    // Creates a new process instance
    pub fn new() -> Self {
        let mut pinger: pThread = pThread::new();
        let mut ping_me: qThread = qThread::new();
        let channel = mpsc::channel();
        // build connection: 
            pinger.data_source = Some(channel.0);
        // build connection: 
            ping_me.data_sink = Some(channel.1);
        Self { pinger, ping_me };
    }
    
    // Starts all threads in the process
    pub fn start(self: &mut  Self) -> () {
        thread::Builder::new()
            .name("pinger".to_string())
            .stack_size(self.pinger.stack_size as usize)
            .spawn(move || { self.pinger.run() }).unwrap();
        thread::Builder::new()
            .name("ping_me".to_string())
            .stack_size(self.ping_me.stack_size as usize)
            .spawn(move || { self.ping_me.run() }).unwrap();
    }
    
}

// AADL Data Type: Simple_Type
pub type Simple_Type = ();

// Port handler for Data_Source
// Direction: Out
pub async fn handle_Data_Source(port: Option<mpsc::Sender<()>>) -> () {
    // Handle port: Data_Source;
}

// Port handler for Data_Sink
// Direction: In
pub async fn handle_Data_Sink(port: Option<mpsc::Receiver<()>>) -> () {
    // Handle port: Data_Sink;
}

// AADL Thread: p
#[derive(Debug, Clone)]
pub struct pThread {
    // Port: Data_Source Out
    pub data_source: Option<mpsc::Sender<Simple_Type>>,
    
    // --- AADL属性 ---
    pub recover_entrypoint_source_text: String, // AADL属性: Recover_Entrypoint_Source_Text
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub deadline: u64, // AADL属性: Deadline
    pub priority: u64, // AADL属性: Priority
    pub dispatch_offset: u64, // AADL属性: Dispatch_Offset
}

impl pThread {
    // 创建组件并初始化AADL属性
    pub fn new() -> Self {
        Self {
            data_source: None,
            recover_entrypoint_source_text: "recover".to_string(), // AADL属性: Recover_Entrypoint_Source_Text
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 2000, // AADL属性: Period
            deadline: 2000, // AADL属性: Deadline
            priority: 2, // AADL属性: Priority
            dispatch_offset: 500, // AADL属性: Dispatch_Offset
        }
    }
}
// AADL Thread: q
#[derive(Debug, Clone)]
pub struct qThread {
    // Port: Data_Sink In
    pub data_sink: Option<mpsc::Receiver<Simple_Type>>,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub deadline: u64, // AADL属性: deadline
    pub priority: u64, // AADL属性: Priority
}

impl qThread {
    // 创建组件并初始化AADL属性
    pub fn new() -> Self {
        Self {
            data_sink: None,
            dispatch_protocol: "Sporadic".to_string(), // AADL属性: Dispatch_Protocol
            period: 10, // AADL属性: Period
            deadline: 10, // AADL属性: deadline
            priority: 1, // AADL属性: Priority
        }
    }
}
