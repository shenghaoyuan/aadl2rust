// Auto-generated from AADL package: aadlbook_software_image_acquisition
// 生成时间: 2025-12-24 18:40:24

#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_assignments)]
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

use crate::aadlbook_icd::*;
// ---------------- cpu ----------------
fn set_thread_affinity(cpu: isize) {
    unsafe {
        let mut cpuset: cpu_set_t = std::mem::zeroed();
        CPU_ZERO(&mut cpuset);
        CPU_SET(cpu as usize, &mut cpuset);
        sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset);
    }
}

// AADL Process: image_acquisition
#[derive(Debug)]
pub struct image_acquisitionProcess {
    pub picture: Option<Receiver<[[i32; 4]; 4]>>,// Port: picture In
    pub obstacle_detected: Option<Sender<bool>>,// Port: obstacle_detected Out
    pub cpu_id: isize,// 进程 CPU ID
    pub pictureSend: Option<Sender<[[i32; 4]; 4]>>,// 内部端口: picture In
    pub obstacle_detectedRece: Option<Receiver<bool>>,// 内部端口: obstacle_detected Out
    pub acq_thr: image_acquisition_thrThread,// 子组件线程(acq_thr : thread image_acquisition_thr)
}

impl Process for image_acquisitionProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut acq_thr: image_acquisition_thrThread = image_acquisition_thrThread::new(cpu_id);
        let mut pictureSend = None;
        let mut obstacle_detectedRece = None;
        let c0 = crossbeam_channel::unbounded();
        pictureSend = Some(c0.0);
        // build connection: 
            acq_thr.picture = Some(c0.1);
        let c1 = crossbeam_channel::unbounded();
        // build connection: 
            acq_thr.obstacle_detected = Some(c1.0);
        obstacle_detectedRece = Some(c1.1);
        return Self { picture: None, pictureSend, obstacle_detected: None, obstacle_detectedRece, acq_thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn run(self: Self) -> () {
        let Self { picture, pictureSend, obstacle_detected, obstacle_detectedRece, acq_thr, .. } = self;
        thread::Builder::new()
            .name("acq_thr".to_string())
            .spawn(move || { acq_thr.run() }).unwrap();
        let obstacle_detectedRece_rx = obstacle_detectedRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_obstacle_detectedRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = obstacle_detectedRece_rx.try_recv() {
                    if let Some(tx) = &obstacle_detected {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let picture_rx = picture.unwrap();
        thread::Builder::new()
            .name("data_forwarder_picture".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = picture_rx.try_recv() {
                    if let Some(tx) = &pictureSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
    }
    
}

// AADL Thread: image_acquisition_thr
#[derive(Debug)]
pub struct image_acquisition_thrThread {
    pub picture: Option<Receiver<[[i32; 4]; 4]>>,// Port: picture In
    pub obstacle_detected: Option<Sender<bool>>,// Port: obstacle_detected Out
    pub mipsbudget: f64,// AADL属性: mipsbudget
    pub dispatch_protocol: String,// AADL属性: Dispatch_Protocol
    pub period: u64,// AADL属性: Period
    pub cpu_id: isize,// 结构体新增 CPU ID
}

impl Thread for image_acquisition_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            dispatch_protocol: "Periodic".to_string(), 
            picture: None, 
            period: 50, 
            obstacle_detected: None, 
            mipsbudget: 25.0, 
            cpu_id: cpu_id, // CPU ID
        };
    }
    
    // Thread execution entry point
    // Period: None ms
    fn run(mut self) -> () {
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(2000);
        let mut next_release = Instant::now() + period;
        // Behavior Annex state machine states
        enum State {
            // State: s0
            s0,
        }
        
        let mut state: State = State::s0;
        loop {
            let now = Instant::now();
            if now < next_release {
                std::thread::sleep(next_release - now);
            };
            {
                let picture = self.picture.as_mut().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
                // --- BA 宏步执行 ---
                loop {
                    match state {
                        State::s0 if picture == true => {
                            if let Some(sender) = &self.obstacle_detected {
                                let _ = sender.send(false);
                            };
                            state = State::s0;
                            // complete,需要停
                        },
                        _ => {
                            break;
                        },
                    };
                    break;
                };
            };
            next_release += period;
        };
    }
    
}

