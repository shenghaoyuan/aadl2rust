// Auto-generated from AADL package: aadlbook_software_speed_controller
// 生成时间: 2025-12-20 18:11:10

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

use crate::aadlbook_icd::*;
use crate::sei::*;
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
    pub obstacle_position: Option<Receiver<bool>>,// Port: obstacle_position In
    pub current_speed: Option<BcReceiver<u16>>,// Port: current_speed In
    pub desired_speed: Option<BcReceiver<u16>>,// Port: desired_speed In
    pub brake_cmd: Option<Sender<i8>>,// Port: brake_cmd Out
    pub speed_cmd: Option<Sender<i8>>,// Port: speed_cmd Out
    pub warning: Option<Sender<bool>>,// Port: warning Out
    pub cpu_id: isize,// 进程 CPU ID
    pub obstacle_positionSend: Option<BcSender<bool>>,// 内部端口: obstacle_position In
    pub current_speedSend: Option<BcSender<u16>>,// 内部端口: current_speed In
    pub desired_speedSend: Option<BcSender<u16>>,// 内部端口: desired_speed In
    pub brake_cmdRece: Option<Receiver<i8>>,// 内部端口: brake_cmd Out
    pub speed_cmdRece: Option<Receiver<i8>>,// 内部端口: speed_cmd Out
    pub warningRece: Option<Receiver<bool>>,// 内部端口: warning Out
    pub accel_thr: speed_controller_accel_thrThread,// 子组件线程（accel_thr : thread speed_controller_accel_thr）
    pub brake_thr: speed_controller_brake_thrThread,// 子组件线程（brake_thr : thread speed_controller_brake_thr）
    pub warning_thr: speed_controller_warning_thrThread,// 子组件线程（warning_thr : thread speed_controller_warning_thr）
}

impl Process for speed_controllerProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let accel_thr: speed_controller_accel_thrThread = speed_controller_accel_thrThread::new(cpu_id);
        let brake_thr: speed_controller_brake_thrThread = speed_controller_brake_thrThread::new(cpu_id);
        let warning_thr: speed_controller_warning_thrThread = speed_controller_warning_thrThread::new(cpu_id);
        let mut obstacle_positionSend = None;
        let mut current_speedSend = None;
        let mut desired_speedSend = None;
        let mut brake_cmdRece = None;
        let mut speed_cmdRece = None;
        let mut warningRece = None;
        let channel = broadcast::channel::<>(100);
        obstacle_positionSend = Some(channel.0.clone());
        // build connection: 
            accel_thr.obstacle_position = Some(channel.0.subscribe());
        // build connection: 
            brake_thr.obstacle_position = Some(channel.0.subscribe());
        // build connection: 
            warning_thr.obstacle_position = Some(channel.0.subscribe());
        let channel = broadcast::channel::<>(100);
        current_speedSend = Some(channel.0.clone());
        // build connection: 
            accel_thr.current_speed = Some(channel.0.subscribe());
        // build connection: 
            brake_thr.current_speed = Some(channel.0.subscribe());
        // build connection: 
            warning_thr.current_speed = Some(channel.0.subscribe());
        let channel = broadcast::channel::<>(100);
        desired_speedSend = Some(channel.0.clone());
        // build connection: 
            accel_thr.desired_speed = Some(channel.0.subscribe());
        // build connection: 
            brake_thr.desired_speed = Some(channel.0.subscribe());
        // build connection: 
            warning_thr.desired_speed = Some(channel.0.subscribe());
        let c03 = crossbeam_channel::unbounded();
        // build connection: 
            accel_thr.speed_cmd = Some(c03.0);
        speed_cmdRece = Some(c03.1);
        let c13 = crossbeam_channel::unbounded();
        // build connection: 
            brake_thr.brake_cmd = Some(c13.0);
        brake_cmdRece = Some(c13.1);
        let c23 = crossbeam_channel::unbounded();
        // build connection: 
            warning_thr.warning = Some(c23.0);
        warningRece = Some(c23.1);
        return Self { obstacle_position: None, obstacle_positionSend, current_speed: None, current_speedSend, desired_speed: None, desired_speedSend, brake_cmd: None, brake_cmdRece, speed_cmd: None, speed_cmdRece, warning: None, warningRece, accel_thr, brake_thr, warning_thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn run(self: Self) -> () {
        let Self { obstacle_position, obstacle_positionSend, current_speed, current_speedSend, desired_speed, desired_speedSend, brake_cmd, brake_cmdRece, speed_cmd, speed_cmdRece, warning, warningRece, accel_thr, brake_thr, warning_thr, cpu_id, .. } = self;
        thread::Builder::new()
            .name("accel_thr".to_string())
            .spawn(move || { accel_thr.run() }).unwrap();
        thread::Builder::new()
            .name("brake_thr".to_string())
            .spawn(move || { brake_thr.run() }).unwrap();
        thread::Builder::new()
            .name("warning_thr".to_string())
            .spawn(move || { warning_thr.run() }).unwrap();
        let mut brake_cmdRece_rx = brake_cmdRece.unwrap();
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
        let mut current_speed_rx = current_speed.unwrap();
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
        let mut desired_speed_rx = desired_speed.unwrap();
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
        let mut obstacle_position_rx = obstacle_position.unwrap();
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
        let mut speed_cmdRece_rx = speed_cmdRece.unwrap();
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
        let mut warningRece_rx = warningRece.unwrap();
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
    pub obstacle_position: Option<BcReceiver<bool>>,// Port: obstacle_position In
    pub current_speed: Option<BcReceiver<u16>>,// Port: current_speed In
    pub desired_speed: Option<BcReceiver<u16>>,// Port: desired_speed In
    pub warning: Option<Sender<bool>>,// Port: warning Out
    pub dispatch_protocol: String,// AADL属性: Dispatch_Protocol
    pub period: u64,// AADL属性: Period
    pub mipsbudget: f64,// AADL属性: mipsbudget
    pub cpu_id: isize,// 结构体新增 CPU ID
}

// AADL Thread: speed_controller_brake_thr
#[derive(Debug)]
pub struct speed_controller_brake_thrThread {
    pub obstacle_position: Option<BcReceiver<bool>>,// Port: obstacle_position In
    pub current_speed: Option<BcReceiver<u16>>,// Port: current_speed In
    pub desired_speed: Option<BcReceiver<u16>>,// Port: desired_speed In
    pub brake_cmd: Option<Sender<i8>>,// Port: brake_cmd Out
    pub dispatch_protocol: String,// AADL属性: Dispatch_Protocol
    pub period: u64,// AADL属性: Period
    pub mipsbudget: f64,// AADL属性: mipsbudget
    pub cpu_id: isize,// 结构体新增 CPU ID
}

// AADL Thread: speed_controller_accel_thr
#[derive(Debug)]
pub struct speed_controller_accel_thrThread {
    pub obstacle_position: Option<BcReceiver<bool>>,// Port: obstacle_position In
    pub current_speed: Option<BcReceiver<u16>>,// Port: current_speed In
    pub desired_speed: Option<BcReceiver<u16>>,// Port: desired_speed In
    pub speed_cmd: Option<Sender<i8>>,// Port: speed_cmd Out
    pub dispatch_protocol: String,// AADL属性: Dispatch_Protocol
    pub period: u64,// AADL属性: Period
    pub mipsbudget: f64,// AADL属性: mipsbudget
    pub cpu_id: isize,// 结构体新增 CPU ID
}

impl Thread for speed_controller_accel_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            obstacle_position: None, 
            dispatch_protocol: "Periodic".to_string(), 
            current_speed: None, 
            speed_cmd: None, 
            mipsbudget: 5.0, 
            period: 5, 
            desired_speed: None, 
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
        #[derive(Debug, Clone)]
        enum State {
            // State: s0
            s0,
        }
        
        let mut state: State = State::s0;
        loop {
            let start = Instant::now();
            let current_speed = self.current_speed.as_mut().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
            {
                // --- BA 宏步执行 ---
                loop {
                    match state {
                        State::s0 if 60 < current_speed => {
                            if let Some(sender) = &self.speed_cmd {
                                let _ = sender.send(0);
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

impl Thread for speed_controller_brake_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            obstacle_position: None, 
            current_speed: None, 
            dispatch_protocol: "Periodic".to_string(), 
            period: 5, 
            desired_speed: None, 
            mipsbudget: 5.0, 
            brake_cmd: None, 
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
        #[derive(Debug, Clone)]
        enum State {
            // State: s0
            s0,
        }
        
        let mut state: State = State::s0;
        loop {
            let start = Instant::now();
            let obstacle_position = self.obstacle_position.as_mut().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
            {
                // --- BA 宏步执行 ---
                loop {
                    match state {
                        State::s0 if obstacle_position == true => {
                            if let Some(sender) = &self.brake_cmd {
                                let _ = sender.send(1);
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

impl Thread for speed_controller_warning_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            desired_speed: None, 
            mipsbudget: 5.0, 
            warning: None, 
            obstacle_position: None, 
            current_speed: None, 
            period: 5, 
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
        let mut next_release = Instant::now() + period;
        // Behavior Annex state machine states
        #[derive(Debug, Clone)]
        enum State {
            // State: s0
            s0,
        }
        
        let mut state: State = State::s0;
        loop {
            let start = Instant::now();
            let obstacle_position = self.obstacle_position.as_mut().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
            {
                // --- BA 宏步执行 ---
                loop {
                    match state {
                        State::s0 if obstacle_position == true => {
                            if let Some(sender) = &self.warning {
                                let _ = sender.send(true);
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

