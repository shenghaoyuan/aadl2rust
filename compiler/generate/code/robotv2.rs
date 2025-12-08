// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-12-08 23:04:36

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

// AADL Data Type: Alpha_Type
pub type Alpha_Type = bool;

pub mod action_spg {
    // Auto-generated from AADL subprogram: action_spg
    // C binding to: action
    // source_files: robot.c
    use super::{action};
    // Wrapper for C function action
    // Original AADL port: d_action
    pub fn receive(d_action: bool) -> () {
        unsafe { action(d_action);
         };
    }
    
}

// AADL Thread: capteur
#[derive(Debug)]
pub struct capteurThread {
    pub evenement: Option<Sender<bool>>,// Port: evenement Out
    pub cpu_id: isize,// 结构体新增 CPU ID
    pub dispatch_protocol: String,// AADL属性(impl): Dispatch_Protocol
    pub period: u64,// AADL属性(impl): Period
}

// AADL Thread: controle
#[derive(Debug)]
pub struct controleThread {
    pub info_capteur: Option<Receiver<bool>>,// Port: info_capteur In
    pub comm_servo: Option<Sender<bool>>,// Port: comm_servo Out
    pub cpu_id: isize,// 结构体新增 CPU ID
    pub dispatch_protocol: String,// AADL属性(impl): Dispatch_Protocol
    pub period: u64,// AADL属性(impl): Period
}

// AADL Thread: servomoteur
#[derive(Debug)]
pub struct servomoteurThread {
    pub ordre: Option<Receiver<bool>>,// Port: ordre In
    pub cpu_id: isize,// 结构体新增 CPU ID
    pub dispatch_protocol: String,// AADL属性(impl): Dispatch_Protocol
    pub period: u64,// AADL属性(impl): Period
}

impl Thread for controleThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            info_capteur: None, 
            comm_servo: None, 
            dispatch_protocol: "Periodic".to_string(), 
            period: 110, 
            cpu_id: cpu_id, // CPU ID
        };
    }
    
    // Thread execution entry point
    // Period: Some(110) ms
    fn run(mut self) -> () {
        unsafe {
            let prio = period_to_priority(self.period as f64);
            let mut param: sched_param = sched_param { sched_priority: prio };
            let ret = pthread_setschedparam(pthread_self(), *CPU_ID_TO_SCHED_POLICY.get(&self.cpu_id).unwrap_or(&SCHED_FIFO), &mut param);
            if ret != 0 {
                eprintln!("controleThread: Failed to set thread priority from period: {}", ret);
            };
        };
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(110);
        // Behavior Annex state machine states
        #[derive(Debug, Clone)]
        enum State {
            // State: s_inline
            s_inline,
            // State: s1
            s1,
            // State: s2
            s2,
            // State: s_outline
            s_outline,
        }
        
        let mut state: State = State::s_inline;
        loop {
            let start = Instant::now();
            let info_capteur = self.info_capteur.as_mut().and_then(|rx| { rx.try_recv().ok() }).unwrap_or_else(|| { Default::default() });
            {
                // --- BA 宏步执行 ---
                loop {
                    match state {
                        State::s_inline => {
                            // on dispatch → s1
                            state = State::s1;
                            continue;
                        },
                        State::s1 if info_capteur == true => {
                            state = State::s_inline;
                            // complete,需要停
                        },
                        State::s1 if info_capteur == false => {
                            if let Some(sender) = &self.comm_servo {
                                let _ = sender.send(false);
                            };
                            state = State::s_outline;
                            // complete,需要停
                        },
                        State::s_outline => {
                            // on dispatch → s2
                            state = State::s2;
                            continue;
                        },
                        State::s2 if info_capteur == false => {
                            state = State::s_outline;
                            // complete,需要停
                        },
                        State::s2 if info_capteur == true => {
                            if let Some(sender) = &self.comm_servo {
                                let _ = sender.send(true);
                            };
                            state = State::s_inline;
                            // complete,需要停
                        },
                        State::s2 => {
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

impl Thread for capteurThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            evenement: None, 
            dispatch_protocol: "Periodic".to_string(), 
            period: 110, 
            cpu_id: cpu_id, // CPU ID
        };
    }
    
    // Thread execution entry point
    // Period: Some(110) ms
    fn run(mut self) -> () {
        unsafe {
            let prio = period_to_priority(self.period as f64);
            let mut param: sched_param = sched_param { sched_priority: prio };
            let ret = pthread_setschedparam(pthread_self(), *CPU_ID_TO_SCHED_POLICY.get(&self.cpu_id).unwrap_or(&SCHED_FIFO), &mut param);
            if ret != 0 {
                eprintln!("capteurThread: Failed to set thread priority from period: {}", ret);
            };
        };
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(110);
        let mut count1: i32 = 0;
        // Behavior Annex state machine states
        #[derive(Debug, Clone)]
        enum State {
            // State: s0
            s0,
        }
        
        let mut state: State = State::s0;
        loop {
            let start = Instant::now();
            {
                // --- BA 宏步执行 ---
                loop {
                    match state {
                        State::s0 => {
                            // TODO: Timed action not implemented
                            count1 = count1 + 1;
                            if let Some(sender) = &self.evenement {
                                let _ = sender.send(count1 % 2 == 0);
                            };
                            // on dispatch → s0
                            state = State::s0;
                            // complete,需要停
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

impl Thread for servomoteurThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            period: 10, 
            ordre: None, 
            dispatch_protocol: "Sporadic".to_string(), 
            cpu_id: cpu_id, // CPU ID
        };
    }
    
    // Thread execution entry point
    // Period: Some(10) ms
    fn run(mut self) -> () {
        unsafe {
            let prio = period_to_priority(self.period as f64);
            let mut param: sched_param = sched_param { sched_priority: prio };
            let ret = pthread_setschedparam(pthread_self(), *CPU_ID_TO_SCHED_POLICY.get(&self.cpu_id).unwrap_or(&SCHED_FIFO), &mut param);
            if ret != 0 {
                eprintln!("servomoteurThread: Failed to set thread priority from period: {}", ret);
            };
        };
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let min_interarrival: std::time::Duration = Duration::from_millis(10);
        let mut last_dispatch: std::time::Instant = Instant::now();
        let mut events = Vec::new();
        loop {
            if events.is_empty() {
                if let Some(rx) = &self.ordre {
                    if let Ok(val) = rx.try_recv() {
                        let ts = Instant::now();
                        events.push(((val, 0, ts)));
                    };
                };
            };
            if let Some((idx, (val, _urgency, _ts))) = events.iter().enumerate().max_by(|a, b| match a.1.1.cmp(&b.1.1) {
                        std::cmp::Ordering::Equal => b.1.2.cmp(&a.1.2),
                        other => other,
                    }) {
                let (val, _, _) = events.remove(idx);
                let now = Instant::now();
                let elapsed = now.duration_since(last_dispatch);
                if elapsed < min_interarrival {
                    std::thread::sleep(min_interarrival - elapsed);
                };
                {
                    // --- 调用序列（等价 AADL 的 Wrapper）---
                           // a_spg();
                    // a_spg;
                    action_spg::receive(val);
                };
                last_dispatch = Instant::now();
            } else {
                std::thread::sleep(Duration::from_millis(1));
            };
        };
    }
    
}

// AADL Process: p_capteur
#[derive(Debug)]
pub struct p_capteurProcess {
    pub evenement: Option<Sender<bool>>,// Port: evenement Out
    pub cpu_id: isize,// 进程 CPU ID
    pub evenementRece: Option<Receiver<bool>>,// 内部端口: evenement Out
    #[allow(dead_code)]
    pub th_c: capteurThread,// 子组件线程（th_c : thread capteur）
}

// AADL Process: p_controle
#[derive(Debug)]
pub struct p_controleProcess {
    pub info_capteur_droit: Option<Receiver<bool>>,// Port: info_capteur_droit In
    pub comm_servo_droit: Option<Sender<bool>>,// Port: comm_servo_droit Out
    pub info_capteur_gauche: Option<Receiver<bool>>,// Port: info_capteur_gauche In
    pub comm_servo_gauche: Option<Sender<bool>>,// Port: comm_servo_gauche Out
    pub cpu_id: isize,// 进程 CPU ID
    pub info_capteur_droitSend: Option<BcSender<bool>>,// 内部端口: info_capteur_droit In
    pub comm_servo_droitRece: Option<Receiver<bool>>,// 内部端口: comm_servo_droit Out
    pub info_capteur_gaucheSend: Option<BcSender<bool>>,// 内部端口: info_capteur_gauche In
    pub comm_servo_gaucheRece: Option<Receiver<bool>>,// 内部端口: comm_servo_gauche Out
    #[allow(dead_code)]
    pub th_ctrl_droit: controleThread,// 子组件线程（th_ctrl_droit : thread controle）
    #[allow(dead_code)]
    pub th_ctrl_gauche: controleThread,// 子组件线程（th_ctrl_gauche : thread controle）
}

// AADL Process: p_servomoteur
#[derive(Debug)]
pub struct p_servomoteurProcess {
    pub ordre: Option<Receiver<bool>>,// Port: ordre In
    pub cpu_id: isize,// 进程 CPU ID
    pub ordreSend: Option<BcSender<bool>>,// 内部端口: ordre In
    #[allow(dead_code)]
    pub th_servomoteur: servomoteurThread,// 子组件线程（th_servomoteur : thread servomoteur）
}

impl Process for p_capteurProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut th_c: capteurThread = capteurThread::new(cpu_id);
        let mut evenementRece = None;
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            th_c.evenement = Some(channel.0);
        evenementRece = Some(channel.1);
        return Self { evenement: None, evenementRece, th_c, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn start(self: Self) -> () {
        let Self { evenement, evenementRece, th_c, cpu_id, .. } = self;
        thread::Builder::new()
            .name("th_c".to_string())
            .spawn(|| { th_c.run() }).unwrap();
        let mut evenementRece_rx = evenementRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_evenementRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = evenementRece_rx.try_recv() {
                    if let Some(tx) = &evenement {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
    }
    
}

impl Process for p_controleProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut th_ctrl_droit: controleThread = controleThread::new(cpu_id);
        let mut th_ctrl_gauche: controleThread = controleThread::new(cpu_id);
        let mut info_capteur_droitSend = None;
        let mut comm_servo_droitRece = None;
        let mut info_capteur_gaucheSend = None;
        let mut comm_servo_gaucheRece = None;
        let channel = crossbeam_channel::unbounded();
        info_capteur_droitSend = Some(channel.0);
        // build connection: 
            th_ctrl_droit.info_capteur = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            th_ctrl_droit.comm_servo = Some(channel.0);
        comm_servo_droitRece = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        info_capteur_gaucheSend = Some(channel.0);
        // build connection: 
            th_ctrl_gauche.info_capteur = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            th_ctrl_gauche.comm_servo = Some(channel.0);
        comm_servo_gaucheRece = Some(channel.1);
        return Self { info_capteur_droit: None, info_capteur_droitSend, comm_servo_droit: None, comm_servo_droitRece, info_capteur_gauche: None, info_capteur_gaucheSend, comm_servo_gauche: None, comm_servo_gaucheRece, th_ctrl_droit, th_ctrl_gauche, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn start(self: Self) -> () {
        let Self { info_capteur_droit, info_capteur_droitSend, comm_servo_droit, comm_servo_droitRece, info_capteur_gauche, info_capteur_gaucheSend, comm_servo_gauche, comm_servo_gaucheRece, th_ctrl_droit, th_ctrl_gauche, cpu_id, .. } = self;
        thread::Builder::new()
            .name("th_ctrl_droit".to_string())
            .spawn(|| { th_ctrl_droit.run() }).unwrap();
        thread::Builder::new()
            .name("th_ctrl_gauche".to_string())
            .spawn(|| { th_ctrl_gauche.run() }).unwrap();
        let mut comm_servo_droitRece_rx = comm_servo_droitRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_comm_servo_droitRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = comm_servo_droitRece_rx.try_recv() {
                    if let Some(tx) = &comm_servo_droit {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let mut comm_servo_gaucheRece_rx = comm_servo_gaucheRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_comm_servo_gaucheRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = comm_servo_gaucheRece_rx.try_recv() {
                    if let Some(tx) = &comm_servo_gauche {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let mut info_capteur_droit_rx = info_capteur_droit.unwrap();
        thread::Builder::new()
            .name("data_forwarder_info_capteur_droit".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = info_capteur_droit_rx.try_recv() {
                    if let Some(tx) = &info_capteur_droitSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let mut info_capteur_gauche_rx = info_capteur_gauche.unwrap();
        thread::Builder::new()
            .name("data_forwarder_info_capteur_gauche".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = info_capteur_gauche_rx.try_recv() {
                    if let Some(tx) = &info_capteur_gaucheSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
    }
    
}

impl Process for p_servomoteurProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut th_servomoteur: servomoteurThread = servomoteurThread::new(cpu_id);
        let mut ordreSend = None;
        let channel = crossbeam_channel::unbounded();
        ordreSend = Some(channel.0);
        // build connection: 
            th_servomoteur.ordre = Some(channel.1);
        return Self { ordre: None, ordreSend, th_servomoteur, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn start(self: Self) -> () {
        let Self { ordre, ordreSend, th_servomoteur, cpu_id, .. } = self;
        thread::Builder::new()
            .name("th_servomoteur".to_string())
            .spawn(|| { th_servomoteur.run() }).unwrap();
        let mut ordre_rx = ordre.unwrap();
        thread::Builder::new()
            .name("data_forwarder_ordre".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = ordre_rx.try_recv() {
                    if let Some(tx) = &ordreSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
    }
    
}

// AADL System: robot
#[derive(Debug)]
pub struct robotSystem {
    #[allow(dead_code)]
    pub proc_capteur_droit: p_capteurProcess,// 子组件进程（proc_capteur_droit : process p_capteur）
    #[allow(dead_code)]
    pub proc_capteur_gauche: p_capteurProcess,// 子组件进程（proc_capteur_gauche : process p_capteur）
    #[allow(dead_code)]
    pub proc_controle: p_controleProcess,// 子组件进程（proc_controle : process p_controle）
    #[allow(dead_code)]
    pub proc_servomoteur_droit: p_servomoteurProcess,// 子组件进程（proc_servomoteur_droit : process p_servomoteur）
    #[allow(dead_code)]
    pub proc_servomoteur_gauche: p_servomoteurProcess,// 子组件进程（proc_servomoteur_gauche : process p_servomoteur）
}

impl System for robotSystem {
    // Creates a new system instance
    fn new() -> Self {
        let mut proc_capteur_droit: p_capteurProcess = p_capteurProcess::new(0);
        let mut proc_capteur_gauche: p_capteurProcess = p_capteurProcess::new(0);
        let mut proc_controle: p_controleProcess = p_controleProcess::new(0);
        let mut proc_servomoteur_droit: p_servomoteurProcess = p_servomoteurProcess::new(0);
        let mut proc_servomoteur_gauche: p_servomoteurProcess = p_servomoteurProcess::new(0);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            proc_capteur_droit.evenement = Some(channel.0);
        // build connection: 
            proc_controle.info_capteur_droit = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            proc_capteur_gauche.evenement = Some(channel.0);
        // build connection: 
            proc_controle.info_capteur_gauche = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            proc_controle.comm_servo_droit = Some(channel.0);
        // build connection: 
            proc_servomoteur_droit.ordre = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            proc_controle.comm_servo_gauche = Some(channel.0);
        // build connection: 
            proc_servomoteur_gauche.ordre = Some(channel.1);
        return Self { proc_capteur_droit, proc_capteur_gauche, proc_controle, proc_servomoteur_droit, proc_servomoteur_gauche }  //显式return;
    }
    
    // Runs the system, starts all processes
    fn run(self: Self) -> () {
        self.proc_capteur_droit.start();
        self.proc_capteur_gauche.start();
        self.proc_controle.start();
        self.proc_servomoteur_droit.start();
        self.proc_servomoteur_gauche.start();
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

// prio(P)=max(1,min(99,99−⌊k⋅log10(P)⌋))
// 根据周期计算优先级，周期越短优先级越高
// 用于 RMS (Rate Monotonic Scheduling) 和 DMS (Deadline Monotonic Scheduling)
pub fn period_to_priority(period_ms: f64) -> i32 {
    let k: f64 = 10.0;
    let raw: f64 = 99.0 - k * period_ms.log10().floor();
    return raw.max(1.0).min(99.0) as i32;
}

