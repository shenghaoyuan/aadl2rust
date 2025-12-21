// Auto-generated from AADL package: aadlbook_software_entertainment
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
// ---------------- cpu ----------------
fn set_thread_affinity(cpu: isize) {
    unsafe {
        let mut cpuset: cpu_set_t = std::mem::zeroed();
        CPU_ZERO(&mut cpuset);
        CPU_SET(cpu as usize, &mut cpuset);
        sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset);
    }
}

// AADL Process: entertainment
#[derive(Debug)]
pub struct entertainmentProcess {
    pub music_in: Option<Receiver<bool>>,// Port: music_in In
    pub contacts: Option<Receiver<i8>>,// Port: contacts In
    pub infos: Option<Sender<i8>>,// Port: infos Out
    pub music_out: Option<Sender<bool>>,// Port: music_out Out
    pub cpu_id: isize,// 进程 CPU ID
    pub music_inSend: Option<BcSender<bool>>,// 内部端口: music_in In
    pub contactsSend: Option<BcSender<i8>>,// 内部端口: contacts In
    pub infosRece: Option<Receiver<i8>>,// 内部端口: infos Out
    pub music_outRece: Option<Receiver<bool>>,// 内部端口: music_out Out
    pub enter_thr: entertainment_thrThread,// 子组件线程（enter_thr : thread entertainment_thr）
}

impl Process for entertainmentProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let enter_thr: entertainment_thrThread = entertainment_thrThread::new(cpu_id);
        let mut music_inSend = None;
        let mut contactsSend = None;
        let mut infosRece = None;
        let mut music_outRece = None;
        let c0 = crossbeam_channel::unbounded();
        music_inSend = Some(c0.0);
        // build connection: 
            enter_thr.music_in = Some(c0.1);
        let c1 = crossbeam_channel::unbounded();
        contactsSend = Some(c1.0);
        // build connection: 
            enter_thr.contacts = Some(c1.1);
        let c2 = crossbeam_channel::unbounded();
        // build connection: 
            enter_thr.infos = Some(c2.0);
        infosRece = Some(c2.1);
        let c3 = crossbeam_channel::unbounded();
        // build connection: 
            enter_thr.music_out = Some(c3.0);
        music_outRece = Some(c3.1);
        return Self { music_in: None, music_inSend, contacts: None, contactsSend, infos: None, infosRece, music_out: None, music_outRece, enter_thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn run(self: Self) -> () {
        let Self { music_in, music_inSend, contacts, contactsSend, infos, infosRece, music_out, music_outRece, enter_thr, cpu_id, .. } = self;
        thread::Builder::new()
            .name("enter_thr".to_string())
            .spawn(move || { enter_thr.run() }).unwrap();
        let mut contacts_rx = contacts.unwrap();
        thread::Builder::new()
            .name("data_forwarder_contacts".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = contacts_rx.try_recv() {
                    if let Some(tx) = &contactsSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let mut infosRece_rx = infosRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_infosRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = infosRece_rx.try_recv() {
                    if let Some(tx) = &infos {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let mut music_in_rx = music_in.unwrap();
        thread::Builder::new()
            .name("data_forwarder_music_in".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = music_in_rx.try_recv() {
                    if let Some(tx) = &music_inSend {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
        let mut music_outRece_rx = music_outRece.unwrap();
        thread::Builder::new()
            .name("data_forwarder_music_outRece".to_string())
            .spawn(move || {
            loop {
                if let Ok(msg) = music_outRece_rx.try_recv() {
                    if let Some(tx) = &music_out {
                        let _ = tx.send(msg);
                    };
                };
                std::thread::sleep(std::time::Duration::from_millis(1));
            };
        }).unwrap();
    }
    
}

// AADL Thread: entertainment_thr
#[derive(Debug)]
pub struct entertainment_thrThread {
    pub music_in: Option<Receiver<bool>>,// Port: music_in In
    pub contacts: Option<Receiver<i8>>,// Port: contacts In
    pub infos: Option<Sender<i8>>,// Port: infos Out
    pub music_out: Option<Sender<bool>>,// Port: music_out Out
    pub dispatch_protocol: String,// AADL属性: Dispatch_Protocol
    pub period: u64,// AADL属性: Period
    pub mipsbudget: f64,// AADL属性: mipsbudget
    pub cpu_id: isize,// 结构体新增 CPU ID
}

impl Thread for entertainment_thrThread {
    // 创建组件并初始化AADL属性
    fn new(cpu_id: isize) -> Self {
        return Self {
            contacts: None, 
            music_out: None, 
            music_in: None, 
            dispatch_protocol: "Periodic".to_string(), 
            mipsbudget: 5.0, 
            infos: None, 
            period: 5, 
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
        let mut infos_value: Base_Types::Integer_8 = 0;
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
                            if let Some(sender) = &self.infos {
                                let _ = sender.send(16);
                            };
                            if let Some(sender) = &self.music_out {
                                let _ = sender.send(false);
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

