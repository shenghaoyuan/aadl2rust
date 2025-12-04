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

// AADL Process: entertainment
#[derive(Debug)]
pub struct entertainmentProcess {
    pub music_in: Option<Receiver<bool>>,// Port: music_in In
    pub contacts: Option<Receiver<i8>>,// Port: contacts In
    pub infos: Option<Sender<i8>>,// Port: infos Out
    pub music_out: Option<Sender<bool>>,// Port: music_out Out
    pub cpu_id: isize,// 进程 CPU ID
    pub music_inSend: Option<Sender<bool>>,// 内部端口: music_in In
    pub contactsSend: Option<Sender<i8>>,// 内部端口: contacts In
    pub infosRece: Option<Receiver<i8>>,// 内部端口: infos Out
    pub music_outRece: Option<Receiver<bool>>,// 内部端口: music_out Out
    #[allow(dead_code)]
    pub enter_thr: entertainment_thrThread,// 子组件线程（enter_thr : thread entertainment_thr）
}

impl Process for entertainmentProcess {
    // Creates a new process instance
    fn new(cpu_id: isize) -> Self {
        let mut enter_thr: entertainment_thrThread = entertainment_thrThread::new(cpu_id);
        let mut music_inSend = None;
        let mut contactsSend = None;
        let mut infosRece = None;
        let mut music_outRece = None;
        music_inSend = Some(channel.0);
        // build connection: 
            enter_thr.music_in = Some(channel.1);
        contactsSend = Some(channel.0);
        // build connection: 
            enter_thr.contacts = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            enter_thr.infos = Some(channel.0);
        infosRece = Some(channel.1);
        let channel = crossbeam_channel::unbounded();
        // build connection: 
            enter_thr.music_out = Some(channel.0);
        music_outRece = Some(channel.1);
        return Self { music_in: None, music_inSend, contacts: None, contactsSend, infos: None, infosRece, music_out: None, music_outRece, enter_thr, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    fn start(self: Self) -> () {
        let Self { music_in, music_inSend, contacts, contactsSend, infos, infosRece, music_out, music_outRece, enter_thr, cpu_id, .. } = self;
        thread::Builder::new()
            .name("enter_thr".to_string())
            .spawn(|| { enter_thr.run() }).unwrap();
        let music_in_rx = music_in.unwrap();
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
        let contacts_rx = contacts.unwrap();
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
        let infosRece_rx = infosRece.unwrap();
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
        let music_outRece_rx = music_outRece.unwrap();
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
            music_out: None, 
            period: 5, 
            music_in: None, 
            infos: None, 
            dispatch_protocol: "Periodic".to_string(), 
            mipsbudget: 5.0, 
            contacts: None, 
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
        return map;
    };
}

