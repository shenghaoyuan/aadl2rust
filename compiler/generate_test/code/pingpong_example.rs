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

// AADL Process: pingpong_proc
#[derive(Debug)]
pub struct pingpong_procProcess {
    pub cpu_id: isize,// 进程 CPU ID
    pub ping_thr: ping_thrThread,// 子组件线程（ping_thr : thread ping_thr）
    pub pong_thr: pong_thrThread,// 子组件线程（pong_thr : thread pong_thr）
}

impl Process for pingpong_procProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let ping_thr: ping_thrThread = ping_thrThread::new(cpu_id);
        let pong_thr: pong_thrThread = pong_thrThread::new(cpu_id);
        let cnx = crossbeam_channel::unbounded();
        // build connection: 
            ping_thr.data_s = Some(cnx.0);
        // build connection: 
            pong_thr.data_r = Some(cnx.1);
        return Self { ping_thr, pong_thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn run(self: Self) -> () {
        let Self { ping_thr, pong_thr, cpu_id, .. } = self;
        thread::Builder::new()
            .name("ping_thr".to_string())
            .spawn(move || { ping_thr.run() }).unwrap();
        thread::Builder::new()
            .name("pong_thr".to_string())
            .spawn(move || { pong_thr.run() }).unwrap();
    }
    
}

// AADL System: PingPongSystem
#[derive(Debug)]
pub struct pingpongsystemSystem {
    pub pingpong_proc: pingpong_procProcess,// 子组件进程（pingpong_proc : process pingpong_proc）
}

impl System for pingpongsystemSystem {
    // Creates a new system instance
    fn new() -> Self {
        let mut pingpong_proc: pingpong_procProcess = pingpong_procProcess::new(0);
        return Self { pingpong_proc }  //显式return;
    }
    
    // Runs the system, starts all processes
    fn run(self: Self) -> () {
        self.pingpong_proc.run();
    }
    
}

pub mod ping_spg {
    // Auto-generated from AADL subprogram: Ping_Spg
    // C binding to: user_do_ping_spg
    // source_files: ping.c
    use super::{user_do_ping_spg};
    // Wrapper for C function user_do_ping_spg
    // Original AADL port: Data_Source
    pub fn send(data_source: &mut i32) -> () {
        unsafe { user_do_ping_spg(data_source);
         };
    }
    
}

pub mod pong_spg {
    // Auto-generated from AADL subprogram: Pong_Spg
    // C binding to: user_ping_spg
    // source_files: ping.c
    use super::{user_ping_spg};
    // Wrapper for C function user_ping_spg
    // Original AADL port: Data_Sink
    pub fn receive(data_sink: i32) -> () {
        unsafe { user_ping_spg(data_sink);
         };
    }
    
}

// AADL Thread: ping_thr
#[derive(Debug)]
pub struct ping_thrThread {
    pub data_s: Option<Sender<i32>>,// Port: data_s Out
    pub cpu_id: isize,// 结构体新增 CPU ID
    pub dispatch_protocol: String,// AADL属性(impl): Dispatch_Protocol
    pub period: u64,// AADL属性(impl): Period
    pub priority: u64,// AADL属性(impl): Priority
    pub deadline: u64,// AADL属性(impl): Deadline
}

impl Thread for ping_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            data_s: None, 
            dispatch_protocol: "Periodic".to_string(), 
            priority: 2, 
            period: 2000, 
            deadline: 2000, 
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
                eprintln!("ping_thrThread: Failed to set thread priority: {}", ret);
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
                if let Some(sender) = &self.data_s {
                    let mut val = 0;
                    ping_spg::send(&mut val);
                    sender.send(val).unwrap();
                };
            };
            next_release += period;
        };
    }
    
}

// AADL Thread: pong_thr
#[derive(Debug)]
pub struct pong_thrThread {
    pub data_r: Option<Receiver<i32>>,// Port: data_r In
    pub cpu_id: isize,// 结构体新增 CPU ID
    pub dispatch_protocol: String,// AADL属性(impl): Dispatch_Protocol
    pub period: u64,// AADL属性(impl): Period
    pub priority: u64,// AADL属性(impl): Priority
    pub deadline: u64,// AADL属性(impl): deadline
}

impl Thread for pong_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            dispatch_protocol: "Sporadic".to_string(), 
            priority: 1, 
            deadline: 10, 
            period: 10, 
            data_r: None, 
            cpu_id: cpu_id, // CPU ID
        };
    }
    
    // Thread execution entry point
    // Period: Some(10) ms
    fn run(mut self) -> () {
        unsafe {
            let mut param: sched_param = sched_param { sched_priority: 1 };
            let ret = pthread_setschedparam(pthread_self(), *CPU_ID_TO_SCHED_POLICY.get(&self.cpu_id).unwrap_or(&SCHED_FIFO), &mut param);
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
            if let Some((idx, (val, _urgency, _ts))) = events.iter().enumerate().max_by(|a, b| match a.1.1.cmp(&b.1.1) {
                        std::cmp::Ordering::Equal => b.1.2.cmp(&a.1.2),
                        other => other,
                    }) {
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

