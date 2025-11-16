// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-10-15 13:31:52

#![allow(unused_imports)]
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc,Mutex};
use std::thread;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use std::collections::HashMap;
include!(concat!(env!("OUT_DIR"), "/aadl_c_bindings.rs"));
use crate::common_traits::*;

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

