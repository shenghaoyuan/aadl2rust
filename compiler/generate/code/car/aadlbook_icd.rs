// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-09-19 17:00:16

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

// AADL Struct: obstacle_position
#[derive(Debug, Clone)]
pub struct obstacle_position {
    // 子组件字段: present
    pub present: bool,
    // 子组件字段: x
    pub x: Unsigned_8,
}

// AADL Data Type: speed
pub type speed = u16;

// AADL Data Type: picture
pub type picture = [i32; 1];

// AADL Data Type: boolean
pub type boolean = bool;

// AADL Data Type: pressure
pub type pressure = Integer_8;

// AADL Data Type: entertainment_infos
pub type entertainment_infos = Integer_8;

// AADL Data Type: speed_cmd
pub type speed_cmd = Integer_8;

// AADL Data Type: brake_cmd
pub type brake_cmd = Integer_8;

// AADL Data Type: distance
pub type distance = u32;

// AADL Data Type: music
pub type music = ();

// AADL Data Type: contacts
pub type contacts = ();

