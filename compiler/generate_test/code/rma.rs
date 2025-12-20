// Auto-generated from AADL package: rmaaadl
// 生成时间: 2025-12-20 17:31:23

#![allow(unused_imports)]
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc,Mutex};
use std::thread;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::common_traits::*;
use tokio::sync::broadcast::{self,Sender as BcSender, Receiver as BcReceiver};
use libc::{self, syscall, SYS_gettid};
use rand::{Rng};
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
    pub cpu_id: isize,// 结构体新增 CPU ID
    pub dispatch_protocol: String,// AADL属性(impl): Dispatch_Protocol
    pub priority: u64,// AADL属性(impl): Priority
    pub period: u64,// AADL属性(impl): Period
    pub deadline: u64,// AADL属性(impl): Deadline
}

impl Thread for taskThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            period: 1000, 
            dispatch_protocol: "Periodic".to_string(), 
            priority: 1, 
            deadline: 1000, 
            cpu_id: cpu_id, // CPU ID
        };
    }
    
    // Thread execution entry point
    // Period: Some(1000) ms
    fn run(mut self) -> () {
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
        let mut next_release = Instant::now() + period;
        loop {
            let now = Instant::now();
            if now < next_release {
                std::thread::sleep(next_release - now);
            };
            {
                // --- 调用序列（等价 AADL 的 Wrapper）---
                           // p_spg();
                // p_spg;
                hello_spg_1::execute();
            };
            next_release += period;
        };
    }
    
}

// AADL Thread: task2
#[derive(Debug)]
pub struct task2Thread {
    pub cpu_id: isize,// 结构体新增 CPU ID
    pub dispatch_protocol: String,// AADL属性(impl): Dispatch_Protocol
    pub priority: u64,// AADL属性(impl): Priority
    pub period: u64,// AADL属性(impl): Period
    pub deadline: u64,// AADL属性(impl): Deadline
}

impl Thread for task2Thread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            period: 500, 
            priority: 2, 
            dispatch_protocol: "Periodic".to_string(), 
            deadline: 500, 
            cpu_id: cpu_id, // CPU ID
        };
    }
    
    // Thread execution entry point
    // Period: Some(500) ms
    fn run(mut self) -> () {
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
        let mut next_release = Instant::now() + period;
        loop {
            let now = Instant::now();
            if now < next_release {
                std::thread::sleep(next_release - now);
            };
            {
                // --- 调用序列（等价 AADL 的 Wrapper）---
                           // p_spg();
                // p_spg;
                hello_spg_2::execute();
            };
            next_release += period;
        };
    }
    
}

// AADL Process: node_a
#[derive(Debug)]
pub struct node_aProcess {
    pub cpu_id: isize,// 进程 CPU ID
    pub task1: taskThread,// 子组件线程（Task1 : thread Task）
    pub task2: task2Thread,// 子组件线程（Task2 : thread Task2）
}

impl Process for node_aProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let task1: taskThread = taskThread::new(cpu_id);
        let task2: task2Thread = task2Thread::new(cpu_id);
        return Self { task1, task2, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn run(self: Self) -> () {
        let Self { task1, task2, cpu_id, .. } = self;
        thread::Builder::new()
            .name("task1".to_string())
            .spawn(move || { task1.run() }).unwrap();
        thread::Builder::new()
            .name("task2".to_string())
            .spawn(move || { task2.run() }).unwrap();
    }
    
}

// AADL System: rma
#[derive(Debug)]
pub struct rmaSystem {
    pub node_a: node_aProcess,// 子组件进程（node_a : process node_a）
}

impl System for rmaSystem {
    // Creates a new system instance
    fn new() -> Self {
        let mut node_a: node_aProcess = node_aProcess::new(-1);
        return Self { node_a }  //显式return;
    }
    
    // Runs the system, starts all processes
    fn run(self: Self) -> () {
        self.node_a.run();
    }
    
}

// CPU ID到调度策略的映射
lazy_static! {
    static ref CPU_ID_TO_SCHED_POLICY: HashMap<isize, i32> = {
        let mut map: HashMap<isize, i32> = HashMap::new();
        map.insert(1, SCHED_FIFO);
        map.insert(0, SCHED_FIFO);
        return map;
    };
}

