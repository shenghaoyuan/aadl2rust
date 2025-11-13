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

// AADL Process: panel_control
#[derive(Debug)]
pub struct panel_controlProcess {
    pub increase_speed: Option<Receiver<()>>,// Port: increase_speed In
    pub decrease_speed: Option<Receiver<()>>,// Port: decrease_speed In
    pub current_speed: Option<Receiver<u16>>,// Port: current_speed In
    pub desired_speed: Option<Sender<u16>>,// Port: desired_speed Out
    pub tire_pressure_in: Option<Receiver<i8>>,// Port: tire_pressure_in In
    pub tire_pressure_out: Option<Sender<i8>>,// Port: tire_pressure_out Out
    pub cpu_id: isize,// 进程 CPU ID
    pub increase_speedSend: Option<Sender<()>>,// 内部端口: increase_speed In
    pub decrease_speedSend: Option<Sender<()>>,// 内部端口: decrease_speed In
    pub current_speedSend: Option<Sender<u16>>,// 内部端口: current_speed In
    pub desired_speedRece: Option<Receiver<u16>>,// 内部端口: desired_speed Out
    pub tire_pressure_inSend: Option<Sender<i8>>,// 内部端口: tire_pressure_in In
    pub tire_pressure_outRece: Option<Receiver<i8>>,// 内部端口: tire_pressure_out Out
    #[allow(dead_code)]
    pub thr: panel_control_thrThread,// 子组件线程（thr : thread panel_control_thr）
}

impl Process for panel_controlProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut thr: panel_control_thrThread = panel_control_thrThread::new(cpu_id);
        let mut increase_speedSend = None;
        let mut decrease_speedSend = None;
        let mut current_speedSend = None;
        let mut desired_speedRece = None;
        let mut tire_pressure_inSend = None;
        let mut tire_pressure_outRece = None;
        let channel = crossbeam_channel::unbounded();
        increase_speedSend = Some(channel.0);
        // build connection: 
            thr.increase_speed = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        decrease_speedSend = Some(channel.0);
        // build connection: 
            thr.decrease_speed = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        current_speedSend = Some(channel.0);
        // build connection: 
            thr.current_speed = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        tire_pressure_inSend = Some(channel.0);
        // build connection: 
            thr.tire_pressure_in = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            thr.tire_pressure_out = Some(channel.0);
        tire_pressure_outRece = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            thr.desired_speed = Some(channel.0);
        desired_speedRece = Some(channel.1);
        return Self { increase_speed: None, increase_speedSend, decrease_speed: None, decrease_speedSend, current_speed: None, current_speedSend, desired_speed: None, desired_speedRece, tire_pressure_in: None, tire_pressure_inSend, tire_pressure_out: None, tire_pressure_outRece, thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn start(self: Self) -> () {
        let Self { increase_speed, increase_speedSend, decrease_speed, decrease_speedSend, current_speed, current_speedSend, desired_speed, desired_speedRece, tire_pressure_in, tire_pressure_inSend, tire_pressure_out, tire_pressure_outRece, thr, cpu_id, .. } = self;
        thread::Builder::new()
            .name("thr".to_string())
            .spawn(|| { thr.run() }).unwrap();
        let increase_speed_rx = increase_speed.unwrap();
        thread::Builder::new()
            .name("data_forwarder_increase_speed".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = increase_speed_rx.try_recv() {
                    if let Some(tx) = &increase_speedSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let decrease_speed_rx = decrease_speed.unwrap();
        thread::Builder::new()
            .name("data_forwarder_decrease_speed".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = decrease_speed_rx.try_recv() {
                    if let Some(tx) = &decrease_speedSend {
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
        let tire_pressure_in_rx = tire_pressure_in.unwrap();
        thread::Builder::new()
            .name("data_forwarder_tire_pressure_in".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = tire_pressure_in_rx.try_recv() {
                    if let Some(tx) = &tire_pressure_inSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let tire_pressure_outRece_rx = tire_pressure_outRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_tire_pressure_outRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = tire_pressure_outRece_rx.try_recv() {
                    if let Some(tx) = &tire_pressure_out {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let desired_speedRece_rx = desired_speedRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_desired_speedRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = desired_speedRece_rx.try_recv() {
                    if let Some(tx) = &desired_speed {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
    }
    
}

// AADL Thread: panel_control_thr
#[derive(Debug)]
pub struct panel_control_thrThread {
    pub increase_speed: Option<Receiver<()>>,// Port: increase_speed In
    pub decrease_speed: Option<Receiver<()>>,// Port: decrease_speed In
    pub current_speed: Option<Receiver<u16>>,// Port: current_speed In
    pub desired_speed: Option<Sender<u16>>,// Port: desired_speed Out
    pub tire_pressure_in: Option<Receiver<i8>>,// Port: tire_pressure_in In
    pub tire_pressure_out: Option<Sender<i8>>,// Port: tire_pressure_out Out
    pub cpu_id: isize,// 结构体新增 CPU ID
}

impl Thread for panel_control_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            decrease_speed: None, 
            increase_speed: None, 
            tire_pressure_out: None, 
            desired_speed: None, 
            tire_pressure_in: None, 
            current_speed: None, 
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
            // State: s2
            s2,
        }
        
        let mut state: State = State::s0;
        loop {
            let start = Instant::now();
            let current_speed_val = self.current_speed.as_ref().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
            let tire_pressure_in_val = self.tire_pressure_in.as_ref().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
            {
                // --- BA 宏步执行 ---
                loop {
                    match state {
                        State::s0 if 0 < current_speed_val => {
                            if let Some(sender) = &self.desired_speed {
                                let _ = sender.send(current_speed + 1);
                            };
                            state = State::s1;
                            continue;
                        },
                        State::s1 if 0 < tire_pressure_in_val => {
                            if let Some(sender) = &self.tire_pressure_out {
                                let _ = sender.send(tire_pressure_in);
                            };
                            state = State::s2;
                            continue;
                        },
                        State::s2 => {
                            state = State::s0;
                            // complete，停
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

