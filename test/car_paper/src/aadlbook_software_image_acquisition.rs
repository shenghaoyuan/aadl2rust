// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-11-11 18:58:30

#![allow(unused_imports)]
use crate::aadlbook_icd::*;
use crate::common_traits::*;
use crossbeam_channel::{Receiver, Sender};
use lazy_static::lazy_static;
use libc::{
    cpu_set_t, pthread_self, pthread_setschedparam, sched_param, sched_setaffinity, CPU_SET,
    CPU_ZERO, SCHED_FIFO,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
include!(concat!(env!("OUT_DIR"), "/aadl_c_bindings.rs"));

use libc::{self, syscall, SYS_gettid};

fn get_tid() -> libc::pid_t {
    unsafe { syscall(SYS_gettid) as libc::pid_t }
}

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
    pub picture: Option<Receiver<[[i32; 4]; 4]>>, // Port: picture In
    pub obstacle_detected: Option<Sender<bool>>,  // Port: obstacle_detected Out
    pub cpu_id: isize,                            // 进程 CPU ID
    pub pictureSend: Option<Sender<[[i32; 4]; 4]>>, // 内部端口: picture In
    pub obstacle_detectedRece: Option<Receiver<bool>>, // 内部端口: obstacle_detected Out
    #[allow(dead_code)]
    pub thr_acq: image_acquisition_thrThread, // 子组件线程（thr_acq : thread image_acquisition_thr）
}

impl Process for image_acquisitionProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut thr_acq: image_acquisition_thrThread = image_acquisition_thrThread::new(cpu_id);
        let mut pictureSend = None;
        let mut obstacle_detectedRece = None;
        let channel = crossbeam_channel::unbounded();
        pictureSend = Some(channel.0);
        // build connection:
        thr_acq.picture = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection:
        thr_acq.obstacle_detected = Some(channel.0);
        obstacle_detectedRece = Some(channel.1);
        return Self {
            picture: None,
            pictureSend,
            obstacle_detected: None,
            obstacle_detectedRece,
            thr_acq,
            cpu_id,
        }; //显式return;
    }

    // Starts all threads in the process
    fn start(self: Self) -> () {
        let Self {
            picture,
            pictureSend,
            obstacle_detected,
            obstacle_detectedRece,
            thr_acq,
            cpu_id,
            ..
        } = self;
        thread::Builder::new()
            .name("acq_thr".to_string())
            .spawn(|| thr_acq.run())
            .unwrap();
        let picture_rx = picture.unwrap();
        thread::Builder::new()
            .name("data_forwarder_picture".to_string())
            .spawn(move || loop {
                if let Ok(msg) = picture_rx.recv() {
                    if let Some(tx) = &pictureSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            })
            .unwrap();
        let obstacle_detectedRece_rx = obstacle_detectedRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_obstacle_detectedRece".to_string())
            .spawn(move || loop {
                if let Ok(msg) = obstacle_detectedRece_rx.recv() {
                    if let Some(tx) = &obstacle_detected {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            })
            .unwrap();
    }
}

// AADL Thread: image_acquisition_thr
#[derive(Debug)]
pub struct image_acquisition_thrThread {
    pub picture: Option<Receiver<[[i32; 4]; 4]>>, // Port: picture In
    pub obstacle_detected: Option<Sender<bool>>,  // Port: obstacle_detected Out
    pub mipsbudget: f64,                          // AADL属性: mipsbudget
    pub dispatch_protocol: String,                // AADL属性: Dispatch_Protocol
    pub period: u64,                              // AADL属性: Period
    pub cpu_id: isize,                            // 结构体新增 CPU ID
}

impl Thread for image_acquisition_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            picture: None,
            dispatch_protocol: "Periodic".to_string(),
            period: 50,
            mipsbudget: 25.0,
            obstacle_detected: None,
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
        // Behavior Annex state machine states
        #[derive(Debug, Clone)]
        enum State {
            // State: s0
            s0,
        }

        let mut state: State = State::s0;
        loop {
            let t0 = std::time::Instant::now();
            while t0.elapsed().as_millis() < 20 {}

            let start = Instant::now();
            let picture = self
                .picture
                .as_ref()
                .and_then(|rx| rx.recv().ok())
                .unwrap_or_else(|| Default::default());
            {
                // --- BA 宏步执行 ---
                loop {
                    match state {
                        State::s0 => {
                            if let Some(sender) = &self.obstacle_detected {
                                if picture[0][0] > 100 {
                                    println!("Tid=[{}]acquisition:No obstacle exists.",get_tid());
                                    let _ = sender.send(false);
                                } else {
                                    println!("Tid=[{}]acquisition[T]:obstacle exists.",get_tid());
                                    let _ = sender.send(true);
                                }
                            }
                            // on dispatch → s0
                            state = State::s0;
                            // complete，需要停
                        }
                    };
                    break;
                }
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        }
    }
}
