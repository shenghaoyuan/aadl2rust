// Auto-generated from AADL package: aadlbook_software_image_acquisition
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

// AADL Process: image_acquisition
#[derive(Debug)]
pub struct image_acquisitionProcess {
    pub picture: Option<Receiver<[[i32; 4]; 4]>>,// Port: picture In
    pub obstacle_detected: Option<Sender<bool>>,// Port: obstacle_detected Out
    pub cpu_id: isize,// 进程 CPU ID
    pub pictureSend: Option<BcSender<[[i32; 4]; 4]>>,// 内部端口: picture In
    pub obstacle_detectedRece: Option<Receiver<bool>>,// 内部端口: obstacle_detected Out
    #[allow(dead_code)]
    pub acq_thr: image_acquisition_thrThread,// 子组件线程（acq_thr : thread image_acquisition_thr）
}

impl Process for image_acquisitionProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut acq_thr: image_acquisition_thrThread = image_acquisition_thrThread::new(cpu_id);
        let mut pictureSend = None;
        let mut obstacle_detectedRece = None;
        let channel = crossbeam_channel::unbounded();
        pictureSend = Some(channel.0);
        // build connection: 
            acq_thr.picture = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            acq_thr.obstacle_detected = Some(channel.0);
        obstacle_detectedRece = Some(channel.1);
        return Self { picture: None, pictureSend, obstacle_detected: None, obstacle_detectedRece, acq_thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn start(self: Self) -> () {
        let Self { picture, pictureSend, obstacle_detected, obstacle_detectedRece, acq_thr, cpu_id, .. } = self;
        thread::Builder::new()
            .name("acq_thr".to_string())
            .spawn(|| { acq_thr.run() }).unwrap();
        let mut obstacle_detectedRece_rx = obstacle_detectedRece.unwrap();
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
        let mut picture_rx = picture.unwrap();
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
            picture: None, 
            dispatch_protocol: "Periodic".to_string(), 
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
        // Behavior Annex state machine states
        #[derive(Debug, Clone)]
        enum State {
            // State: s0
            s0,
        }
        
        let mut state: State = State::s0;
        loop {
            let start = Instant::now();
            let picture = self.picture.as_mut().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
            {
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
                        State::s0 => {
                            // 理论上不会执行到这里，但编译器需要这个分支
                            break;
                        },
                    };
                    break;
                };
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

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

