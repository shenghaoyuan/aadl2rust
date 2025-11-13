// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-11-13 19:47:35

#![allow(unused_imports)]
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc,Mutex};
use std::thread;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::common_traits::*;
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
    pub wheel_sensor: Option<Receiver<u16>>,// Port: wheel_sensor In
    pub laser_sensor: Option<Receiver<u16>>,// Port: laser_sensor In
    pub speed: Option<Sender<u16>>,// Port: speed Out
    pub cpu_id: isize,// 进程 CPU ID
    pub wheel_sensorSend: Option<Sender<u16>>,// 内部端口: wheel_sensor In
    pub laser_sensorSend: Option<Sender<u16>>,// 内部端口: laser_sensor In
    pub speedRece: Option<Receiver<u16>>,// 内部端口: speed Out
    #[allow(dead_code)]
    pub thr: speed_voter_thrThread,// 子组件线程（thr : thread speed_voter_thr）
}

impl Process for speed_voterProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut thr: speed_voter_thrThread = speed_voter_thrThread::new(cpu_id);
        let mut wheel_sensorSend = None;
        let mut laser_sensorSend = None;
        let mut speedRece = None;
        let channel = crossbeam_channel::unbounded();
        wheel_sensorSend = Some(channel.0);
        // build connection: 
            thr.wheel_sensor = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        laser_sensorSend = Some(channel.0);
        // build connection: 
            thr.laser_sensor = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            thr.speed = Some(channel.0);
        speedRece = Some(channel.1);
        return Self { wheel_sensor: None, wheel_sensorSend, laser_sensor: None, laser_sensorSend, speed: None, speedRece, thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn start(self: Self) -> () {
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
    pub wheel_sensor: Option<Receiver<u16>>,// Port: wheel_sensor In
    pub laser_sensor: Option<Receiver<u16>>,// Port: laser_sensor In
    pub speed: Option<Sender<u16>>,// Port: speed Out
    pub dispatch_protocol: String,// AADL属性: Dispatch_Protocol
    pub period: u64,// AADL属性: Period
    pub mipsbudget: f64,// AADL属性: mipsbudget
    pub cpu_id: isize,// 结构体新增 CPU ID
}

impl Thread for speed_voter_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            period: 8, 
            speed: None, 
            laser_sensor: None, 
            mipsbudget: 8.0, 
            dispatch_protocol: "Periodic".to_string(), 
            wheel_sensor: None, 
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
        let mut speed_value: u16 = 0;
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
            let laser_sensor_val = self.laser_sensor.as_ref().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
            let wheel_sensor_val = self.wheel_sensor.as_ref().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
            {
                // --- BA 宏步执行 ---
                loop {
                    match state {
                        State::s0 if 0 < wheel_sensor_val => {
                            speed_value = wheel_sensor;
                            state = State::s1;
                            continue;
                        },
                        State::s1 if 0 < laser_sensor_val => {
                            speed_value = laser_sensor + speed_value;
                            if let Some(sender) = &self.speed {
                                let _ = sender.send(speed_value / 2);
                            };
                            state = State::s0;
                            // complete,需要停
                        },
                        State::s0 => {
                            // 理论上不会执行到这里，但编译器需要这个分支
                            break;
                        },
                        State::s1 => {
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

