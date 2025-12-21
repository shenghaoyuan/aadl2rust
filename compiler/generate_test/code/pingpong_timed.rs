// Auto-generated from AADL package: ping_local
// 生成时间: 2025-12-21 19:44:32

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

// AADL Process: a
#[derive(Debug)]
pub struct aProcess {
    pub cpu_id: isize,// 进程 CPU ID
    pub pinger: pThread,// 子组件线程（Pinger : thread P）
    pub ping_me: qThread,// 子组件线程（Ping_Me : thread Q）
}

impl Process for aProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let pinger: pThread = pThread::new(cpu_id);
        let ping_me: qThread = qThread::new(cpu_id);
        let cnx = crossbeam_channel::unbounded();
        // build connection: 
            pinger.data_source = Some(cnx.0);
        // build connection: 
            ping_me.data_sink = Some(cnx.1);
        return Self { pinger, ping_me, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn run(self: Self) -> () {
        let Self { pinger, ping_me, cpu_id, .. } = self;
        thread::Builder::new()
            .name("pinger".to_string())
            .spawn(move || { pinger.run() }).unwrap();
        thread::Builder::new()
            .name("ping_me".to_string())
            .spawn(move || { ping_me.run() }).unwrap();
    }
    
}

// AADL System: PING
#[derive(Debug)]
pub struct pingSystem {
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
        self.node_a.run();
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
            recover_entrypoint_source_text: "recover".to_string(), 
            priority: 2, 
            deadline: 2000, 
            dispatch_offset: 500, 
            period: 2000, 
            dispatch_protocol: "Periodic".to_string(), 
            data_source: None, 
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
                if let Some(sender) = &self.data_source {
                    let mut val = 0;
                    do_ping_spg::send(&mut val);
                    sender.send(val).unwrap();
                };
            };
            next_release += period;
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
            dispatch_protocol: "Timed".to_string(), 
            data_sink: None, 
            period: 1000, 
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
        map.insert(1, SCHED_FIFO);
        map.insert(2, SCHED_FIFO);
        map.insert(3, SCHED_FIFO);
        return map;
    };
}

// prio(P)=max(1,min(99,99−⌊k⋅log10(P)⌋))
// 根据周期计算优先级，周期越短优先级越高
// 用于 RMS (Rate Monotonic Scheduling) 和 DMS (Deadline Monotonic Scheduling)
pub fn period_to_priority(period_ms: f64) -> i32 {
    let k: f64 = 10.0;
    let raw: f64 = 99.0 - k * period_ms.log10().floor();
    return raw.max(1.0).min(99.0) as i32;
}

