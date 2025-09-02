// 自动生成的 Rust 代码 - 来自 AADL 模型
// 生成时间: 2025-09-02 20:25:34

#![allow(unused_imports)]
use std::sync::{mpsc, Arc};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
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
    // source_files: "robot.c"
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
    // Port: evenement Out
    pub evenement: Option<mpsc::Sender<bool>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
}

impl capteurThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            evenement: None,
            cpu_id: cpu_id,
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 110, // AADL属性: Period
        }
    }
}
// AADL Thread: controle
#[derive(Debug)]
pub struct controleThread {
    // Port: info_capteur In
    pub info_capteur: Option<mpsc::Receiver<bool>>,
    // Port: comm_servo Out
    pub comm_servo: Option<mpsc::Sender<bool>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
}

impl controleThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            info_capteur: None,
            comm_servo: None,
            cpu_id: cpu_id,
            dispatch_protocol: "Periodic".to_string(), // AADL属性: Dispatch_Protocol
            period: 110, // AADL属性: Period
        }
    }
}
// AADL Thread: servomoteur
#[derive(Debug)]
pub struct servomoteurThread {
    // Port: ordre In
    pub ordre: Option<mpsc::Receiver<bool>>,
    // 结构体新增 CPU ID
    pub cpu_id: isize,
    
    // --- AADL属性 ---
    pub dispatch_protocol: String, // AADL属性: Dispatch_Protocol
    pub period: u64, // AADL属性: Period
}

impl servomoteurThread {
    // 创建组件并初始化AADL属性
    pub fn new(cpu_id: isize) -> Self {
        Self {
            ordre: None,
            cpu_id: cpu_id,
            dispatch_protocol: "Sporadic".to_string(), // AADL属性: Dispatch_Protocol
            period: 10, // AADL属性: Period
        }
    }
}
impl controleThread {
    // Thread execution entry point
    // Period: Some(110) ms
    pub fn run(mut self) -> () {
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(110);
        loop {
            let start = Instant::now();
            {
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

impl capteurThread {
    // Thread execution entry point
    // Period: Some(110) ms
    pub fn run(mut self) -> () {
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let period: std::time::Duration = Duration::from_millis(110);
        loop {
            let start = Instant::now();
            {
            };
            let elapsed = start.elapsed();
            std::thread::sleep(period.saturating_sub(elapsed));
        };
    }
    
}

impl servomoteurThread {
    // Thread execution entry point
    // Period: Some(10) ms
    pub fn run(mut self) -> () {
        if self.cpu_id > -1 {
            set_thread_affinity(self.cpu_id);
        };
        let min_interarrival: std::time::Duration = Duration::from_millis(10);
        let mut last_dispatch: std::time::Instant = Instant::now();
        loop {
            if let Some(receiver) = &self.ordre {
                match receiver.recv() {
                    Ok(val) => {
                        // 收到消息 → 调用处理函数
                        let now = Instant::now();
                        let elapsed = now.duration_since(last_dispatch);
                        if elapsed < min_interarrival {
                            std::thread::sleep(min_interarrival - elapsed);
                        };
                        {
                            // --- 调用序列（等价 AADL 的 Wrapper）---
            // A_Spg();
                            // A_Spg;
                            action_spg::receive(val);
                        };
                        let mut last_dispatch = Instant::now();
                    },
                    Err(_) => {
                        eprintln!("servomoteurThread: channel closed");
                        return;
                    },
                };
            };
        };
    }
    
}

// AADL Process: p_capteur
#[derive(Debug)]
pub struct p_capteurProcess {
    // Port: evenement Out
    pub evenement: Option<mpsc::Sender<bool>>,
    // 进程 CPU ID
    pub cpu_id: isize,
    // 内部端口: evenement Out
    pub evenementRece: Option<mpsc::Receiver<bool>>,
    // 子组件线程（th_c : thread capteur）
    #[allow(dead_code)]
    pub th_c: capteurThread,
}

// AADL Process: p_controle
#[derive(Debug)]
pub struct p_controleProcess {
    // Port: info_capteur_droit In
    pub info_capteur_droit: Option<mpsc::Receiver<bool>>,
    // Port: comm_servo_droit Out
    pub comm_servo_droit: Option<mpsc::Sender<bool>>,
    // Port: info_capteur_gauche In
    pub info_capteur_gauche: Option<mpsc::Receiver<bool>>,
    // Port: comm_servo_gauche Out
    pub comm_servo_gauche: Option<mpsc::Sender<bool>>,
    // 进程 CPU ID
    pub cpu_id: isize,
    // 内部端口: info_capteur_droit In
    pub info_capteur_droitSend: Option<mpsc::Sender<bool>>,
    // 内部端口: comm_servo_droit Out
    pub comm_servo_droitRece: Option<mpsc::Receiver<bool>>,
    // 内部端口: info_capteur_gauche In
    pub info_capteur_gaucheSend: Option<mpsc::Sender<bool>>,
    // 内部端口: comm_servo_gauche Out
    pub comm_servo_gaucheRece: Option<mpsc::Receiver<bool>>,
    // 子组件线程（th_ctrl_droit : thread controle）
    #[allow(dead_code)]
    pub th_ctrl_droit: controleThread,
    // 子组件线程（th_ctrl_gauche : thread controle）
    #[allow(dead_code)]
    pub th_ctrl_gauche: controleThread,
}

// AADL Process: p_servomoteur
#[derive(Debug)]
pub struct p_servomoteurProcess {
    // Port: ordre In
    pub ordre: Option<mpsc::Receiver<bool>>,
    // 进程 CPU ID
    pub cpu_id: isize,
    // 内部端口: ordre In
    pub ordreSend: Option<mpsc::Sender<bool>>,
    // 子组件线程（th_servomoteur : thread servomoteur）
    #[allow(dead_code)]
    pub th_servomoteur: servomoteurThread,
}

impl p_capteurProcess {
    // Creates a new process instance
    pub fn new(cpu_id: isize) -> Self {
        let mut th_c: capteurThread = capteurThread::new(cpu_id);
        let mut evenementRece = None;
        let channel = mpsc::channel();
        // build connection: 
            th_c.evenement = Some(channel.0);
        evenementRece = Some(channel.1);
        return Self { evenement: None, evenementRece, th_c, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    pub fn start(self: Self) -> () {
        let Self { evenement, evenementRece, th_c, cpu_id, .. } = self;
        thread::Builder::new()
            .name("th_c".to_string())
            .spawn(|| { th_c.run() }).unwrap();
        let evenementRece_rx = evenementRece.unwrap();
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

impl p_controleProcess {
    // Creates a new process instance
    pub fn new(cpu_id: isize) -> Self {
        let mut th_ctrl_droit: controleThread = controleThread::new(cpu_id);
        let mut th_ctrl_gauche: controleThread = controleThread::new(cpu_id);
        let mut info_capteur_droitSend = None;
        let mut comm_servo_droitRece = None;
        let mut info_capteur_gaucheSend = None;
        let mut comm_servo_gaucheRece = None;
        let channel = mpsc::channel();
        info_capteur_droitSend = Some(channel.0);
        // build connection: 
            th_ctrl_droit.info_capteur = Some(channel.1);
        let channel = mpsc::channel();
        // build connection: 
            th_ctrl_droit.comm_servo = Some(channel.0);
        comm_servo_droitRece = Some(channel.1);
        let channel = mpsc::channel();
        info_capteur_gaucheSend = Some(channel.0);
        // build connection: 
            th_ctrl_gauche.info_capteur = Some(channel.1);
        let channel = mpsc::channel();
        // build connection: 
            th_ctrl_gauche.comm_servo = Some(channel.0);
        comm_servo_gaucheRece = Some(channel.1);
        return Self { info_capteur_droit: None, info_capteur_droitSend, comm_servo_droit: None, comm_servo_droitRece, info_capteur_gauche: None, info_capteur_gaucheSend, comm_servo_gauche: None, comm_servo_gaucheRece, th_ctrl_droit, th_ctrl_gauche, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    pub fn start(self: Self) -> () {
        let Self { info_capteur_droit, info_capteur_droitSend, comm_servo_droit, comm_servo_droitRece, info_capteur_gauche, info_capteur_gaucheSend, comm_servo_gauche, comm_servo_gaucheRece, th_ctrl_droit, th_ctrl_gauche, cpu_id, .. } = self;
        thread::Builder::new()
            .name("th_ctrl_droit".to_string())
            .spawn(|| { th_ctrl_droit.run() }).unwrap();
        thread::Builder::new()
            .name("th_ctrl_gauche".to_string())
            .spawn(|| { th_ctrl_gauche.run() }).unwrap();
        let info_capteur_droit_rx = info_capteur_droit.unwrap();
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
        let comm_servo_droitRece_rx = comm_servo_droitRece.unwrap();
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
        let info_capteur_gauche_rx = info_capteur_gauche.unwrap();
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
        let comm_servo_gaucheRece_rx = comm_servo_gaucheRece.unwrap();
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
    }
    
}

impl p_servomoteurProcess {
    // Creates a new process instance
    pub fn new(cpu_id: isize) -> Self {
        let mut th_servomoteur: servomoteurThread = servomoteurThread::new(cpu_id);
        let mut ordreSend = None;
        let channel = mpsc::channel();
        ordreSend = Some(channel.0);
        // build connection: 
            th_servomoteur.ordre = Some(channel.1);
        return Self { ordre: None, ordreSend, th_servomoteur, cpu_id }  //显式return;
    }
    
    // Starts all threads in the process
    pub fn start(self: Self) -> () {
        let Self { ordre, ordreSend, th_servomoteur, cpu_id, .. } = self;
        thread::Builder::new()
            .name("th_servomoteur".to_string())
            .spawn(|| { th_servomoteur.run() }).unwrap();
        let ordre_rx = ordre.unwrap();
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
    // 子组件进程（proc_capteur_droit : process p_capteur）
    #[allow(dead_code)]
    pub proc_capteur_droit: p_capteurProcess,
    // 子组件进程（proc_capteur_gauche : process p_capteur）
    #[allow(dead_code)]
    pub proc_capteur_gauche: p_capteurProcess,
    // 子组件进程（proc_controle : process p_controle）
    #[allow(dead_code)]
    pub proc_controle: p_controleProcess,
    // 子组件进程（proc_servomoteur_droit : process p_servomoteur）
    #[allow(dead_code)]
    pub proc_servomoteur_droit: p_servomoteurProcess,
    // 子组件进程（proc_servomoteur_gauche : process p_servomoteur）
    #[allow(dead_code)]
    pub proc_servomoteur_gauche: p_servomoteurProcess,
}

impl robotSystem {
    // Creates a new system instance
    pub fn new() -> Self {
        let mut proc_capteur_droit: p_capteurProcess = p_capteurProcess::new(0);
        let mut proc_capteur_gauche: p_capteurProcess = p_capteurProcess::new(0);
        let mut proc_controle: p_controleProcess = p_controleProcess::new(0);
        let mut proc_servomoteur_droit: p_servomoteurProcess = p_servomoteurProcess::new(0);
        let mut proc_servomoteur_gauche: p_servomoteurProcess = p_servomoteurProcess::new(0);
        let channel = mpsc::channel();
        // build connection: 
            proc_capteur_droit.evenement = Some(channel.0);
        // build connection: 
            proc_controle.info_capteur_droit = Some(channel.1);
        let channel = mpsc::channel();
        // build connection: 
            proc_capteur_gauche.evenement = Some(channel.0);
        // build connection: 
            proc_controle.info_capteur_gauche = Some(channel.1);
        let channel = mpsc::channel();
        // build connection: 
            proc_controle.comm_servo_droit = Some(channel.0);
        // build connection: 
            proc_servomoteur_droit.ordre = Some(channel.1);
        let channel = mpsc::channel();
        // build connection: 
            proc_controle.comm_servo_gauche = Some(channel.0);
        // build connection: 
            proc_servomoteur_gauche.ordre = Some(channel.1);
        return Self { proc_capteur_droit, proc_capteur_gauche, proc_controle, proc_servomoteur_droit, proc_servomoteur_gauche }  //显式return;
    }
    
    // Runs the system, starts all processes
    pub fn run(self: Self) -> () {
        self.proc_capteur_droit.start();
        self.proc_capteur_gauche.start();
        self.proc_controle.start();
        self.proc_servomoteur_droit.start();
        self.proc_servomoteur_gauche.start();
    }
    
}

