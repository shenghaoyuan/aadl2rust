// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-11-12 12:15:15

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

// AADL System: integration
#[derive(Debug)]
pub struct integrationSystem {
    #[allow(dead_code)]
    pub obstacle_camera: cameraDevice,// 子组件设备（obstacle_camera : device camera）
    #[allow(dead_code)]
    pub obstacle_radar: radarDevice,// 子组件设备（obstacle_radar : device radar）
    #[allow(dead_code)]
    pub wheel_sensor: speed_wheel_sensorDevice,// 子组件设备（wheel_sensor : device speed_wheel_sensor）
    #[allow(dead_code)]
    pub laser_sensor: speed_laser_sensorDevice,// 子组件设备（laser_sensor : device speed_laser_sensor）
    #[allow(dead_code)]
    pub panel: panelDevice,// 子组件设备（panel : device panel）
    #[allow(dead_code)]
    pub tire_pressure: tpmsDevice,// 子组件设备（tire_pressure : device tpms）
    #[allow(dead_code)]
    pub bluetooth_ctrl: bluetooth_controllerDevice,// 子组件设备（bluetooth_ctrl : device bluetooth_controller）
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
    pub brake: brakeDevice,// 子组件设备（brake : device brake）
    #[allow(dead_code)]
    pub acceleration: accelerationDevice,// 子组件设备（acceleration : device acceleration）
    #[allow(dead_code)]
    pub screen: screenDevice,// 子组件设备（screen : device screen）
    #[allow(dead_code)]
    pub speaker: speakerDevice,// 子组件设备（speaker : device speaker）
}

impl System for integrationSystem {
    // Creates a new system instance
    fn new() -> Self {
        let mut obstacle_camera: cameraDevice = cameraDevice::new();
        let mut obstacle_radar: radarDevice = radarDevice::new();
        let mut wheel_sensor: speed_wheel_sensorDevice = speed_wheel_sensorDevice::new();
        let mut laser_sensor: speed_laser_sensorDevice = speed_laser_sensorDevice::new();
        let mut panel: panelDevice = panelDevice::new();
        let mut tire_pressure: tpmsDevice = tpmsDevice::new();
        let mut bluetooth_ctrl: bluetooth_controllerDevice = bluetooth_controllerDevice::new();
        let mut image_acquisition: image_acquisitionProcess = image_acquisitionProcess::new(0);
        let mut obstacle_detection: obstacle_detectionProcess = obstacle_detectionProcess::new(0);
        let mut panel_controller: panel_controlProcess = panel_controlProcess::new(0);
        let mut speed_voter: speed_voterProcess = speed_voterProcess::new(0);
        let mut speed_ctrl: speed_controllerProcess = speed_controllerProcess::new(0);
        let mut entertainment: entertainmentProcess = entertainmentProcess::new(0);
        let mut brake: brakeDevice = brakeDevice::new();
        let mut acceleration: accelerationDevice = accelerationDevice::new();
        let mut screen: screenDevice = screenDevice::new();
        let mut speaker: speakerDevice = speakerDevice::new();
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            obstacle_camera.picture = Some(channel.0);
        // build connection: 
            image_acquisition.picture = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            image_acquisition.obstacle_detected = Some(channel.0);
        // build connection: 
            obstacle_detection.camera = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            obstacle_radar.distance_estimate = Some(channel.0);
        // build connection: 
            obstacle_detection.radar = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            obstacle_detection.obstacle_position = Some(channel.0);
        // build connection: 
            speed_ctrl.obstacle_position = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            wheel_sensor.speed = Some(channel.0);
        // build connection: 
            speed_voter.wheel_sensor = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            laser_sensor.speed = Some(channel.0);
        // build connection: 
            speed_voter.laser_sensor = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            speed_voter.speed = Some(channel.0);
        // build connection: 
            speed_ctrl.current_speed = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            speed_voter.speed = Some(channel.0);
        // build connection: 
            screen.actual_speed = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            speed_ctrl.speed_cmd = Some(channel.0);
        // build connection: 
            acceleration.cmd = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            speed_ctrl.brake_cmd = Some(channel.0);
        // build connection: 
            brake.cmd = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            speed_ctrl.warning = Some(channel.0);
        // build connection: 
            screen.warning = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            panel.increase_speed = Some(channel.0);
        // build connection: 
            panel_controller.increase_speed = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            panel.decrease_speed = Some(channel.0);
        // build connection: 
            panel_controller.decrease_speed = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            panel_controller.desired_speed = Some(channel.0);
        // build connection: 
            speed_ctrl.desired_speed = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            speed_voter.speed = Some(channel.0);
        // build connection: 
            panel_controller.current_speed = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            panel_controller.desired_speed = Some(channel.0);
        // build connection: 
            screen.desired_speed = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            bluetooth_ctrl.contacts = Some(channel.0);
        // build connection: 
            entertainment.contacts = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            bluetooth_ctrl.music = Some(channel.0);
        // build connection: 
            entertainment.music_in = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            entertainment.music_out = Some(channel.0);
        // build connection: 
            speaker.music = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            entertainment.infos = Some(channel.0);
        // build connection: 
            screen.entertainment_infos = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            panel_controller.tire_pressure_out = Some(channel.0);
        // build connection: 
            screen.tire_pressure = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            tire_pressure.pressure = Some(channel.0);
        // build connection: 
            panel_controller.tire_pressure_in = Some(channel.1);
        return Self { obstacle_camera, obstacle_radar, wheel_sensor, laser_sensor, panel, tire_pressure, bluetooth_ctrl, image_acquisition, obstacle_detection, panel_controller, speed_voter, speed_ctrl, entertainment, brake, acceleration, screen, speaker }  //显式return;
    }
    
    // Runs the system, starts all processes
    fn run(self: Self) -> () {
        std::thread::spawn(|| self.obstacle_camera.run());;
        std::thread::spawn(|| self.obstacle_radar.run());;
        std::thread::spawn(|| self.wheel_sensor.run());;
        std::thread::spawn(|| self.laser_sensor.run());;
        std::thread::spawn(|| self.panel.run());;
        std::thread::spawn(|| self.tire_pressure.run());;
        std::thread::spawn(|| self.bluetooth_ctrl.run());;
        self.image_acquisition.start();
        self.obstacle_detection.start();
        self.panel_controller.start();
        self.speed_voter.start();
        self.speed_ctrl.start();
        self.entertainment.start();
        std::thread::spawn(|| self.brake.run());;
        std::thread::spawn(|| self.acceleration.run());;
        std::thread::spawn(|| self.screen.run());;
        std::thread::spawn(|| self.speaker.run());;
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

