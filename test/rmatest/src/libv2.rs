// edit自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-08-14 13:52:09

#![allow(unused_imports)]
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use libc::{
    pthread_self, sched_param, pthread_setschedparam, SCHED_FIFO,
    cpu_set_t, CPU_SET, CPU_ZERO, sched_setaffinity,
};

include!(concat!(env!("OUT_DIR"), "/aadl_c_bindings.rs"));

pub mod hello_spg_1 {
    use super::user_hello_spg_1;
    pub fn execute() -> () {
        unsafe { user_hello_spg_1(); }
    }
}

pub mod hello_spg_2 {
    use super::user_hello_spg_2;
    pub fn execute() -> () {
        unsafe { user_hello_spg_2(); }
    }
}

// ---------------- cpu ----------------
fn set_thread_affinity(cpu: usize) {
    unsafe {
        let mut cpuset: cpu_set_t = std::mem::zeroed();
        CPU_ZERO(&mut cpuset);
        CPU_SET(cpu, &mut cpuset);
        sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset);
    }
}

// ---------------- AADL Thread: task ----------------
#[derive(Debug)]
pub struct taskThread {
    pub dispatch_protocol: String,
    pub priority: u64, // Linux 实时优先级 1~99
    pub period: u64,
    pub deadline: u64,
}

impl taskThread {
    pub fn new() -> Self {
        Self {
            dispatch_protocol: "Periodic".to_string(),
            priority: 1, // low优先级
            period: 1000,
            deadline: 1000,
        }
    }

    pub fn run(mut self) -> () {
        unsafe {
            let mut param = sched_param { sched_priority: self.priority as i32 };
            let ret = pthread_setschedparam(pthread_self(), SCHED_FIFO, &mut param);
            if ret != 0 {
                eprintln!("taskThread: Failed to set thread priority: {}", ret);
            }
        }

        set_thread_affinity(0); // 限制在 CPU 0

        let period = Duration::from_millis(self.period);
        loop {
            let start = Instant::now();
            hello_spg_1::execute();
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period - elapsed);
            }
        }
    }
}

// ---------------- AADL Thread: task2 ----------------
#[derive(Debug)]
pub struct task2Thread {
    pub dispatch_protocol: String,
    pub priority: u64,
    pub period: u64,
    pub deadline: u64,
}

impl task2Thread {
    pub fn new() -> Self {
        Self {
            dispatch_protocol: "Periodic".to_string(),
            priority: 2, // 高优先级
            period: 500,
            deadline: 500,
        }
    }

    pub fn run(mut self) -> () {
        unsafe {
            let mut param = sched_param { sched_priority: self.priority as i32 };
            let ret = pthread_setschedparam(pthread_self(), SCHED_FIFO, &mut param);
            if ret != 0 {
                eprintln!("task2Thread: Failed to set thread priority: {}", ret);
            }
        }

        set_thread_affinity(0); // 限制在 CPU 0

        let period = Duration::from_millis(self.period);
        loop {
            let start = Instant::now();
            hello_spg_2::execute();
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period - elapsed);
            }
        }
    }
}

// ---------------- Process implementation: node_a ----------------
#[derive(Debug)]
pub struct node_aProcess {
    #[allow(dead_code)]
    pub task1: taskThread,
    #[allow(dead_code)]
    pub task2: task2Thread,
}

impl node_aProcess {
    pub fn new() -> Self {
        Self {
            task1: taskThread::new(),
            task2: task2Thread::new(),
        }
    }

    pub fn start(self: Self) -> () {
        thread::Builder::new()
            .name("task1".to_string())
            .spawn(move || self.task1.run())
            .unwrap();
        thread::Builder::new()
            .name("task2".to_string())
            .spawn(move || self.task2.run())
            .unwrap();
    }
}
