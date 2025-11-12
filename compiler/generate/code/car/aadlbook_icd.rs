// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-11-12 12:15:15

#![allow(unused_imports)]
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc,Mutex};
use std::thread;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::common_traits::*;
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

