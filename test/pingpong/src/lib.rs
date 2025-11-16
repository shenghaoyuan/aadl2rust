// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-10-14 21:06:54

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

// AADL Process: a
#[derive(Debug)]
pub struct aProcess {
    pub cpu_id: isize, // 进程 CPU ID
    #[allow(dead_code)]
    pub pinger: pThread, // 子组件线程（Pinger : thread P）
    #[allow(dead_code)]
    pub ping_me: qThread, // 子组件线程（Ping_Me : thread Q）
}

impl Process for aProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut pinger: pThread = pThread::new(cpu_id);
        let mut ping_me: qThread = qThread::new(cpu_id);
        let channel = crossbeam_channel::unbounded();
        // build connection:
        pinger.data_source_1 = Some(channel.0);
        // build connection:
        ping_me.data_sink_1 = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection:
        pinger.data_source_2 = Some(channel.0);
        // build connection:
        ping_me.data_sink_2 = Some(channel.1);
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

// AADL System: PING
#[derive(Debug)]
pub struct pingSystem {
    #[allow(dead_code)]
    pub node_a: aProcess, // 子组件进程（Node_A : process A）
}

impl System for pingSystem {
    // Creates a new system instance
    fn new() -> Self {
        let mut node_a: aProcess = aProcess::new(0);
        return Self { node_a }; //显式return;
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
    use super::{custom_int, user_do_ping_spg};
    // Wrapper for C function user_do_ping_spg
    // Original AADL port: Data_Source
    pub fn send(data_source: &mut custom_int) -> () {
        unsafe {
            user_do_ping_spg(data_source);
        };
    }
}

pub mod ping_spg {
    // Auto-generated from AADL subprogram: Ping_Spg
    // C binding to: user_ping_spg
    // source_files: ping.c
    use super::{custom_int, user_ping_spg};
    // Wrapper for C function user_ping_spg
    // Original AADL port: Data_Sink
    pub fn receive(data_sink: custom_int) -> () {
        unsafe {
            user_ping_spg(data_sink);
        };
    }
}

// AADL Thread: p
#[derive(Debug)]
pub struct pThread {
    pub data_source_1: Option<Sender<custom_int>>, // Port: Data_Source_1 Out
    pub data_source_2: Option<Sender<custom_int>>, // Port: Data_Source_2 Out
    pub dispatch_protocol: String,                 // AADL属性: Dispatch_Protocol
    pub cpu_id: isize,                             // 结构体新增 CPU ID
    pub recover_entrypoint_source_text: String,    // AADL属性(impl): Recover_Entrypoint_Source_Text
    pub period: u64,                               // AADL属性(impl): Period
    pub deadline: u64,                             // AADL属性(impl): Deadline
    pub priority: u64,                             // AADL属性(impl): Priority
    pub dispatch_offset: u64,                      // AADL属性(impl): Dispatch_Offset
}

impl Thread for pThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            period: 2000,
            data_source_2: None,
            dispatch_offset: 500,
            priority: 2,
            deadline: 2000,
            data_source_1: None,
            dispatch_protocol: "Periodic".to_string(),
            recover_entrypoint_source_text: "recover".to_string(),
            cpu_id: cpu_id, // CPU ID
        };
    }

    // Thread execution entry point
    // Period: Some(2000) ms
    fn run(mut self) -> () {
        unsafe {
            let mut param: sched_param = sched_param { sched_priority: 2 };
            let ret = pthread_setschedparam(
                pthread_self(),
                *CPU_ID_TO_SCHED_POLICY
                    .get(&self.cpu_id)
                    .unwrap_or(&SCHED_FIFO),
                &mut param,
            );
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
                // p_spg() -> p_spg2();
                // p_spg;
                if let Some(sender) = &self.data_source_1 {
                    let mut val = 0;
                    do_ping_spg::send(&mut val);
                    sender.send(val).unwrap();
                };
                // p_spg2;
                if let Some(sender) = &self.data_source_2 {
                    let mut val = 0;
                    do_ping_spg::send(&mut val);
                    sender.send(val).unwrap();
                };
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        }
    }
}

// AADL Thread: q
#[derive(Debug)]
pub struct qThread {
    pub data_sink_1: Option<Receiver<custom_int>>, // Port: Data_Sink_1 In
    pub data_sink_2: Option<Receiver<custom_int>>, // Port: Data_Sink_2 In
    pub cpu_id: isize,                             // 结构体新增 CPU ID
    pub dispatch_protocol: String,                 // AADL属性(impl): Dispatch_Protocol
    pub period: u64,                               // AADL属性(impl): Period
    pub deadline: u64,                             // AADL属性(impl): deadline
    pub priority: u64,                             // AADL属性(impl): Priority
}

impl Thread for qThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            data_sink_2: None,
            priority: 1,
            data_sink_1: None,
            dispatch_protocol: "Sporadic".to_string(),
            deadline: 10,
            period: 10,
            cpu_id: cpu_id, // CPU ID
        };
    }

    // Thread execution entry point
    // Period: Some(10) ms
    fn run(mut self) -> () {
        let mut param = sched_param { sched_priority: 1 };
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
            eprintln!("qThread: Failed to set thread priority: {}", ret);
        };
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let min_interarrival: std::time::Duration = Duration::from_millis(10);
        let mut last_dispatch: std::time::Instant = Instant::now();
        let mut events = Vec::new();
        loop {
            if events.is_empty() {
                if let Some(rx) = &self.data_sink_2 {
                    if let Ok(val) = rx.try_recv() {
                        events.push(((val, 10, Instant::now())));
                    };
                };
                if let Some(rx) = &self.data_sink_1 {
                    if let Ok(val) = rx.try_recv() {
                        events.push(((val, 5, Instant::now())));
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
                    // q_spg() -> q_spg2();
                    // q_spg;
                    ping_spg::receive(val);
                    // q_spg2;
                    ping_spg::receive(val);
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
