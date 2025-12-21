// Auto-generated from AADL package: aadlbook_devices
// 生成时间: 2025-12-21 19:44:32

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
use crate::aadlbook_platform::*;
// ---------------- cpu ----------------
fn set_thread_affinity(cpu: isize) {
    unsafe {
        let mut cpuset: cpu_set_t = std::mem::zeroed();
        CPU_ZERO(&mut cpuset);
        CPU_SET(cpu as usize, &mut cpuset);
        sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset);
    }
}

// AADL Device: camera
#[derive(Debug)]
pub struct cameraDevice {
    pub picture: Option<Sender<[[i32; 4]; 4]>>,// Port: picture Out
    pub period_ms: u64,// 周期：2000ms
}

impl Device for cameraDevice {
    // Creates a new device instance
    fn new() -> Self {
        return Self {
            picture: None,
            period_ms: 2000,
        };
    }
    
    // Device execution entry point - periodically generates and sends data
    fn run(self: Self) -> () {
        let period: std::time::Duration = Duration::from_millis(self.period_ms);
        let mut rng = rand::thread_rng();
        loop {
            let start = Instant::now();
            let picture_val = 0;
            if let Some(tx) = &self.picture {
                let _ = tx.send(picture_val);
                println!("[camera] send picture = {:?}", picture_val);
            };
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period.saturating_sub(elapsed));
            };
        };
    }
    
}

// AADL Device: radar
#[derive(Debug)]
pub struct radarDevice {
    pub distance_estimate: Option<Sender<bool>>,// Port: distance_estimate Out
    pub period_ms: u64,// 周期：1000ms
}

impl Device for radarDevice {
    // Creates a new device instance
    fn new() -> Self {
        return Self {
            distance_estimate: None,
            period_ms: 1000,
        };
    }
    
    // Device execution entry point - periodically generates and sends data
    fn run(self: Self) -> () {
        let period: std::time::Duration = Duration::from_millis(self.period_ms);
        let mut rng = rand::thread_rng();
        loop {
            let start = Instant::now();
            let distance_estimate_val = rng.gen_bool(0.9);
            if let Some(tx) = &self.distance_estimate {
                let _ = tx.send(distance_estimate_val);
                println!("[radar] send distance_estimate = {:?}", distance_estimate_val);
            };
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period.saturating_sub(elapsed));
            };
        };
    }
    
}

// AADL Device: speed_wheel_sensor
#[derive(Debug)]
pub struct speed_wheel_sensorDevice {
    pub speed: Option<Sender<u16>>,// Port: speed Out
    pub period_ms: u64,// 周期：1000ms
}

impl Device for speed_wheel_sensorDevice {
    // Creates a new device instance
    fn new() -> Self {
        return Self {
            speed: None,
            period_ms: 1000,
        };
    }
    
    // Device execution entry point - periodically generates and sends data
    fn run(self: Self) -> () {
        let period: std::time::Duration = Duration::from_millis(self.period_ms);
        let mut rng = rand::thread_rng();
        loop {
            let start = Instant::now();
            let speed_val = rng.gen_range(0, 201);
            if let Some(tx) = &self.speed {
                let _ = tx.send(speed_val);
                println!("[speed_wheel_sensor] send speed = {:?}", speed_val);
            };
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period.saturating_sub(elapsed));
            };
        };
    }
    
}

// AADL Device: speed_laser_sensor
#[derive(Debug)]
pub struct speed_laser_sensorDevice {
    pub speed: Option<Sender<u16>>,// Port: speed Out
    pub period_ms: u64,// 周期：1000ms
}

impl Device for speed_laser_sensorDevice {
    // Creates a new device instance
    fn new() -> Self {
        return Self {
            speed: None,
            period_ms: 1000,
        };
    }
    
    // Device execution entry point - periodically generates and sends data
    fn run(self: Self) -> () {
        let period: std::time::Duration = Duration::from_millis(self.period_ms);
        let mut rng = rand::thread_rng();
        loop {
            let start = Instant::now();
            let speed_val = rng.gen_range(0, 201);
            if let Some(tx) = &self.speed {
                let _ = tx.send(speed_val);
                println!("[speed_laser_sensor] send speed = {:?}", speed_val);
            };
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period.saturating_sub(elapsed));
            };
        };
    }
    
}

// AADL Device: brake
#[derive(Debug)]
pub struct brakeDevice {
    pub cmd: Option<Receiver<i8>>,// Port: cmd In
    pub period_ms: u64,// 周期：100ms
}

impl Device for brakeDevice {
    // Creates a new device instance
    fn new() -> Self {
        return Self {
            cmd: None,
            period_ms: 100,
        };
    }
    
    // Device execution entry point - periodically generates and sends data
    fn run(self: Self) -> () {
        let period: std::time::Duration = Duration::from_millis(self.period_ms);
        loop {
            let start = Instant::now();
            // // --- 从输入端口接收数据 ---
            if let Some(rx) = &self.cmd {
                if let Ok(cmd_in_val) = rx.try_recv() {
                    println!("[brake] Received cmd: {:?}", cmd_in_val);
                    // // TODO: 在此处加入执行逻辑
                };
            };
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period.saturating_sub(elapsed));
            };
        };
    }
    
}

// AADL Device: acceleration
#[derive(Debug)]
pub struct accelerationDevice {
    pub cmd: Option<Receiver<i8>>,// Port: cmd In
    pub period_ms: u64,// 周期：100ms
}

impl Device for accelerationDevice {
    // Creates a new device instance
    fn new() -> Self {
        return Self {
            cmd: None,
            period_ms: 100,
        };
    }
    
    // Device execution entry point - periodically generates and sends data
    fn run(self: Self) -> () {
        let period: std::time::Duration = Duration::from_millis(self.period_ms);
        loop {
            let start = Instant::now();
            // // --- 从输入端口接收数据 ---
            if let Some(rx) = &self.cmd {
                if let Ok(cmd_in_val) = rx.try_recv() {
                    println!("[acceleration] Received cmd: {:?}", cmd_in_val);
                    // // TODO: 在此处加入执行逻辑
                };
            };
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period.saturating_sub(elapsed));
            };
        };
    }
    
}

// AADL Device: panel
#[derive(Debug)]
pub struct panelDevice {
    pub increase_speed: Option<Sender<u16>>,// Port: increase_speed Out
    pub decrease_speed: Option<Sender<u16>>,// Port: decrease_speed Out
    pub period_ms: u64,// 周期：2000ms
}

impl Device for panelDevice {
    // Creates a new device instance
    fn new() -> Self {
        return Self {
            increase_speed: None,
            decrease_speed: None,
            period_ms: 2000,
        };
    }
    
    // Device execution entry point - periodically generates and sends data
    fn run(self: Self) -> () {
        let period: std::time::Duration = Duration::from_millis(self.period_ms);
        let mut rng = rand::thread_rng();
        loop {
            let start = Instant::now();
            let increase_speed_val = rng.gen_range(0, 201);
            if let Some(tx) = &self.increase_speed {
                let _ = tx.send(increase_speed_val);
                println!("[panel] send increase_speed = {:?}", increase_speed_val);
            };
            let decrease_speed_val = rng.gen_range(0, 201);
            if let Some(tx) = &self.decrease_speed {
                let _ = tx.send(decrease_speed_val);
                println!("[panel] send decrease_speed = {:?}", decrease_speed_val);
            };
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period.saturating_sub(elapsed));
            };
        };
    }
    
}

// AADL Device: screen
#[derive(Debug)]
pub struct screenDevice {
    pub tire_pressure: Option<Receiver<i8>>,// Port: tire_pressure In
    pub desired_speed: Option<BcReceiver<u16>>,// Port: desired_speed In
    pub actual_speed: Option<BcReceiver<u16>>,// Port: actual_speed In
    pub warning: Option<Receiver<bool>>,// Port: warning In
    pub entertainment_infos: Option<Receiver<i8>>,// Port: entertainment_infos In
    pub period_ms: u64,// 周期：2000ms
}

impl Device for screenDevice {
    // Creates a new device instance
    fn new() -> Self {
        return Self {
            tire_pressure: None,
            desired_speed: None,
            actual_speed: None,
            warning: None,
            entertainment_infos: None,
            period_ms: 2000,
        };
    }
    
    // Device execution entry point - periodically generates and sends data
    fn run(self: Self) -> () {
        let period: std::time::Duration = Duration::from_millis(self.period_ms);
        loop {
            let start = Instant::now();
            // // --- 从输入端口接收数据 ---
            if let Some(rx) = &self.tire_pressure {
                if let Ok(tire_pressure_in_val) = rx.try_recv() {
                    println!("[screen] Received tire_pressure: {:?}", tire_pressure_in_val);
                    // // TODO: 在此处加入执行逻辑
                };
            };
            if let Some(rx) = &self.desired_speed {
                if let Ok(desired_speed_in_val) = rx.try_recv() {
                    println!("[screen] Received desired_speed: {:?}", desired_speed_in_val);
                    // // TODO: 在此处加入执行逻辑
                };
            };
            if let Some(rx) = &self.actual_speed {
                if let Ok(actual_speed_in_val) = rx.try_recv() {
                    println!("[screen] Received actual_speed: {:?}", actual_speed_in_val);
                    // // TODO: 在此处加入执行逻辑
                };
            };
            if let Some(rx) = &self.warning {
                if let Ok(warning_in_val) = rx.try_recv() {
                    println!("[screen] Received warning: {:?}", warning_in_val);
                    // // TODO: 在此处加入执行逻辑
                };
            };
            if let Some(rx) = &self.entertainment_infos {
                if let Ok(entertainment_infos_in_val) = rx.try_recv() {
                    println!("[screen] Received entertainment_infos: {:?}", entertainment_infos_in_val);
                    // // TODO: 在此处加入执行逻辑
                };
            };
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period.saturating_sub(elapsed));
            };
        };
    }
    
}

// AADL Device: tpms
#[derive(Debug)]
pub struct tpmsDevice {
    pub pressure: Option<Sender<i8>>,// Port: pressure Out
    pub period_ms: u64,// 周期：2000ms
}

impl Device for tpmsDevice {
    // Creates a new device instance
    fn new() -> Self {
        return Self {
            pressure: None,
            period_ms: 2000,
        };
    }
    
    // Device execution entry point - periodically generates and sends data
    fn run(self: Self) -> () {
        let period: std::time::Duration = Duration::from_millis(self.period_ms);
        let mut rng = rand::thread_rng();
        loop {
            let start = Instant::now();
            let pressure_val = rng.gen_range(0, 201);
            if let Some(tx) = &self.pressure {
                let _ = tx.send(pressure_val);
                println!("[tpms] send pressure = {:?}", pressure_val);
            };
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period.saturating_sub(elapsed));
            };
        };
    }
    
}

// AADL Device: bluetooth_controller
#[derive(Debug)]
pub struct bluetooth_controllerDevice {
    pub music: Option<Sender<bool>>,// Port: music Out
    pub contacts: Option<Sender<i8>>,// Port: contacts Out
    pub period_ms: u64,// 周期：2000ms
}

impl Device for bluetooth_controllerDevice {
    // Creates a new device instance
    fn new() -> Self {
        return Self {
            music: None,
            contacts: None,
            period_ms: 2000,
        };
    }
    
    // Device execution entry point - periodically generates and sends data
    fn run(self: Self) -> () {
        let period: std::time::Duration = Duration::from_millis(self.period_ms);
        let mut rng = rand::thread_rng();
        loop {
            let start = Instant::now();
            let music_val = rng.gen_bool(0.9);
            if let Some(tx) = &self.music {
                let _ = tx.send(music_val);
                println!("[bluetooth_controller] send music = {:?}", music_val);
            };
            let contacts_val = rng.gen_range(0, 201);
            if let Some(tx) = &self.contacts {
                let _ = tx.send(contacts_val);
                println!("[bluetooth_controller] send contacts = {:?}", contacts_val);
            };
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period.saturating_sub(elapsed));
            };
        };
    }
    
}

// AADL Device: speaker
#[derive(Debug)]
pub struct speakerDevice {
    pub music: Option<Receiver<bool>>,// Port: music In
    pub period_ms: u64,// 周期：2000ms
}

impl Device for speakerDevice {
    // Creates a new device instance
    fn new() -> Self {
        return Self {
            music: None,
            period_ms: 2000,
        };
    }
    
    // Device execution entry point - periodically generates and sends data
    fn run(self: Self) -> () {
        let period: std::time::Duration = Duration::from_millis(self.period_ms);
        loop {
            let start = Instant::now();
            // // --- 从输入端口接收数据 ---
            if let Some(rx) = &self.music {
                if let Ok(music_in_val) = rx.try_recv() {
                    println!("[speaker] Received music: {:?}", music_in_val);
                    // // TODO: 在此处加入执行逻辑
                };
            };
            let elapsed = start.elapsed();
            if elapsed < period {
                std::thread::sleep(period.saturating_sub(elapsed));
            };
        };
    }
    
}

// CPU ID到调度策略的映射
lazy_static! {
    static ref CPU_ID_TO_SCHED_POLICY: HashMap<isize, i32> = {
        let mut map: HashMap<isize, i32> = HashMap::new();
        map.insert(0, SCHED_FIFO);
        map.insert(1, SCHED_FIFO);
        map.insert(2, SCHED_FIFO);
        map.insert(3, SCHED_FIFO);
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

