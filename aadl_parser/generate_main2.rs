// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-08-13 14:24:24

#![allow(unused_imports)]
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
include!(concat!(env!("OUT_DIR"), "/c_bindings.rs"));

pub mod hello_spg_1 {
    // Auto-generated from AADL subprogram: Hello_Spg_1
    // C binding to: user_hello_spg_1
    // source_files: "hello.c"
    use super::{user_hello_spg_1};
    // Direct execution wrapper for C function user_hello_spg_1
    // This component has no communication ports
    pub fn execute() -> () {
        unsafe { user_hello_spg_1();
         };
    }
    
}

pub mod hello_spg_2 {
    // Auto-generated from AADL subprogram: Hello_Spg_2
    // C binding to: user_hello_spg_2
    // source_files: "hello.c"
    use super::{user_hello_spg_2};
    // Direct execution wrapper for C function user_hello_spg_2
    // This component has no communication ports
    pub fn execute() -> () {
        unsafe { user_hello_spg_2();
         };
    }
    
}

// AADL Thread: task
#[derive(Debug)]
pub struct taskThread {
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub priority: u64, // AADL属性: Priority
    pub period: u64, // AADL属性: Period
    pub deadline: u64, // AADL属性: Deadline
}

impl taskThread {
    // 创建组件并初始化AADL属性
    pub fn new() -> Self {
        Self {
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            priority: 1, // AADL属性: Priority
            period: 1000, // AADL属性: Period
            deadline: 1000, // AADL属性: Deadline
        }
    }
}
impl taskThread {
    // Thread execution entry point
    // Period: Some(1000) ms
    pub fn run(mut self) -> () {
        let period: std::time::Duration = Duration::from_millis(1000);
        loop {
            let start = Instant::now();
            {
                hello_spg_1::execute();
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

// AADL Thread: task2
#[derive(Debug)]
pub struct task2Thread {
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub priority: u64, // AADL属性: Priority
    pub period: u64, // AADL属性: Period
    pub deadline: u64, // AADL属性: Deadline
}

impl task2Thread {
    // 创建组件并初始化AADL属性
    pub fn new() -> Self {
        Self {
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            priority: 2, // AADL属性: Priority
            period: 500, // AADL属性: Period
            deadline: 500, // AADL属性: Deadline
        }
    }
}
impl task2Thread {
    // Thread execution entry point
    // Period: Some(500) ms
    pub fn run(mut self) -> () {
        let period: std::time::Duration = Duration::from_millis(500);
        loop {
            let start = Instant::now();
            {
                hello_spg_2::execute();
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

// Process implementation: node_a
// Auto-generated from AADL
#[derive(Debug)]
pub struct node_aProcess {
    // Subcomponent: Task1
    #[allow(dead_code)]
    pub task1: taskThread,
    // Subcomponent: Task2
    #[allow(dead_code)]
    pub task2: task2Thread,
}

impl node_aProcess {
    // Creates a new process instance
    pub fn new() -> Self {
        let mut task1: taskThread = taskThread::new();
        let mut task2: task2Thread = task2Thread::new();
        return Self { task1, task2 }  //显式return;
    }
    
    // Starts all threads in the process
    pub fn start(self: Self) -> () {
        thread::Builder::new()
            .name("task1".to_string())
            .spawn(move || { self.task1.run() }).unwrap();
        thread::Builder::new()
            .name("task2".to_string())
            .spawn(move || { self.task2.run() }).unwrap();
    }
    
}

