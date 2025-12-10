// Auto-generated from AADL package: base_types_example_types
// 生成时间: 2025-12-10 21:18:20

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

// AADL Data Type: One_Dimension_Array
pub type One_Dimension_Array = [i32; 42];

// AADL Data Type: Two_Dimensions_Array
pub type Two_Dimensions_Array = [[i32; 75]; 74];

// AADL Struct: A_Struct1
#[derive(Debug, Clone)]
pub struct A_Struct1 {
    pub f1: f32,
    pub c2: char,
}

// AADL Struct: A_Struct2
#[derive(Debug, Clone)]
pub struct A_Struct2 {
    pub f1: f32,// 子组件字段: f1
    pub c2: char,// 子组件字段: c2
}

// AADL Union: A_Union1
#[derive(Debug, Clone)]
pub union A_Union1 {
    pub f1: f32,
    pub f2: char,
}

// AADL Union: A_Union2
#[derive(Debug, Clone)]
pub union A_Union2 {
    pub f1: f32,// 联合体字段: f1
    pub c2: char,// 联合体字段: c2
}

// AADL Enum: An_Enum
#[derive(Debug, Clone)]
pub enum An_Enum {
    foo,
    bar,
}

// AADL Data Type: C_Type
pub type C_Type = the_type;

// CPU ID到调度策略的映射
lazy_static! {
    static ref CPU_ID_TO_SCHED_POLICY: HashMap<isize, i32> = {
        let mut map: HashMap<isize, i32> = HashMap::new();
        map.insert(3, SCHED_FIFO);
        map.insert(1, SCHED_FIFO);
        map.insert(0, SCHED_FIFO);
        map.insert(2, SCHED_FIFO);
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

