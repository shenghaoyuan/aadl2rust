// Auto-generated from AADL package: aadlbook_icd
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

// AADL Data Type: obstacle_position
pub type obstacle_position = bool;

// AADL Data Type: speed
pub type speed = u16;

// AADL Data Type: picture
pub type picture = [[i32; 4]; 4];

// AADL Data Type: boolean
pub type boolean = bool;

// AADL Data Type: pressure
pub type pressure = i8;

// AADL Data Type: entertainment_infos
pub type entertainment_infos = i8;

// AADL Data Type: speed_cmd
pub type speed_cmd = i8;

// AADL Data Type: brake_cmd
pub type brake_cmd = i8;

// AADL Data Type: distance
pub type distance = u32;

// AADL Data Type: music
pub type music = bool;

// AADL Data Type: contacts
pub type contacts = i8;

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

