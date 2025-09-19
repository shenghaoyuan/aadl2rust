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

// AADL Process: speed_voter
#[derive(Debug)]
pub struct speed_voterProcess {
    // Port: wheel_sensor In
    pub wheel_sensor: Option<mpsc::Receiver<speed>>,
    // Port: laser_sensor In
    pub laser_sensor: Option<mpsc::Receiver<speed>>,
    // Port: speed Out
    pub speed: Option<mpsc::Sender<speed>>,
    // 进程 CPU ID
    pub cpu_id: isize,
    // 内部端口: wheel_sensor In
    pub wheel_sensorSend: Option<mpsc::Sender<speed>>,
    // 内部端口: laser_sensor In
    pub laser_sensorSend: Option<mpsc::Sender<speed>>,
    // 内部端口: speed Out
    pub speedRece: Option<mpsc::Receiver<speed>>,
    // 子组件线程（thr : thread speed_voter_thr）
    #[allow(dead_code)]
    pub thr: speed_voter_thrThread,
}

impl speed_voterProcess {
    // Creates a new process instance
    pub fn new(cpu_id: isize) -> Self {
        let mut thr: speed_voter_thrThread = speed_voter_thrThread::new(cpu_id);
        let mut wheel_sensorSend = None;
        let mut laser_sensorSend = None;
        let mut speedRece = None;
        let channel = mpsc::channel();
        wheel_sensorSend = Some(channel.0);
        // build connection: 
            thr.wheel_sensor = Some(channel.1);
        let channel = mpsc::channel();
        laser_sensorSend = Some(channel.0);
        // build connection: 
            thr.laser_sensor = Some(channel.1);
        let channel = mpsc::channel();
        // build connection: 
            thr.speed = Some(channel.0);
        speedRece = Some(channel.1);
        return Self { wheel_sensor: None, wheel_sensorSend, laser_sensor: None, laser_sensorSend, speed: None, speedRece, thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    pub fn start(self: Self) -> () {
        let Self { wheel_sensor, wheel_sensorSend, laser_sensor, laser_sensorSend, speed, speedRece, thr, cpu_id, .. } = self;
        thread::Builder::new()
            .name("thr".to_string())
            .spawn(|| { thr.run() }).unwrap();
        let wheel_sensor_rx = wheel_sensor.unwrap();
        thread::Builder::new()
            .name("data_forwarder_wheel_sensor".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = wheel_sensor_rx.try_recv() {
                    if let Some(tx) = &wheel_sensorSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let laser_sensor_rx = laser_sensor.unwrap();
        thread::Builder::new()
            .name("data_forwarder_laser_sensor".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = laser_sensor_rx.try_recv() {
                    if let Some(tx) = &laser_sensorSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let speedRece_rx = speedRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_speedRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = speedRece_rx.try_recv() {
                    if let Some(tx) = &speed {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
    }
    
}

// AADL Thread: speed_voter_thr
#[derive(Debug)]
pub struct speed_voter_thrThread {
    // Port: wheel_sensor In
    pub wheel_sensor: Option<mpsc::Receiver<speed>>,
    // Port: laser_sensor In
    pub laser_sensor: Option<mpsc::Receiver<speed>>,
    // Port: speed Out
    pub speed: Option<mpsc::Sender<speed>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub mipsbudget: f64, // AADL属性: mipsbudget
}

impl speed_voter_thrThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            wheel_sensor: None,
            laser_sensor: None,
            speed: None,
            cpu_id: cpu_id,
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 8, // AADL属性: Period
            mipsbudget: 8, // AADL属性: mipsbudget
        }
    }
}
impl speed_voter_thrThread {
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

