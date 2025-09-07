// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-09-07 22:03:36

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

// AADL Data Type: POS_Internal_Type
pub type POS_Internal_Type = i32;

// Shared data type for POS
// Auto-generated from AADL data implementation
pub type POSShared = Arc<Mutex<POS_Internal_Type>>;

pub mod update {
    // Auto-generated from AADL subprogram: Update
    // C binding to: user_update
    // source_files: "toy.c"
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
    // source_files: "toy.c"
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
    // source_files: "toy.c"
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
    // source_files: "toy.c"
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
    // source_files: "toy.c"
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
    // source_files: "toy.c"
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
    // AADL feature: GNC_POS : requires data access POS.Impl
    pub gnc_pos: POSShared,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub deadline: u64, // AADL属性: Deadline
    pub priority: u64, // AADL属性: Priority
}

impl gnc_threadThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize, gnc_pos: POSShared) -> Self {
        Self {
            gnc_pos: gnc_pos,
            cpu_id: cpu_id,
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 1000, // AADL属性: Period
            deadline: 1000, // AADL属性: Deadline
            priority: 50, // AADL属性: Priority
        }
    }
}
impl gnc_threadThread {
    // Thread execution entry point
    // Period: Some(1000) ms
    pub fn run(mut self) -> () {
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
        loop {
            let start = Instant::now();
            {
                // --- 调用序列（等价 AADL 的 Wrapper）---
            // Welcome() -> Update_POS() -> GNC_Work() -> Read_POS() -> Bye();
                // Welcome;
                gnc_identity::execute();
                // Update_POS;
                {
                    {
                        if let Ok(mut guard) = self.gnc_pos.lock() {
                            update::call(&mut guard);
                        };
                    };
                };
                // GNC_Work;
                gnc_job::execute();
                // Read_POS;
                {
                    {
                        if let Ok(mut guard) = self.gnc_pos.lock() {
                            read_pos::call(&mut guard);
                        };
                    };
                };
                // Bye;
                gnc_identity::execute();
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

// AADL Thread: tmtc_thread
#[derive(Debug)]
pub struct tmtc_threadThread {
    // AADL feature: TMTC_POS : requires data access POS.Impl
    pub tmtc_pos: POSShared,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub deadline: u64, // AADL属性: Deadline
    pub priority: u64, // AADL属性: Priority
}

impl tmtc_threadThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize, tmtc_pos: POSShared) -> Self {
        Self {
            tmtc_pos: tmtc_pos,
            cpu_id: cpu_id,
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 100, // AADL属性: Period
            deadline: 100, // AADL属性: Deadline
            priority: 20, // AADL属性: Priority
        }
    }
}
impl tmtc_threadThread {
    // Thread execution entry point
    // Period: Some(100) ms
    pub fn run(mut self) -> () {
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
        loop {
            let start = Instant::now();
            {
                // --- 调用序列（等价 AADL 的 Wrapper）---
            // Welcome() -> TMTC_Work() -> Update() -> Bye();
                // Welcome;
                tmtc_identity::execute();
                // TMTC_Work;
                tmtc_job::execute();
                // Update;
                {
                    {
                        if let Ok(mut guard) = self.tmtc_pos.lock() {
                            update::call(&mut guard);
                        };
                    };
                };
                // Bye;
                tmtc_identity::execute();
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

// AADL Process: toy_example_proc
#[derive(Debug)]
pub struct toy_example_procProcess {
    // 进程 CPU ID
    pub cpu_id: isize,
    // 子组件线程（GNC_Th : thread GNC_Thread）
    #[allow(dead_code)]
    pub gnc_th: gnc_threadThread,
    // 子组件线程（TMTC_Th : thread TMTC_Thread）
    #[allow(dead_code)]
    pub tmtc_th: tmtc_threadThread,
    // 共享数据（POS_Data : data POS）
    #[allow(dead_code)]
    pub pos_data: POSShared,
}

impl toy_example_procProcess {
    // Creates a new process instance
    pub fn new(cpu_id: isize) -> Self {
        let mut pos_data: POSShared = Arc::new(Mutex::new(0));
        let mut gnc_th: gnc_threadThread = gnc_threadThread::new(cpu_id, pos_data.clone());
        let mut tmtc_th: tmtc_threadThread = tmtc_threadThread::new(cpu_id, pos_data.clone());
        return Self { gnc_th, tmtc_th, pos_data, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    pub fn start(self: Self) -> () {
        let Self { gnc_th, tmtc_th, pos_data, cpu_id, .. } = self;
        thread::Builder::new()
            .name("gnc_th".to_string())
            .spawn(|| { gnc_th.run() }).unwrap();
        thread::Builder::new()
            .name("tmtc_th".to_string())
            .spawn(|| { tmtc_th.run() }).unwrap();
    }
    
}

// AADL System: toy_example
#[derive(Debug)]
pub struct toy_exampleSystem {
    // 子组件进程（GNC_TMTC_POS : process Toy_Example_Proc）
    #[allow(dead_code)]
    pub gnc_tmtc_pos: toy_example_procProcess,
}

impl toy_exampleSystem {
    // Creates a new system instance
    pub fn new() -> Self {
        let mut gnc_tmtc_pos: toy_example_procProcess = toy_example_procProcess::new(0);
        return Self { gnc_tmtc_pos }  //显式return;
    }
    
    // Runs the system, starts all processes
    pub fn run(self: Self) -> () {
        self.gnc_tmtc_pos.start();
    }
    
}

// CPU ID到调度策略的映射
// 自动从AADL CPU实现中生成
lazy_static! {
    static ref CPU_ID_TO_SCHED_POLICY: HashMap<isize, i32> = {
        let mut map: HashMap<isize, i32> = HashMap::new();
        map.insert(0, SCHED_FIFO);
        return map;
    };
}

