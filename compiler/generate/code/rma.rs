// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-09-16 20:05:01

#![allow(unused_imports)]
use std::sync::{mpsc, Arc};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use std::collections::HashMap;
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

pub mod hello_spg_1 {
    // Auto-generated from AADL subprogram: Hello_Spg_1
    // C binding to: user_hello_spg_1
    // source_files: hello.c
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
    // source_files: hello.c
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
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub priority: u64, // AADL属性: Priority
    pub period: u64, // AADL属性: Period
    pub deadline: u64, // AADL属性: Deadline
}

impl taskThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            cpu_id: cpu_id,
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
        unsafe {
            let mut param: sched_param = sched_param { sched_priority: 1 };
            let ret = pthread_setschedparam(pthread_self(), *CPU_ID_TO_SCHED_POLICY.get(&self.cpu_id).unwrap_or(&SCHED_FIFO), &mut param);
            if ret != 0 {
                eprintln!("taskThread: Failed to set thread priority: {}", ret);
            };
        };
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(1000);
        loop {
            let start = Instant::now();
            {
                // --- 调用序列（等价 AADL 的 Wrapper）---
                           // P_Spg();
                // P_Spg;
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
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub priority: u64, // AADL属性: Priority
    pub period: u64, // AADL属性: Period
    pub deadline: u64, // AADL属性: Deadline
}

impl task2Thread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            cpu_id: cpu_id,
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
        unsafe {
            let mut param: sched_param = sched_param { sched_priority: 2 };
            let ret = pthread_setschedparam(pthread_self(), *CPU_ID_TO_SCHED_POLICY.get(&self.cpu_id).unwrap_or(&SCHED_FIFO), &mut param);
            if ret != 0 {
                eprintln!("task2Thread: Failed to set thread priority: {}", ret);
            };
        };
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(500);
        loop {
            let start = Instant::now();
            {
                // --- 调用序列（等价 AADL 的 Wrapper）---
                           // P_Spg();
                // P_Spg;
                hello_spg_2::execute();
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

// AADL Process: node_a
#[derive(Debug)]
pub struct node_aProcess {
    // 进程 CPU ID
    pub cpu_id: isize,
    // 子组件线程（Task1 : thread Task）
    #[allow(dead_code)]
    pub task1: taskThread,
    // 子组件线程（Task2 : thread Task2）
    #[allow(dead_code)]
    pub task2: task2Thread,
}

impl node_aProcess {
    // Creates a new process instance
    pub fn new(cpu_id: isize) -> Self {
        let mut task1: taskThread = taskThread::new(cpu_id);
        let mut task2: task2Thread = task2Thread::new(cpu_id);
        return Self { task1, task2, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    pub fn start(self: Self) -> () {
        let Self { task1, task2, cpu_id, .. } = self;
        thread::Builder::new()
            .name("task1".to_string())
            .spawn(|| { task1.run() }).unwrap();
        thread::Builder::new()
            .name("task2".to_string())
            .spawn(|| { task2.run() }).unwrap();
    }
    
}

// AADL System: rma
#[derive(Debug)]
pub struct rmaSystem {
    // 子组件进程（node_a : process node_a）
    #[allow(dead_code)]
    pub node_a: node_aProcess,
}

impl rmaSystem {
    // Creates a new system instance
    pub fn new() -> Self {
        let mut node_a: node_aProcess = node_aProcess::new(0);
        return Self { node_a }  //显式return;
    }
    
    // Runs the system, starts all processes
    pub fn run(self: Self) -> () {
        self.node_a.start();
    }
    
}

// CPU ID到调度策略的映射
lazy_static! {
    static ref CPU_ID_TO_SCHED_POLICY: HashMap<isize, i32> = {
        let mut map: HashMap<isize, i32> = HashMap::new();
        map.insert(0, SCHED_FIFO);
        return map;
    };
}

