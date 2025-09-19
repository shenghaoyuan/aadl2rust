// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-09-19 17:25:09

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
    // 子组件字段: f1
    pub f1: f32,
    // 子组件字段: c2
    pub c2: char,
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
    // 联合体字段: f1
    pub f1: f32,
    // 联合体字段: c2
    pub c2: char,
}

// AADL Enum: An_Enum
#[derive(Debug, Clone)]
pub enum An_Enum {
    foo,
    bar,
}

// AADL Data Type: C_Type
pub type C_Type = the_type;

