// Auto-generated from AADL package: aadlbook_software_obstacle_detection
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

// AADL Process: obstacle_detection
#[derive(Debug)]
pub struct obstacle_detectionProcess {
    pub camera: Option<Receiver<bool>>,// Port: camera In
    pub radar: Option<Receiver<bool>>,// Port: radar In
    pub obstacle_position: Option<Sender<bool>>,// Port: obstacle_position Out
    pub cpu_id: isize,// 进程 CPU ID
    pub cameraSend: Option<Sender<bool>>,// 内部端口: camera In
    pub radarSend: Option<Sender<bool>>,// 内部端口: radar In
    pub obstacle_positionRece: Option<Receiver<bool>>,// 内部端口: obstacle_position Out
    pub obst_thr: obstacle_detection_thrThread,// 子组件线程(obst_thr : thread obstacle_detection_thr)
}

impl Process for obstacle_detectionProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut obst_thr: obstacle_detection_thrThread = obstacle_detection_thrThread::new(cpu_id);
        let mut cameraSend = None;
        let mut radarSend = None;
        let mut obstacle_positionRece = None;
        let c0 = crossbeam_channel::unbounded();
        cameraSend = Some(c0.0);
        // build connection: 
            obst_thr.camera = Some(c0.1);
        let c1 = crossbeam_channel::unbounded();
        radarSend = Some(c1.0);
        // build connection: 
            obst_thr.radar = Some(c1.1);
        let c2 = crossbeam_channel::unbounded();
        // build connection: 
            obst_thr.obstacle_detected = Some(c2.0);
        obstacle_positionRece = Some(c2.1);
        return Self { camera: None, cameraSend, radar: None, radarSend, obstacle_position: None, obstacle_positionRece, obst_thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn run(self: Self) -> () {
        let Self { camera, cameraSend, radar, radarSend, obstacle_position, obstacle_positionRece, obst_thr, .. } = self;
        thread::Builder::new()
            .name("obst_thr".to_string())
            .spawn(move || { obst_thr.run() }).unwrap();
        let camera_rx = camera.unwrap();
        thread::Builder::new()
            .name("data_forwarder_camera".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = camera_rx.try_recv() {
                    if let Some(tx) = &cameraSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let obstacle_positionRece_rx = obstacle_positionRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_obstacle_positionRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = obstacle_positionRece_rx.try_recv() {
                    if let Some(tx) = &obstacle_position {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let radar_rx = radar.unwrap();
        thread::Builder::new()
            .name("data_forwarder_radar".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = radar_rx.try_recv() {
                    if let Some(tx) = &radarSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
    }
    
}

// AADL Thread: obstacle_detection_thr
#[derive(Debug)]
pub struct obstacle_detection_thrThread {
    pub camera: Option<Receiver<bool>>,// Port: camera In
    pub radar: Option<Receiver<bool>>,// Port: radar In
    pub obstacle_detected: Option<Sender<bool>>,// Port: obstacle_detected Out
    pub dispatch_protocol: String,// AADL属性: Dispatch_Protocol
    pub period: u64,// AADL属性: Period
    pub mipsbudget: f64,// AADL属性: mipsbudget
    pub cpu_id: isize,// 结构体新增 CPU ID
}

impl Thread for obstacle_detection_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            obstacle_detected: None, 
            dispatch_protocol: "Periodic".to_string(), 
            mipsbudget: 10.0, 
            radar: None, 
            period: 100, 
            camera: None, 
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
            // State: s1
            s1,
        }
        
        let mut state: State = State::s0;
        loop {
            let now = Instant::now();
            if now < next_release {
                std::thread::sleep(next_release - now);
            };
            {
                let camera = self.camera.as_mut().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
                let radar = self.radar.as_mut().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
                // --- BA 宏步执行 ---
                loop {
                    match state {
                        State::s0 if camera == true => {
                            if let Some(sender) = &self.obstacle_detected {
                                let _ = sender.send(true);
                            };
                            state = State::s0;
                            // complete,需要停
                        },
                        State::s0 if camera == false => {
                            state = State::s1;
                            continue;
                        },
                        State::s1 if radar == true => {
                            if let Some(sender) = &self.obstacle_detected {
                                let _ = sender.send(true);
                            };
                            state = State::s0;
                            // complete,需要停
                        },
                        State::s1 if radar == false => {
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

