// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-12-04 21:01:10

#![allow(unused_imports)]
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc,Mutex};
use std::thread;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::common_traits::*;
use tokio::sync::broadcast::{self,Sender as BcSender, Receiver as BcReceiver};
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

// AADL System: integration
#[derive(Debug)]
pub struct integrationSystem {
    #[allow(dead_code)]
    pub obstacle_camera_dev: cameraDevice,// 子组件设备（obstacle_camera_dev : device camera）
    #[allow(dead_code)]
    pub obstacle_radar_dev: radarDevice,// 子组件设备（obstacle_radar_dev : device radar）
    #[allow(dead_code)]
    pub wheel_sensor_dev: speed_wheel_sensorDevice,// 子组件设备（wheel_sensor_dev : device speed_wheel_sensor）
    #[allow(dead_code)]
    pub laser_sensor_dev: speed_laser_sensorDevice,// 子组件设备（laser_sensor_dev : device speed_laser_sensor）
    #[allow(dead_code)]
    pub panel_dev: panelDevice,// 子组件设备（panel_dev : device panel）
    #[allow(dead_code)]
    pub tire_pressure_dev: tpmsDevice,// 子组件设备（tire_pressure_dev : device tpms）
    #[allow(dead_code)]
    pub bluetooth_ctrl_dev: bluetooth_controllerDevice,// 子组件设备（bluetooth_ctrl_dev : device bluetooth_controller）
    #[allow(dead_code)]
    pub image_acquisition: image_acquisitionProcess,// 子组件进程（image_acquisition : process image_acquisition）
    #[allow(dead_code)]
    pub obstacle_detection: obstacle_detectionProcess,// 子组件进程（obstacle_detection : process obstacle_detection）
    #[allow(dead_code)]
    pub panel_controller: panel_controlProcess,// 子组件进程（panel_controller : process panel_control）
    #[allow(dead_code)]
    pub speed_voter: speed_voterProcess,// 子组件进程（speed_voter : process speed_voter）
    #[allow(dead_code)]
    pub speed_ctrl: speed_controllerProcess,// 子组件进程（speed_ctrl : process speed_controller）
    #[allow(dead_code)]
    pub entertainment: entertainmentProcess,// 子组件进程（entertainment : process entertainment）
    #[allow(dead_code)]
    pub brake_dev: brakeDevice,// 子组件设备（brake_dev : device brake）
    #[allow(dead_code)]
    pub acceleration_dev: accelerationDevice,// 子组件设备（acceleration_dev : device acceleration）
    #[allow(dead_code)]
    pub screen_dev: screenDevice,// 子组件设备（screen_dev : device screen）
    #[allow(dead_code)]
    pub speaker_dev: speakerDevice,// 子组件设备（speaker_dev : device speaker）
}

impl System for integrationSystem {
    // Creates a new system instance
    fn new() -> Self {
        let mut obstacle_camera_dev: cameraDevice = cameraDevice::new();
        let mut obstacle_radar_dev: radarDevice = radarDevice::new();
        let mut wheel_sensor_dev: speed_wheel_sensorDevice = speed_wheel_sensorDevice::new();
        let mut laser_sensor_dev: speed_laser_sensorDevice = speed_laser_sensorDevice::new();
        let mut panel_dev: panelDevice = panelDevice::new();
        let mut tire_pressure_dev: tpmsDevice = tpmsDevice::new();
        let mut bluetooth_ctrl_dev: bluetooth_controllerDevice = bluetooth_controllerDevice::new();
        let mut image_acquisition: image_acquisitionProcess = image_acquisitionProcess::new(0);
        let mut obstacle_detection: obstacle_detectionProcess = obstacle_detectionProcess::new(0);
        let mut panel_controller: panel_controlProcess = panel_controlProcess::new(0);
        let mut speed_voter: speed_voterProcess = speed_voterProcess::new(0);
        let mut speed_ctrl: speed_controllerProcess = speed_controllerProcess::new(0);
        let mut entertainment: entertainmentProcess = entertainmentProcess::new(0);
        let mut brake_dev: brakeDevice = brakeDevice::new();
        let mut acceleration_dev: accelerationDevice = accelerationDevice::new();
        let mut screen_dev: screenDevice = screenDevice::new();
        let mut speaker_dev: speakerDevice = speakerDevice::new();
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            obstacle_camera_dev.picture = Some(channel.0);
        // build connection: 
            image_acquisition.picture = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            image_acquisition.obstacle_detected = Some(channel.0);
        // build connection: 
            obstacle_detection.camera = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            obstacle_radar_dev.distance_estimate = Some(channel.0);
        // build connection: 
            obstacle_detection.radar = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            obstacle_detection.obstacle_position = Some(channel.0);
        // build connection: 
            speed_ctrl.obstacle_position = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            wheel_sensor_dev.speed = Some(channel.0);
        // build connection: 
            speed_voter.wheel_sensor = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            laser_sensor_dev.speed = Some(channel.0);
        // build connection: 
            speed_voter.laser_sensor = Some(channel.1);
        let channel = broadcast::channel::<>(100);
        // build connection: 
            speed_voter.speed = Some(channel.0.clone());
        // build connection: 
            speed_ctrl.current_speed = Some(channel.0.subscribe());
        // build connection: 
            screen_dev.actual_speed = Some(channel.0.subscribe());
        // build connection: 
            panel_controller.current_speed = Some(channel.0.subscribe());
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            speed_ctrl.speed_cmd = Some(channel.0);
        // build connection: 
            acceleration_dev.cmd = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            speed_ctrl.brake_cmd = Some(channel.0);
        // build connection: 
            brake_dev.cmd = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            speed_ctrl.warning = Some(channel.0);
        // build connection: 
            screen_dev.warning = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            panel_dev.increase_speed = Some(channel.0);
        // build connection: 
            panel_controller.increase_speed = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            panel_dev.decrease_speed = Some(channel.0);
        // build connection: 
            panel_controller.decrease_speed = Some(channel.1);
        let channel = broadcast::channel::<>(100);
        // build connection: 
            panel_controller.desired_speed = Some(channel.0.clone());
        // build connection: 
            speed_ctrl.desired_speed = Some(channel.0.subscribe());
        // build connection: 
            screen_dev.desired_speed = Some(channel.0.subscribe());
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            bluetooth_ctrl_dev.contacts = Some(channel.0);
        // build connection: 
            entertainment.contacts = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            bluetooth_ctrl_dev.music = Some(channel.0);
        // build connection: 
            entertainment.music_in = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            entertainment.music_out = Some(channel.0);
        // build connection: 
            speaker_dev.music = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            entertainment.infos = Some(channel.0);
        // build connection: 
            screen_dev.entertainment_infos = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            panel_controller.tire_pressure_out = Some(channel.0);
        // build connection: 
            screen_dev.tire_pressure = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            tire_pressure_dev.pressure = Some(channel.0);
        // build connection: 
            panel_controller.tire_pressure_in = Some(channel.1);
        return Self { obstacle_camera_dev, obstacle_radar_dev, wheel_sensor_dev, laser_sensor_dev, panel_dev, tire_pressure_dev, bluetooth_ctrl_dev, image_acquisition, obstacle_detection, panel_controller, speed_voter, speed_ctrl, entertainment, brake_dev, acceleration_dev, screen_dev, speaker_dev }  //显式return;
    }
    
    // Runs the system, starts all processes
    fn run(self: Self) -> () {
        thread::Builder::new()
            .name("obstacle_camera_dev".to_string())
            .spawn(move || { self.obstacle_camera_dev.run() }).unwrap();
        thread::Builder::new()
            .name("obstacle_radar_dev".to_string())
            .spawn(move || { self.obstacle_radar_dev.run() }).unwrap();
        thread::Builder::new()
            .name("wheel_sensor_dev".to_string())
            .spawn(move || { self.wheel_sensor_dev.run() }).unwrap();
        thread::Builder::new()
            .name("laser_sensor_dev".to_string())
            .spawn(move || { self.laser_sensor_dev.run() }).unwrap();
        thread::Builder::new()
            .name("panel_dev".to_string())
            .spawn(move || { self.panel_dev.run() }).unwrap();
        thread::Builder::new()
            .name("tire_pressure_dev".to_string())
            .spawn(move || { self.tire_pressure_dev.run() }).unwrap();
        thread::Builder::new()
            .name("bluetooth_ctrl_dev".to_string())
            .spawn(move || { self.bluetooth_ctrl_dev.run() }).unwrap();
        self.image_acquisition.start();
        self.obstacle_detection.start();
        self.panel_controller.start();
        self.speed_voter.start();
        self.speed_ctrl.start();
        self.entertainment.start();
        thread::Builder::new()
            .name("brake_dev".to_string())
            .spawn(move || { self.brake_dev.run() }).unwrap();
        thread::Builder::new()
            .name("acceleration_dev".to_string())
            .spawn(move || { self.acceleration_dev.run() }).unwrap();
        thread::Builder::new()
            .name("screen_dev".to_string())
            .spawn(move || { self.screen_dev.run() }).unwrap();
        thread::Builder::new()
            .name("speaker_dev".to_string())
            .spawn(move || { self.speaker_dev.run() }).unwrap();
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

