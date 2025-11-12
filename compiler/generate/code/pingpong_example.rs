// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-10-26 14:42:31

#![allow(unused_imports)]
use crossbeam_channel::{Receiver, Sender};
use lazy_static::lazy_static;
use libc::{
    cpu_set_t, pthread_self, pthread_setschedparam, sched_param, sched_setaffinity, CPU_SET,
    CPU_ZERO, SCHED_FIFO,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
include!(concat!(env!("OUT_DIR"), "/aadl_c_bindings.rs"));

// ---------------- cpu ----------------
// 通过 POSIX API 把当前线程绑定到指定的 CPU 核心上执行。
fn set_thread_affinity(cpu: isize) {
    unsafe {
        let mut cpuset: cpu_set_t = std::mem::zeroed();
        CPU_ZERO(&mut cpuset);
        CPU_SET(cpu as usize, &mut cpuset);
        sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset);
    }
}

// ---------------- System ----------------
pub trait System {
    fn new() -> Self
    where
        Self: Sized;
    fn run(self);
}

// ---------------- Process ----------------
pub trait Process {
    fn new(cpu_id: isize) -> Self
    where
        Self: Sized;
    fn start(self);
}

// ---------------- Thread ----------------
pub trait Thread {
    fn new(cpu_id: isize) -> Self
    where
        Self: Sized;
    fn run(self);
}

// AADL Process: pingpong_proc
#[derive(Debug)]
pub struct pingpong_procProcess {
    pub cpu_id: isize, // 进程 CPU ID
    #[allow(dead_code)]
    pub pinger: ping_thrThread, // 子组件线程（Pinger : thread ping_thr）
    #[allow(dead_code)]
    pub ping_me: pong_thrThread, // 子组件线程（Ping_Me : thread pong_thr）
}

impl Process for pingpong_procProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut pinger: ping_thrThread = ping_thrThread::new(cpu_id);
        let mut ping_me: pong_thrThread = pong_thrThread::new(cpu_id);
        let channel = crossbeam_channel::unbounded();
        // build connection:
        pinger.data_s = Some(channel.0);
        // build connection:
        ping_me.data_r = Some(channel.1);
        return Self {
            pinger,
            ping_me,
            cpu_id,
        }; //显式return;
    }

    // Starts all threads in the process
    fn start(self: Self) -> () {
        let Self {
            pinger,
            ping_me,
            cpu_id,
            ..
        } = self;
        thread::Builder::new()
            .name("pinger".to_string())
            .spawn(|| pinger.run())
            .unwrap();
        thread::Builder::new()
            .name("ping_me".to_string())
            .spawn(|| ping_me.run())
            .unwrap();
    }
}

// AADL System: PingPongSystem
#[derive(Debug)]
pub struct pingpongsystemSystem {
    #[allow(dead_code)]
    pub proc: pingpong_procProcess, // 子组件进程（proc : process pingpong_proc）
}

impl System for pingpongsystemSystem {
    // Creates a new system instance
    fn new() -> Self {
        let mut proc: pingpong_procProcess = pingpong_procProcess::new(0);
        return Self { proc }; //显式return;
    }

    // Runs the system, starts all processes
    fn run(self: Self) -> () {
        self.proc.start();
    }
}

pub mod ping_spg {
    // Auto-generated from AADL subprogram: Ping_Spg
    // C binding to: user_do_ping_spg
    // source_files: ping.c
    use super::user_do_ping_spg;
    // Wrapper for C function user_do_ping_spg
    // Original AADL port: Data_Source
    pub fn send(data_source: &mut i32) -> () {
        unsafe {
            user_do_ping_spg(data_source);
        };
    }
}

pub mod pong_spg {
    // Auto-generated from AADL subprogram: Pong_Spg
    // C binding to: user_ping_spg
    // source_files: ping.c
    use super::user_ping_spg;
    // Wrapper for C function user_ping_spg
    // Original AADL port: Data_Sink
    pub fn receive(data_sink: i32) -> () {
        unsafe {
            user_ping_spg(data_sink);
        };
    }
}

// AADL Thread: ping_thr
#[derive(Debug)]
pub struct ping_thrThread {
    pub data_s: Option<Sender<i32>>, // Port: data_s Out
    pub cpu_id: isize,               // 结构体新增 CPU ID
    pub dispatch_protocol: String,   // AADL属性(impl): Dispatch_Protocol
    pub period: u64,                 // AADL属性(impl): Period
    pub priority: u64,               // AADL属性(impl): Priority
    pub deadline: u64,               // AADL属性(impl): Deadline
}

impl Thread for ping_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            priority: 2,
            deadline: 2000,
            period: 2000,
            dispatch_protocol: "Periodic".to_string(),
            data_s: None,
            cpu_id: cpu_id, // CPU ID
        };
    }

    // Thread execution entry point
    // Period: Some(2000) ms
    fn run(mut self) -> () {
        let mut param: sched_param = sched_param { sched_priority: 2 };
        let ret: i32;
        unsafe {
            ret = pthread_setschedparam(
                pthread_self(),
                *CPU_ID_TO_SCHED_POLICY
                    .get(&self.cpu_id)
                    .unwrap_or(&SCHED_FIFO),
                &mut param,
            );
        };
        if ret != 0 {
            eprintln!("ping_thrThread: Failed to set thread priority: {}", ret);
        };
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(2000);
        loop {
            let start = Instant::now();
            {
                // --- 调用序列（等价 AADL 的 Wrapper）---
                // p_spg();
                // p_spg;
                if let Some(sender) = &self.data_s {
                    let mut val = 0;
                    ping_spg::send(&mut val);
                    sender.send(val).unwrap();
                };
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        }
    }
}

// AADL Thread: pong_thr
#[derive(Debug)]
pub struct pong_thrThread {
    pub data_r: Option<Receiver<i32>>, // Port: data_r In
    pub cpu_id: isize,                 // 结构体新增 CPU ID
    pub dispatch_protocol: String,     // AADL属性(impl): Dispatch_Protocol
    pub period: u64,                   // AADL属性(impl): Period
    pub priority: u64,                 // AADL属性(impl): Priority
    pub deadline: u64,                 // AADL属性(impl): deadline
}

impl Thread for pong_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            dispatch_protocol: "Sporadic".to_string(),
            deadline: 10,
            period: 10,
            priority: 1,
            data_r: None,
            cpu_id: cpu_id, // CPU ID
        };
    }

    // Thread execution entry point
    // Period: Some(10) ms
    fn run(mut self) -> () {
        unsafe {
            let mut param: sched_param = sched_param { sched_priority: 1 };
            let ret = pthread_setschedparam(
                pthread_self(),
                *CPU_ID_TO_SCHED_POLICY
                    .get(&self.cpu_id)
                    .unwrap_or(&SCHED_FIFO),
                &mut param,
            );
            if ret != 0 {
                eprintln!("pong_thrThread: Failed to set thread priority: {}", ret);
            };
        };
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let min_interarrival: std::time::Duration = Duration::from_millis(10);
        let mut last_dispatch: std::time::Instant = Instant::now();
        let mut events = Vec::new();
        loop {
            if events.is_empty() {
                if let Some(rx) = &self.data_r {
                    if let Ok(val) = rx.try_recv() {
                        let ts = Instant::now();
                        events.push(((val, 0, ts)));
                    };
                };
            };
            if let Some((idx, (val, _urgency, _ts))) =
                events
                    .iter()
                    .enumerate()
                    .max_by(|a, b| match a.1 .1.cmp(&b.1 .1) {
                        std::cmp::Ordering::Equal => b.1 .2.cmp(&a.1 .2),
                        other => other,
                    })
            {
                let (val, _, _) = events.remove(idx);
                let now = Instant::now();
                let elapsed = now.duration_since(last_dispatch);
                if elapsed < min_interarrival {
                    std::thread::sleep(min_interarrival - elapsed);
                };
                {
                    // --- 调用序列（等价 AADL 的 Wrapper）---
                    // q_spg();
                    // q_spg;
                    pong_spg::receive(val);
                };
                last_dispatch = Instant::now();
            } else {
                std::thread::sleep(Duration::from_millis(1));
            };
        }
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
