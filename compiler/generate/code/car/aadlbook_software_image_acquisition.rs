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

// AADL Process: image_acquisition
#[derive(Debug)]
pub struct image_acquisitionProcess {
    // Port: picture In
    pub picture: Option<mpsc::Receiver<picture>>,
    // Port: obstacle_detected Out
    pub obstacle_detected: Option<mpsc::Sender<obstacle_position>>,
    // 进程 CPU ID
    pub cpu_id: isize,
    // 内部端口: picture In
    pub pictureSend: Option<mpsc::Sender<picture>>,
    // 内部端口: obstacle_detected Out
    pub obstacle_detectedRece: Option<mpsc::Receiver<obstacle_position>>,
    // 子组件线程（thr_acq : thread image_acquisition_thr）
    #[allow(dead_code)]
    pub thr_acq: image_acquisition_thrThread,
}

impl image_acquisitionProcess {
    // Creates a new process instance
    pub fn new(cpu_id: isize) -> Self {
        let mut thr_acq: image_acquisition_thrThread = image_acquisition_thrThread::new(cpu_id);
        let mut pictureSend = None;
        let mut obstacle_detectedRece = None;
        let channel = mpsc::channel();
        pictureSend = Some(channel.0);
        // build connection: 
            thr_acq.picture = Some(channel.1);
        let channel = mpsc::channel();
        // build connection: 
            thr_acq.obstacle_detected = Some(channel.0);
        obstacle_detectedRece = Some(channel.1);
        return Self { picture: None, pictureSend, obstacle_detected: None, obstacle_detectedRece, thr_acq, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    pub fn start(self: Self) -> () {
        let Self { picture, pictureSend, obstacle_detected, obstacle_detectedRece, thr_acq, cpu_id, .. } = self;
        thread::Builder::new()
            .name("thr_acq".to_string())
            .spawn(|| { thr_acq.run() }).unwrap();
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
    }
    
}

// AADL Thread: image_acquisition_thr
#[derive(Debug)]
pub struct image_acquisition_thrThread {
    // Port: picture In
    pub picture: Option<mpsc::Receiver<picture>>,
    // Port: obstacle_detected Out
    pub obstacle_detected: Option<mpsc::Sender<obstacle_position>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub mipsbudget: f64, // AADL属性: mipsbudget
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
}

impl image_acquisition_thrThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            picture: None,
            obstacle_detected: None,
            cpu_id: cpu_id,
            mipsbudget: 25, // AADL属性: mipsbudget
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 50, // AADL属性: Period
        }
    }
}
impl image_acquisition_thrThread {
    // Thread execution entry point
    // Period: None ms
    pub fn run(mut self) -> () {
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(2000);
        loop {
            let start = Instant::now();
            {
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

