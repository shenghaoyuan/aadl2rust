// Auto-generated from AADL package: base_types
// 生成时间: 2025-12-10 21:18:19

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

// AADL Data Type: Boolean
pub type Boolean = bool;

// AADL Data Type: Integer
pub type Integer = i32;

// AADL Data Type: Integer_8
pub type Integer_8 = i8;

// AADL Data Type: Integer_16
pub type Integer_16 = i16;

// AADL Data Type: Integer_32
pub type Integer_32 = i32;

// AADL Data Type: Integer_64
pub type Integer_64 = i64;

// AADL Data Type: Unsigned_8
pub type Unsigned_8 = u8;

// AADL Data Type: Unsigned_16
pub type Unsigned_16 = u16;

// AADL Data Type: Unsigned_32
pub type Unsigned_32 = u32;

// AADL Data Type: Unsigned_64
pub type Unsigned_64 = u64;

// AADL Data Type: Natural
pub type Natural = usize;

// AADL Data Type: Float
pub type Float = f32;

// AADL Data Type: Float_32
pub type Float_32 = f32;

// AADL Data Type: Float_64
pub type Float_64 = f64;

// AADL Data Type: Character
pub type Character = char;

// AADL Data Type: String
pub type String = String;

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

