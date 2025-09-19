// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-09-19 17:28:04

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

// AADL Process: a
#[derive(Debug)]
pub struct aProcess {
    // 进程 CPU ID
    pub cpu_id: isize,
    // 子组件线程（Pinger : thread P）
    #[allow(dead_code)]
    pub pinger: pThread,
    // 子组件线程（Ping_Me : thread Q）
    #[allow(dead_code)]
    pub ping_me: qThread,
}

impl aProcess {
    // Creates a new process instance
    pub fn new(cpu_id: isize) -> Self {
        let mut pinger: pThread = pThread::new(cpu_id);
        let mut ping_me: qThread = qThread::new(cpu_id);
        let channel = mpsc::channel();
        // build connection: 
            pinger.data_source = Some(channel.0);
        // build connection: 
            ping_me.data_sink = Some(channel.1);
        return Self { pinger, ping_me, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    pub fn start(self: Self) -> () {
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
    // 子组件进程（Node_A : process A）
    #[allow(dead_code)]
    pub node_a: aProcess,
}

impl pingSystem {
    // Creates a new system instance
    pub fn new() -> Self {
        let mut node_a: aProcess = aProcess::new(0);
        return Self { node_a }  //显式return;
    }
    
    // Runs the system, starts all processes
    pub fn run(self: Self) -> () {
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
    // Port: Data_Source Out
    pub data_source: Option<mpsc::Sender<custom_int>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub recover_entrypoint_source_text: String, // AADL属性: Recover_Entrypoint_Source_Text
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub deadline: u64, // AADL属性: Deadline
    pub priority: u64, // AADL属性: Priority
    pub dispatch_offset: u64, // AADL属性: Dispatch_Offset
}

impl pThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            data_source: None,
            cpu_id: cpu_id,
            recover_entrypoint_source_text: "recover".to_string(), // AADL属性: Recover_Entrypoint_Source_Text
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 2000, // AADL属性: Period
            deadline: 2000, // AADL属性: Deadline
            priority: 2, // AADL属性: Priority
            dispatch_offset: 500, // AADL属性: Dispatch_Offset
        }
    }
}
impl pThread {
    // Thread execution entry point
    // Period: Some(2000) ms
    pub fn run(mut self) -> () {
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
                           // P_Spg();
                // P_Spg;
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
    // Port: Data_Sink In
    pub data_sink: Option<mpsc::Receiver<custom_int>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub deadline: u64, // AADL属性: deadline
    pub priority: u64, // AADL属性: Priority
}

impl qThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            data_sink: None,
            cpu_id: cpu_id,
            dispatch_protocol: "Sporadic".to_string(), // AADL属性: Dispatch_Protocol
            period: 10, // AADL属性: Period
            deadline: 10, // AADL属性: deadline
            priority: 1, // AADL属性: Priority
        }
    }
}
impl qThread {
    // Thread execution entry point
    // Period: Some(10) ms
    pub fn run(mut self) -> () {
        unsafe {
            let mut param: sched_param = sched_param { sched_priority: 1 };
            let ret = pthread_setschedparam(pthread_self(), *CPU_ID_TO_SCHED_POLICY.get(&self.cpu_id).unwrap_or(&SCHED_FIFO), &mut param);
            if ret != 0 {
                eprintln!("qThread: Failed to set thread priority: {}", ret);
            };
        };
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let min_interarrival: std::time::Duration = Duration::from_millis(10);
        let mut last_dispatch: std::time::Instant = Instant::now();
        loop {
            if let Some(receiver) = &self.data_sink {
                match receiver.recv() {
                    Ok(val) => {
                        // 收到消息 → 调用处理函数
                        let now = Instant::now();
                        let elapsed = now.duration_since(last_dispatch);
                        if elapsed < min_interarrival {
                            std::thread::sleep(min_interarrival - elapsed);
                        };
                        {
                            // --- 调用序列（等价 AADL 的 Wrapper）---
                           // Q_Spg();
                            // Q_Spg;
                            ping_spg::receive(val);
                        };
                        last_dispatch = Instant::now();
                    },
                    Err(_) => {
                        eprintln!("qThread: channel closed");
                        return;
                    },
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

