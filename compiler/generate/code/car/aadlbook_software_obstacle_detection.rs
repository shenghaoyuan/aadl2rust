// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-12-08 23:05:12

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

// AADL Process: obstacle_detection
#[derive(Debug)]
pub struct obstacle_detectionProcess {
    pub camera: Option<Receiver<bool>>,// Port: camera In
    pub radar: Option<Receiver<bool>>,// Port: radar In
    pub obstacle_position: Option<Sender<bool>>,// Port: obstacle_position Out
    pub cpu_id: isize,// 进程 CPU ID
    pub cameraSend: Option<BcSender<bool>>,// 内部端口: camera In
    pub radarSend: Option<BcSender<bool>>,// 内部端口: radar In
    pub obstacle_positionRece: Option<Receiver<bool>>,// 内部端口: obstacle_position Out
    #[allow(dead_code)]
    pub obst_thr: obstacle_detection_thrThread,// 子组件线程（obst_thr : thread obstacle_detection_thr）
}

impl Process for obstacle_detectionProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut obst_thr: obstacle_detection_thrThread = obstacle_detection_thrThread::new(cpu_id);
        let mut cameraSend = None;
        let mut radarSend = None;
        let mut obstacle_positionRece = None;
        let channel = crossbeam_channel::unbounded();
        cameraSend = Some(channel.0);
        // build connection: 
            obst_thr.camera = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        radarSend = Some(channel.0);
        // build connection: 
            obst_thr.radar = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            obst_thr.obstacle_detected = Some(channel.0);
        obstacle_positionRece = Some(channel.1);
        return Self { camera: None, cameraSend, radar: None, radarSend, obstacle_position: None, obstacle_positionRece, obst_thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn start(self: Self) -> () {
        let Self { camera, cameraSend, radar, radarSend, obstacle_position, obstacle_positionRece, obst_thr, cpu_id, .. } = self;
        thread::Builder::new()
            .name("obst_thr".to_string())
            .spawn(|| { obst_thr.run() }).unwrap();
        let mut camera_rx = camera.unwrap();
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
        let mut obstacle_positionRece_rx = obstacle_positionRece.unwrap();
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
        let mut radar_rx = radar.unwrap();
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
            camera: None, 
            radar: None, 
            obstacle_detected: None, 
            period: 100, 
            mipsbudget: 10.0, 
            dispatch_protocol: "Periodic".to_string(), 
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
            // State: s1
            s1,
        }
        
        let mut state: State = State::s0;
        loop {
            let start = Instant::now();
            let radar = self.radar.as_mut().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
            let camera = self.camera.as_mut().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
            {
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
                        State::s1 => {
                            // 理论上不会执行到这里，但编译器需要这个分支
                            break;
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
        map.insert(0, SCHED_FIFO);
        return map;
    };
}

