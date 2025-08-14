// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-08-13 14:12:56

#![allow(unused_imports)]
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::{Duration, Instant};
include!(concat!(env!("OUT_DIR"), "/c_bindings.rs"));

pub mod hello_spg_1 {
    use super::user_hello_spg_1;
    pub fn execute() { unsafe { user_hello_spg_1(); } }
}

pub mod hello_spg_2 {
    use super::user_hello_spg_2;
    pub fn execute() { unsafe { user_hello_spg_2(); } }
}

#[derive(Debug)]
pub struct taskThread {
    pub priority: u64,
    pub period: u64,
}

impl taskThread {
    pub fn new() -> Self {
        Self { priority: 1, period: 1000 }
    }

    pub fn run(self, scheduler: Arc<(Mutex<String>, Condvar)>, period: Duration) {
        thread::spawn(move || {
            loop {
                let (lock, cvar) = &*scheduler;
                let mut current = lock.lock().unwrap();
                while *current != "task1" {
                    current = cvar.wait(current).unwrap();
                }
                drop(current);

                println!("{:?} FIRST TASK", Instant::now());
                hello_spg_1::execute();
                thread::sleep(period);
            }
        });
    }
}

#[derive(Debug)]
pub struct task2Thread {
    pub priority: u64,
    pub period: u64,
}

impl task2Thread {
    pub fn new() -> Self {
        Self { priority: 2, period: 500 }
    }

    pub fn run(self, scheduler: Arc<(Mutex<String>, Condvar)>, period: Duration) {
        thread::spawn(move || {
            loop {
                let (lock, cvar) = &*scheduler;
                let mut current = lock.lock().unwrap();
                while *current != "task2" {
                    current = cvar.wait(current).unwrap();
                }
                drop(current);

                println!("{:?} SECOND TASK", Instant::now());
                hello_spg_2::execute();
                thread::sleep(period);
            }
        });
    }
}

#[derive(Debug)]
pub struct node_aProcess {
    pub task1: taskThread,
    pub task2: task2Thread,
}

impl node_aProcess {
    pub fn new() -> Self {
        Self { task1: taskThread::new(), task2: task2Thread::new() }
    }

    pub fn start(self) {
        let scheduler = Arc::new((Mutex::new("task2".to_string()), Condvar::new()));

        // 调度器线程（高优先级任务先运行）
        {
            let sched_clone = Arc::clone(&scheduler);
            thread::spawn(move || {
                loop {
                    {
                        let (lock, cvar) = &*sched_clone;
                        let mut current = lock.lock().unwrap();
                        *current = "task2".to_string();
                        cvar.notify_all();
                    }
                    thread::sleep(Duration::from_millis(500));

                    {
                        let (lock, cvar) = &*sched_clone;
                        let mut current = lock.lock().unwrap();
                        *current = "task1".to_string();
                        cvar.notify_all();
                    }
                    thread::sleep(Duration::from_millis(500));
                }
            });
        }

        // 拆分任务，避免 self 移动问题
        let task1 = self.task1;
        let task2 = self.task2;

        // 启动任务线程
        task1.run(Arc::clone(&scheduler), Duration::from_millis(task1.period));
        task2.run(Arc::clone(&scheduler), Duration::from_millis(task2.period));
    }
}

fn main() {
    let proc = node_aProcess::new();
    proc.start();
    loop { thread::sleep(Duration::from_secs(1)); }
}
