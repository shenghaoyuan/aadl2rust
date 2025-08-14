// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-08-14 22:22:16

#![allow(unused_imports)]
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use libc::{
    pthread_self, sched_param, pthread_setschedparam, SCHED_FIFO,
    cpu_set_t, CPU_SET, CPU_ZERO, sched_setaffinity,
};
include!(concat!(env!("OUT_DIR"), "/c_bindings.rs"));

// ---------------- cpu ----------------
fn set_thread_affinity(cpu: usize) {
    unsafe {
        let mut cpuset: cpu_set_t = std::mem::zeroed();
        CPU_ZERO(&mut cpuset);
        CPU_SET(cpu, &mut cpuset);
        sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset);
    }
}

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
    // 新增 CPU ID
    pub cpu_id: usize,
    
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
            cpu_id: None,
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
    // 新增 CPU ID
    pub cpu_id: usize,
    
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
            cpu_id: None,
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
    // 新增 CPU ID
    pub cpu_id: usize,
}

impl node_aProcess {
    // Creates a new process instance
    pub fn new(cpu_id: usize) -> Self {
        let mut task1: taskThread = taskThread::new(cpu_id);
        let mut task2: task2Thread = task2Thread::new(cpu_id);
        return Self { task1, task2, cpu_id }  //显式return;
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

// AADL System: rma
#[derive(Debug)]
pub struct rmaSystem {
    // 进程和CPU的对应关系
    pub processes: Vec<(String, usize)>,
}

impl rmaSystem {
    // 创建系统实例
    pub fn new() -> Self {
        Self { processes: vec![("node_a".to_string(), 0)] };
    }
    
    // 运行系统，启动所有进程
    pub fn run(self: Self) -> () {
        // TODO: 遍历processes，为每个进程调用start函数并传递CPU参数;
    }
    
}

