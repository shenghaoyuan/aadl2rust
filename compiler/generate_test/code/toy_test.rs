// Auto-generated from AADL package: toy_example_nowrapper
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

// AADL Data Type: POS_Internal_Type
pub type POS_Internal_Type = i32;

// AADL Data Type: POS_Internal_Type2
pub type POS_Internal_Type2 = f32;

// AADL Data Type: POS
pub type POS = ();

pub mod update {
    // Auto-generated from AADL subprogram: Update
    // C binding to: user_update
    // source_files: toy.c
    use super::{user_update, POS_Internal_Type};
    // Call C function user_update with data access reference
    // Generated for requires data access feature
    // Note: Rust compiler will handle the reference to pointer conversion
    pub fn call(pos_ref: &mut POS_Internal_Type) -> () {
        unsafe { user_update(pos_ref);
         };
    }
    
}

pub mod read_pos {
    // Auto-generated from AADL subprogram: Read_POS
    // C binding to: user_read
    // source_files: toy.c
    use super::{user_read, POS_Internal_Type};
    // Call C function user_read with data access reference
    // Generated for requires data access feature
    // Note: Rust compiler will handle the reference to pointer conversion
    pub fn call(pos_ref: &mut POS_Internal_Type) -> () {
        unsafe { user_read(pos_ref);
         };
    }
    
}

pub mod gnc_job {
    // Auto-generated from AADL subprogram: GNC_Job
    // C binding to: user_gnc_job
    // source_files: toy.c
    use super::{user_gnc_job};
    // Direct execution wrapper for C function user_gnc_job
    // This component has no communication ports
    pub fn execute() -> () {
        unsafe { user_gnc_job();
         };
    }
    
}

pub mod tmtc_job {
    // Auto-generated from AADL subprogram: TMTC_Job
    // C binding to: user_tmtc_job
    // source_files: toy.c
    use super::{user_tmtc_job};
    // Direct execution wrapper for C function user_tmtc_job
    // This component has no communication ports
    pub fn execute() -> () {
        unsafe { user_tmtc_job();
         };
    }
    
}

pub mod gnc_identity {
    // Auto-generated from AADL subprogram: GNC_Identity
    // C binding to: user_gnc_identity
    // source_files: toy.c
    use super::{user_gnc_identity};
    // Direct execution wrapper for C function user_gnc_identity
    // This component has no communication ports
    pub fn execute() -> () {
        unsafe { user_gnc_identity();
         };
    }
    
}

pub mod tmtc_identity {
    // Auto-generated from AADL subprogram: TMTC_Identity
    // C binding to: user_tmtc_identity
    // source_files: toy.c
    use super::{user_tmtc_identity};
    // Direct execution wrapper for C function user_tmtc_identity
    // This component has no communication ports
    pub fn execute() -> () {
        unsafe { user_tmtc_identity();
         };
    }
    
}

// AADL Thread: gnc_thread
#[derive(Debug)]
pub struct gnc_threadThread {
    pub gnc_pos: POSShared,// AADL feature: GNC_POS : requires data access POS.Impl
    pub cpu_id: isize,// 结构体新增 CPU ID
    pub dispatch_protocol: String,// AADL属性(impl): Dispatch_Protocol
    pub period: u64,// AADL属性(impl): Period
    pub deadline: u64,// AADL属性(impl): Deadline
    pub priority: u64,// AADL属性(impl): Priority
}

impl Thread for gnc_threadThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize, gnc_pos: POSShared) -> Self {
        return Self {
            deadline: 1000, 
            gnc_pos: gnc_pos, 
            dispatch_protocol: "Periodic".to_string(), 
            period: 1000, 
            priority: 50, 
            cpu_id: cpu_id, // CPU ID
        };
    }
    
    // Thread execution entry point
    // Period: Some(1000) ms
    fn run(mut self) -> () {
        unsafe {
            let mut param: sched_param = sched_param { sched_priority: 50 };
            let ret = pthread_setschedparam(pthread_self(), *CPU_ID_TO_SCHED_POLICY.get(&self.cpu_id).unwrap_or(&SCHED_FIFO), &mut param);
            if ret != 0 {
                eprintln!("gnc_threadThread: Failed to set thread priority: {}", ret);
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
                           // welcome() -> update_pos() -> gnc_work() -> read_pos() -> bye();
                // welcome;
                gnc_identity::execute();
                // update_pos;
                {
                    {
                        if let Ok(mut guard) = self.gnc_pos.lock() {
                            update::call(&mut guard);
                        };
                    };
                };
                // gnc_work;
                gnc_job::execute();
                // read_pos;
                {
                    {
                        if let Ok(mut guard) = self.gnc_pos.lock() {
                            read_pos::call(&mut guard);
                        };
                    };
                };
                // bye;
                gnc_identity::execute();
            };
            next_release += period;
        };
    }
    
}

// AADL Thread: tmtc_thread
#[derive(Debug)]
pub struct tmtc_threadThread {
    pub tmtc_pos: POSShared,// AADL feature: TMTC_POS : requires data access POS.Impl
    pub cpu_id: isize,// 结构体新增 CPU ID
    pub dispatch_protocol: String,// AADL属性(impl): Dispatch_Protocol
    pub period: u64,// AADL属性(impl): Period
    pub deadline: u64,// AADL属性(impl): Deadline
    pub priority: u64,// AADL属性(impl): Priority
}

impl Thread for tmtc_threadThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize, tmtc_pos: POSShared) -> Self {
        return Self {
            priority: 20, 
            dispatch_protocol: "Periodic".to_string(), 
            period: 100, 
            tmtc_pos: tmtc_pos, 
            deadline: 100, 
            cpu_id: cpu_id, // CPU ID
        };
    }
    
    // Thread execution entry point
    // Period: Some(100) ms
    fn run(mut self) -> () {
        unsafe {
            let mut param: sched_param = sched_param { sched_priority: 20 };
            let ret = pthread_setschedparam(pthread_self(), *CPU_ID_TO_SCHED_POLICY.get(&self.cpu_id).unwrap_or(&SCHED_FIFO), &mut param);
            if ret != 0 {
                eprintln!("tmtc_threadThread: Failed to set thread priority: {}", ret);
            };
        };
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(100);
        let mut next_release = Instant::now() + period;
        loop {
            let now = Instant::now();
            if now < next_release {
                std::thread::sleep(next_release - now);
            };
            {
                // --- 调用序列（等价 AADL 的 Wrapper）---
                           // welcome() -> tmtc_work() -> update() -> bye();
                // welcome;
                tmtc_identity::execute();
                // tmtc_work;
                tmtc_job::execute();
                // update;
                {
                    {
                        if let Ok(mut guard) = self.tmtc_pos.lock() {
                            update::call(&mut guard);
                        };
                    };
                };
                // bye;
                tmtc_identity::execute();
            };
            next_release += period;
        };
    }
    
}

// AADL Process: toy_example_proc
#[derive(Debug)]
pub struct toy_example_procProcess {
    pub cpu_id: isize,// 进程 CPU ID
    pub gnc_th: gnc_threadThread,// 子组件线程（GNC_Th : thread GNC_Thread）
    pub tmtc_th: tmtc_threadThread,// 子组件线程（TMTC_Th : thread TMTC_Thread）
    pub pos_data: POSShared,// 共享数据（POS_Data : data POS）
}

impl Process for toy_example_procProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let pos_data: POSShared = Arc::new(Mutex::new(0));
        let gnc_th: gnc_threadThread = gnc_threadThread::new(cpu_id, Arc::clone(&pos_data));
        let tmtc_th: tmtc_threadThread = tmtc_threadThread::new(cpu_id, Arc::clone(&pos_data));
        return Self { gnc_th, tmtc_th, pos_data, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn run(self: Self) -> () {
        let Self { gnc_th, tmtc_th, pos_data, cpu_id, .. } = self;
        thread::Builder::new()
            .name("gnc_th".to_string())
            .spawn(move || { gnc_th.run() }).unwrap();
        thread::Builder::new()
            .name("tmtc_th".to_string())
            .spawn(move || { tmtc_th.run() }).unwrap();
    }
    
}

// AADL System: toy_example
#[derive(Debug)]
pub struct toy_exampleSystem {
    pub gnc_tmtc_pos: toy_example_procProcess,// 子组件进程（GNC_TMTC_POS : process Toy_Example_Proc）
}

impl System for toy_exampleSystem {
    // Creates a new system instance
    fn new() -> Self {
        let mut gnc_tmtc_pos: toy_example_procProcess = toy_example_procProcess::new(2);
        return Self { gnc_tmtc_pos }  //显式return;
    }
    
    // Runs the system, starts all processes
    fn run(self: Self) -> () {
        self.gnc_tmtc_pos.run();
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

