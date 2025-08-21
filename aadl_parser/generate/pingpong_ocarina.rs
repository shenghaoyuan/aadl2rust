// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-08-21 16:22:40

#![allow(unused_imports)]
use std::sync::{mpsc, Arc};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
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

// Process implementation: A
// Auto-generated from AADL
#[derive(Debug)]
pub struct aProcess {
    // 子组件线程（Pinger : thread P）
    #[allow(dead_code)]
    pub pinger: pThread,
    // 子组件线程（Ping_Me : thread Q）
    #[allow(dead_code)]
    pub ping_me: qThread,
    // 新增 CPU ID
    pub cpu_id: isize,
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
        thread::Builder::new()
            .name("pinger".to_string())
            .spawn(move || { self.pinger.run() }).unwrap();
        thread::Builder::new()
            .name("ping_me".to_string())
            .spawn(move || { self.ping_me.run() }).unwrap();
    }
    
}

// AADL System: PING
#[derive(Debug)]
pub struct pingSystem {
    // 进程和CPU的对应关系
    pub processes: Vec<(String, isize)>,
}

impl pingSystem {
    // 创建系统实例
    pub fn new() -> Self {
        return Self { processes: vec![("Node_A".to_string(), 0)] };
    }
    
    // 运行系统，启动所有进程
    pub fn run(self: Self) -> () {
        for (proc_name, cpu_id) in self.processes {
        match proc_name.as_str() {
            "Node_A" => {
                    let proc = aProcess::new(cpu_id);
                    proc.start();
                },
            _ => { eprintln!("Unknown process: {}", proc_name); }
           }
        };
    }
    
}

// AADL Data Type: Simple_Type
pub type Simple_Type = custom_int;

pub mod do_ping_spg {
    // Auto-generated from AADL subprogram: Do_Ping_Spg
    // C binding to: user_do_ping_spg
    // source_files: "ping.c"
    use super::{user_do_ping_spg, Simple_Type};
    // Wrapper for C function user_do_ping_spg
    // Original AADL port: Data_Source
    pub fn send(data_source: &mut Simple_Type) -> () {
        unsafe { user_do_ping_spg(data_source);
         };
    }
    
}

pub mod ping_spg {
    // Auto-generated from AADL subprogram: Ping_Spg
    // C binding to: user_ping_spg
    // source_files: "ping.c"
    use super::{user_ping_spg, Simple_Type};
    // Wrapper for C function user_ping_spg
    // Original AADL port: Data_Sink
    pub fn receive(data_sink: Simple_Type) -> () {
        unsafe { user_ping_spg(data_sink);
         };
    }
    
}

// AADL Thread: p
#[derive(Debug)]
pub struct pThread {
    // Port: Data_Source Out
    pub data_source: Option<mpsc::Sender<Simple_Type>>,
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
            let ret = pthread_setschedparam(pthread_self(), SCHED_FIFO, &mut param);
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
    pub data_sink: Option<mpsc::Receiver<Simple_Type>>,
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
            let ret = pthread_setschedparam(pthread_self(), SCHED_FIFO, &mut param);
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
                        let mut last_dispatch = Instant::now();
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

