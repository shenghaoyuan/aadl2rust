// Auto-generated from AADL package: data_impl_tests
// 生成时间: 2025-12-21 15:16:11

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

use crate::Base_Types::*;
use crate::Data_Model::*;
// ---------------- cpu ----------------
fn set_thread_affinity(cpu: isize) {
    unsafe {
        let mut cpuset: cpu_set_t = std::mem::zeroed();
        CPU_ZERO(&mut cpuset);
        CPU_SET(cpu as usize, &mut cpuset);
        sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset);
    }
}

// AADL Data Type: Shared_Int
pub type Shared_Int = ();

// Port handler for D
// Direction: InOut
pub fn handle_D(port: Option<Sender<i32>>) -> () {
    // Handle port: D;
}

// Port handler for D
// Direction: InOut
pub fn handle_D(port: Option<Sender<i32>>) -> () {
    // Handle port: D;
}

// AADL Data Type: Shared_Impl_Only
pub type Shared_Impl_Only = ();

// Shared data type for Shared_Impl_Only
// Auto-generated from AADL data implementation
pub type Shared_Impl_OnlyShared = Arc<Mutex<Shared_Int>>;

// AADL Data Type: Shared_Type_Only
pub type Shared_Type_Only = ();

// Shared data type for Shared_Type_Only
// Auto-generated from AADL data implementation
pub type Shared_Type_OnlyShared = Arc<Mutex<Integer>>;

// AADL Data Type: Shared_Multi_Data
pub type Shared_Multi_Data = ();

// AADL Data Type: Record_Data
pub type Record_Data = ();

// AADL Data Type: Union_Data
pub type Union_Data = ();

// AADL Process: data_test_proc
#[derive(Debug)]
pub struct data_test_procProcess {
    pub cpu_id: isize,// 进程 CPU ID
    pub shared1: Shared_Impl_OnlyShared,// 共享数据（Shared1 : data Shared_Impl_Only）
    pub shared2: Shared_Type_OnlyShared,// 共享数据（Shared2 : data Shared_Type_Only）
    pub shared3: Shared_Multi_DataShared,// 共享数据（Shared3 : data Shared_Multi_Data）
    pub recinst: Record_DataShared,// 共享数据（RecInst : data Record_Data）
    pub unioninst: Union_DataShared,// 共享数据（UnionInst : data Union_Data）
}

impl Process for data_test_procProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let shared1: Shared_Impl_OnlyShared = Arc::new(Mutex::new(0));
        let shared2: Shared_Type_OnlyShared = Arc::new(Mutex::new(0));
        let shared3: Shared_Multi_DataShared = Arc::new(Mutex::new(0));
        let recinst: Record_DataShared = Arc::new(Mutex::new(0));
        let unioninst: Union_DataShared = Arc::new(Mutex::new(0));
        return Self { shared1, shared2, shared3, recinst, unioninst, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn run(self: Self) -> () {
        let Self { shared1, shared2, shared3, recinst, unioninst, cpu_id, .. } = self;
    }
    
}

// AADL System: Data_Test_System
#[derive(Debug)]
pub struct data_test_systemSystem {
    pub proc: data_test_procProcess,// 子组件进程（Proc : process Data_Test_Proc）
}

impl System for data_test_systemSystem {
    // Creates a new system instance
    fn new() -> Self {
        let mut proc: data_test_procProcess = data_test_procProcess::new(0);
        return Self { proc }  //显式return;
    }
    
    // Runs the system, starts all processes
    fn run(self: Self) -> () {
        self.proc.run();
    }
    
}

// CPU ID到调度策略的映射
lazy_static! {
    static ref CPU_ID_TO_SCHED_POLICY: HashMap<isize, i32> = {
        let mut map: HashMap<isize, i32> = HashMap::new();
        map.insert(0, SCHED_FIFO);
        map.insert(2, SCHED_FIFO);
        map.insert(3, SCHED_FIFO);
        map.insert(1, SCHED_FIFO);
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

