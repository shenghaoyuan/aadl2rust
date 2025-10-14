// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-10-14 22:06:50

#![allow(unused_imports)]
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc,Mutex};
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

// ---------------- System ----------------
pub trait System {
    fn new() -> Self
        where Self: Sized;
    fn run(self);
}

// ---------------- Process ----------------
pub trait Process {
    fn new(cpu_id: isize) -> Self
        where Self: Sized;
    fn start(self);
}

// ---------------- Thread ----------------
pub trait Thread {
    fn new(cpu_id: isize) -> Self
        where Self: Sized;
    fn run(self);
}

// AADL Process: a
#[derive(Debug)]
pub struct aProcess {
    pub cpu_id: isize,// 进程 CPU ID
    #[allow(dead_code)]
    pub pinger: pThread,// 子组件线程（Pinger : thread P）
    #[allow(dead_code)]
    pub ping_me: qThread,// 子组件线程（Ping_Me : thread Q）
}

impl Process for aProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut pinger: pThread = pThread::new(cpu_id);
        let mut ping_me: qThread = qThread::new(cpu_id);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            pinger.data_source = Some(channel.0);
        // build connection: 
            ping_me.data_sink = Some(channel.1);
        return Self { pinger, ping_me, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn start(self: Self) -> () {
        let Self { pinger, ping_me, cpu_id, .. } = self;
        thread::Builder::new()
            .name("pinger".to_string())
            .spawn(|| { pinger.run() }).unwrap();
        thread::Builder::new()
            .name("ping_me".to_string())
            .spawn(|| { ping_me.run() }).unwrap();
    }
    
}

// AADL System: PING
#[derive(Debug)]
pub struct pingSystem {
    #[allow(dead_code)]
    pub node_a: aProcess,// 子组件进程（Node_A : process A）
}

impl System for pingSystem {
    // Creates a new system instance
    fn new() -> Self {
        let mut node_a: aProcess = aProcess::new(0);
        return Self { node_a }  //显式return;
    }
    
    // Runs the system, starts all processes
    fn run(self: Self) -> () {
        self.node_a.start();
    }
    
}

// AADL Data Type: Simple_Type
pub type Simple_Type = custom_int;

pub mod do_ping_spg {
    // Auto-generated from AADL subprogram: Do_Ping_Spg
    // C binding to: user_do_ping_spg
    // source_files: ping.c
    use super::{user_do_ping_spg, custom_int};
    // Wrapper for C function user_do_ping_spg
    // Original AADL port: Data_Source
    pub fn send(data_source: &mut custom_int) -> () {
        unsafe { user_do_ping_spg(data_source);
         };
    }
    
}

pub mod ping_spg {
    // Auto-generated from AADL subprogram: Ping_Spg
    // C binding to: user_ping_spg
    // source_files: ping.c
    use super::{user_ping_spg, custom_int};
    // Wrapper for C function user_ping_spg
    // Original AADL port: Data_Sink
    pub fn receive(data_sink: custom_int) -> () {
        unsafe { user_ping_spg(data_sink);
         };
    }
    
}

// AADL Thread: p
#[derive(Debug)]
pub struct pThread {
    pub data_source: Option<Sender<custom_int>>,// Port: Data_Source Out
    pub cpu_id: isize,// 结构体新增 CPU ID
    pub recover_entrypoint_source_text: String,// AADL属性(impl): Recover_Entrypoint_Source_Text
    pub dispatch_protocol: String,// AADL属性(impl): Dispatch_Protocol
    pub period: u64,// AADL属性(impl): Period
    pub deadline: u64,// AADL属性(impl): Deadline
    pub priority: u64,// AADL属性(impl): Priority
    pub dispatch_offset: u64,// AADL属性(impl): Dispatch_Offset
}

impl Thread for pThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            data_source: None, 
            recover_entrypoint_source_text: "recover".to_string(), 
            dispatch_protocol: "Periodic".to_string(), 
            deadline: 2000, 
            dispatch_offset: 500, 
            period: 2000, 
            priority: 2, 
            cpu_id: cpu_id, // CPU ID
        };
    }
    
    // Thread execution entry point
    // Period: Some(2000) ms
    fn run(mut self) -> () {
        unsafe {
            let mut param: sched_param = sched_param { sched_priority: 2 };
            let ret = pthread_setschedparam(pthread_self(), *CPU_ID_TO_SCHED_POLICY.get(&self.cpu_id).unwrap_or(&SCHED_FIFO), &mut param);
            if ret != 0 {
                eprintln!("pThread: Failed to set thread priority: {}", ret);
            };
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
                if let Some(sender) = &self.data_source {
                    let mut val = 0;
                    do_ping_spg::send(&mut val);
                    sender.send(val).unwrap();
                };
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

// AADL Thread: q
#[derive(Debug)]
pub struct qThread {
    pub data_sink: Option<Receiver<custom_int>>,// Port: Data_Sink In
    pub cpu_id: isize,// 结构体新增 CPU ID
    pub dispatch_protocol: String,// AADL属性(impl): Dispatch_Protocol
    pub period: u64,// AADL属性(impl): Period
}

impl Thread for qThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            period: 1000, 
            data_sink: None, 
            dispatch_protocol: "Timed".to_string(), 
            cpu_id: cpu_id, // CPU ID
        };
    }
    
    // Thread execution entry point
    // Period: Some(1000) ms
    fn run(mut self) -> () {
        unsafe {
            let prio = period_to_priority(self.period as f64);
            let mut param: sched_param = sched_param { sched_priority: prio };
            let ret = pthread_setschedparam(pthread_self(), *CPU_ID_TO_SCHED_POLICY.get(&self.cpu_id).unwrap_or(&SCHED_FIFO), &mut param);
            if ret != 0 {
                eprintln!("qThread: Failed to set thread priority from period: {}", ret);
            };
        };
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(1000);
        let mut start_time: std::time::Instant = Instant::now();
        let mut events = Vec::new();
        loop {
            if events.is_empty() {
                if let Some(rx) = &self.data_sink {
                    if let Ok(val) = rx.try_recv() {
                        let ts = Instant::now();
                        events.push(((val, 0, ts)));
                    };
                };
            };
            if let Some((idx, (val, _urgency, _ts))) = events.iter().enumerate().max_by(|a, b| match a.1.1.cmp(&b.1.1) {
                        std::cmp::Ordering::Equal => b.1.2.cmp(&a.1.2),
                        other => other,
                    }) {
                let (val, _, _) = events.remove(idx);
                {
                    // --- 调用序列（等价 AADL 的 Wrapper）---
                           // q_spg();
                    // q_spg;
                    ping_spg::receive(val);
                };
            } else {
                let now = Instant::now();
                let elapsed = now.duration_since(start_time);
                if elapsed > period {
                    eprintln!("qThread: timeout dispatch → Recover_Entrypoint");
                    // recover_entrypoint();;
                };
            };
        };
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

