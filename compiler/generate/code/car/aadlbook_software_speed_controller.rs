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

// AADL Process: speed_controller
#[derive(Debug)]
pub struct speed_controllerProcess {
    // Port: obstacle_position In
    pub obstacle_position: Option<mpsc::Receiver<obstacle_position>>,
    // Port: current_speed In
    pub current_speed: Option<mpsc::Receiver<speed>>,
    // Port: desired_speed In
    pub desired_speed: Option<mpsc::Receiver<speed>>,
    // Port: brake_cmd Out
    pub brake_cmd: Option<mpsc::Sender<brake_cmd>>,
    // Port: speed_cmd Out
    pub speed_cmd: Option<mpsc::Sender<speed_cmd>>,
    // Port: warning Out
    pub warning: Option<mpsc::Sender<bool>>,
    // 进程 CPU ID
    pub cpu_id: isize,
    // 内部端口: obstacle_position In
    pub obstacle_positionSend: Option<mpsc::Sender<obstacle_position>>,
    // 内部端口: current_speed In
    pub current_speedSend: Option<mpsc::Sender<speed>>,
    // 内部端口: desired_speed In
    pub desired_speedSend: Option<mpsc::Sender<speed>>,
    // 内部端口: brake_cmd Out
    pub brake_cmdRece: Option<mpsc::Receiver<brake_cmd>>,
    // 内部端口: speed_cmd Out
    pub speed_cmdRece: Option<mpsc::Receiver<speed_cmd>>,
    // 内部端口: warning Out
    pub warningRece: Option<mpsc::Receiver<bool>>,
    // 子组件线程（accel_thr : thread speed_controller_accel_thr）
    #[allow(dead_code)]
    pub accel_thr: speed_controller_accel_thrThread,
    // 子组件线程（brake_thr : thread speed_controller_brake_thr）
    #[allow(dead_code)]
    pub brake_thr: speed_controller_brake_thrThread,
    // 子组件线程（warning_thr : thread speed_controller_warning_thr）
    #[allow(dead_code)]
    pub warning_thr: speed_controller_warning_thrThread,
}

impl speed_controllerProcess {
    // Creates a new process instance
    pub fn new(cpu_id: isize) -> Self {
        let mut accel_thr: speed_controller_accel_thrThread = speed_controller_accel_thrThread::new(cpu_id);
        let mut brake_thr: speed_controller_brake_thrThread = speed_controller_brake_thrThread::new(cpu_id);
        let mut warning_thr: speed_controller_warning_thrThread = speed_controller_warning_thrThread::new(cpu_id);
        let mut obstacle_positionSend = None;
        let mut current_speedSend = None;
        let mut desired_speedSend = None;
        let mut brake_cmdRece = None;
        let mut speed_cmdRece = None;
        let mut warningRece = None;
        let channel = mpsc::channel();
        obstacle_positionSend = Some(channel.0);
        // build connection: 
            accel_thr.obstacle_position = Some(channel.1);
        let channel = mpsc::channel();
        current_speedSend = Some(channel.0);
        // build connection: 
            accel_thr.current_speed = Some(channel.1);
        let channel = mpsc::channel();
        desired_speedSend = Some(channel.0);
        // build connection: 
            accel_thr.desired_speed = Some(channel.1);
        let channel = mpsc::channel();
        // build connection: 
            accel_thr.speed_cmd = Some(channel.0);
        speed_cmdRece = Some(channel.1);
        let channel = mpsc::channel();
        obstacle_positionSend = Some(channel.0);
        // build connection: 
            brake_thr.obstacle_position = Some(channel.1);
        let channel = mpsc::channel();
        current_speedSend = Some(channel.0);
        // build connection: 
            brake_thr.current_speed = Some(channel.1);
        let channel = mpsc::channel();
        desired_speedSend = Some(channel.0);
        // build connection: 
            brake_thr.desired_speed = Some(channel.1);
        let channel = mpsc::channel();
        // build connection: 
            brake_thr.brake_cmd = Some(channel.0);
        brake_cmdRece = Some(channel.1);
        let channel = mpsc::channel();
        obstacle_positionSend = Some(channel.0);
        // build connection: 
            warning_thr.obstacle_position = Some(channel.1);
        let channel = mpsc::channel();
        current_speedSend = Some(channel.0);
        // build connection: 
            warning_thr.current_speed = Some(channel.1);
        let channel = mpsc::channel();
        desired_speedSend = Some(channel.0);
        // build connection: 
            warning_thr.desired_speed = Some(channel.1);
        let channel = mpsc::channel();
        // build connection: 
            warning_thr.warning = Some(channel.0);
        warningRece = Some(channel.1);
        return Self { obstacle_position: None, obstacle_positionSend, current_speed: None, current_speedSend, desired_speed: None, desired_speedSend, brake_cmd: None, brake_cmdRece, speed_cmd: None, speed_cmdRece, warning: None, warningRece, accel_thr, brake_thr, warning_thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    pub fn start(self: Self) -> () {
        let Self { obstacle_position, obstacle_positionSend, current_speed, current_speedSend, desired_speed, desired_speedSend, brake_cmd, brake_cmdRece, speed_cmd, speed_cmdRece, warning, warningRece, accel_thr, brake_thr, warning_thr, cpu_id, .. } = self;
        thread::Builder::new()
            .name("accel_thr".to_string())
            .spawn(|| { accel_thr.run() }).unwrap();
        thread::Builder::new()
            .name("brake_thr".to_string())
            .spawn(|| { brake_thr.run() }).unwrap();
        thread::Builder::new()
            .name("warning_thr".to_string())
            .spawn(|| { warning_thr.run() }).unwrap();
        let obstacle_position_rx = obstacle_position.unwrap();
        thread::Builder::new()
            .name("data_forwarder_obstacle_position".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = obstacle_position_rx.try_recv() {
                    if let Some(tx) = &obstacle_positionSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let current_speed_rx = current_speed.unwrap();
        thread::Builder::new()
            .name("data_forwarder_current_speed".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = current_speed_rx.try_recv() {
                    if let Some(tx) = &current_speedSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let desired_speed_rx = desired_speed.unwrap();
        thread::Builder::new()
            .name("data_forwarder_desired_speed".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = desired_speed_rx.try_recv() {
                    if let Some(tx) = &desired_speedSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let speed_cmdRece_rx = speed_cmdRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_speed_cmdRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = speed_cmdRece_rx.try_recv() {
                    if let Some(tx) = &speed_cmd {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let obstacle_position_rx = obstacle_position.unwrap();
        thread::Builder::new()
            .name("data_forwarder_obstacle_position".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = obstacle_position_rx.try_recv() {
                    if let Some(tx) = &obstacle_positionSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let current_speed_rx = current_speed.unwrap();
        thread::Builder::new()
            .name("data_forwarder_current_speed".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = current_speed_rx.try_recv() {
                    if let Some(tx) = &current_speedSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let desired_speed_rx = desired_speed.unwrap();
        thread::Builder::new()
            .name("data_forwarder_desired_speed".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = desired_speed_rx.try_recv() {
                    if let Some(tx) = &desired_speedSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let brake_cmdRece_rx = brake_cmdRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_brake_cmdRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = brake_cmdRece_rx.try_recv() {
                    if let Some(tx) = &brake_cmd {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let obstacle_position_rx = obstacle_position.unwrap();
        thread::Builder::new()
            .name("data_forwarder_obstacle_position".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = obstacle_position_rx.try_recv() {
                    if let Some(tx) = &obstacle_positionSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let current_speed_rx = current_speed.unwrap();
        thread::Builder::new()
            .name("data_forwarder_current_speed".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = current_speed_rx.try_recv() {
                    if let Some(tx) = &current_speedSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let desired_speed_rx = desired_speed.unwrap();
        thread::Builder::new()
            .name("data_forwarder_desired_speed".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = desired_speed_rx.try_recv() {
                    if let Some(tx) = &desired_speedSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let warningRece_rx = warningRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_warningRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = warningRece_rx.try_recv() {
                    if let Some(tx) = &warning {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
    }
    
}

// AADL Thread: speed_controller_warning_thr
#[derive(Debug)]
pub struct speed_controller_warning_thrThread {
    // Port: obstacle_position In
    pub obstacle_position: Option<mpsc::Receiver<obstacle_position>>,
    // Port: current_speed In
    pub current_speed: Option<mpsc::Receiver<speed>>,
    // Port: desired_speed In
    pub desired_speed: Option<mpsc::Receiver<speed>>,
    // Port: warning Out
    pub warning: Option<mpsc::Sender<bool>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub mipsbudget: f64, // AADL属性: mipsbudget
}

impl speed_controller_warning_thrThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            obstacle_position: None,
            current_speed: None,
            desired_speed: None,
            warning: None,
            cpu_id: cpu_id,
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 5, // AADL属性: Period
            mipsbudget: 5, // AADL属性: mipsbudget
        }
    }
}
// AADL Thread: speed_controller_brake_thr
#[derive(Debug)]
pub struct speed_controller_brake_thrThread {
    // Port: obstacle_position In
    pub obstacle_position: Option<mpsc::Receiver<obstacle_position>>,
    // Port: current_speed In
    pub current_speed: Option<mpsc::Receiver<speed>>,
    // Port: desired_speed In
    pub desired_speed: Option<mpsc::Receiver<speed>>,
    // Port: brake_cmd Out
    pub brake_cmd: Option<mpsc::Sender<brake_cmd>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub mipsbudget: f64, // AADL属性: mipsbudget
}

impl speed_controller_brake_thrThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            obstacle_position: None,
            current_speed: None,
            desired_speed: None,
            brake_cmd: None,
            cpu_id: cpu_id,
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 5, // AADL属性: Period
            mipsbudget: 5, // AADL属性: mipsbudget
        }
    }
}
// AADL Thread: speed_controller_accel_thr
#[derive(Debug)]
pub struct speed_controller_accel_thrThread {
    // Port: obstacle_position In
    pub obstacle_position: Option<mpsc::Receiver<obstacle_position>>,
    // Port: current_speed In
    pub current_speed: Option<mpsc::Receiver<speed>>,
    // Port: desired_speed In
    pub desired_speed: Option<mpsc::Receiver<speed>>,
    // Port: speed_cmd Out
    pub speed_cmd: Option<mpsc::Sender<speed_cmd>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
    pub mipsbudget: f64, // AADL属性: mipsbudget
}

impl speed_controller_accel_thrThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            obstacle_position: None,
            current_speed: None,
            desired_speed: None,
            speed_cmd: None,
            cpu_id: cpu_id,
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 5, // AADL属性: Period
            mipsbudget: 5, // AADL属性: mipsbudget
        }
    }
}
