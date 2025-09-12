// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-09-12 19:34:35

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

