#![allow(unused_imports)]
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
include!(concat!(env!("OUT_DIR"), "/c_bindings.rs"));

// === C函数绑定 ===
extern "C" {
    pub fn user_do_ping_spg(val: *mut i32);
    pub fn user_ping_spg(val: i32);
    pub fn recover();
}

// === 安全封装C函数 ===
pub mod sender_spg {
    pub fn send(val: &mut i32) {
        unsafe {
            super::user_do_ping_spg(val);
        }
    }
}

pub mod receiver_spg {
    pub fn receive(val: i32) {
        if val != 0 {
            unsafe {
                super::user_ping_spg(val);
            }
        }
    }
}

pub fn recover_wrapper() {
    unsafe {
        recover();
    }
}

// === 线程定义 ===
pub struct TheSenderThread {
    id: u32,
    sender: Option<mpsc::Sender<i32>>,
    pub stack_size: i64,
    pub period: u64,
}

impl TheSenderThread {
    pub fn new() -> Self {
        Self {
            id: 0,
            sender: None,
            stack_size: 40000,
            period: 2000,
        }
    }

    pub fn run(&mut self) {
        let period = Duration::from_millis(self.period);
        loop {
            let start = Instant::now();
            if let Some(sender) = &self.sender {
                let mut val = 0;
                sender_spg::send(&mut val);
                sender.send(val).unwrap();
            }
            let elapsed = start.elapsed();
            if elapsed < period {
                thread::sleep(period - elapsed);
            }
        }
    }
}

pub struct TheReceiverThread {
    id: u32,
    receiver: Option<mpsc::Receiver<i32>>,
    pub stack_size: i64,
    pub period: u64,
}

impl TheReceiverThread {
    pub fn new() -> Self {
        Self {
            id: 1,
            receiver: None,
            stack_size: 40000,
            period: 1000,
        }
    }

    pub fn run(&mut self) {
        let period = Duration::from_millis(self.period);
        loop {
            let start = Instant::now();
            if let Some(receiver) = &self.receiver {
                let val = receiver.recv().unwrap();
                receiver_spg::receive(val);
            }
            let elapsed = start.elapsed();
            if elapsed < period {
                thread::sleep(period - elapsed);
            }
        }
    }
}

// === 进程结构体 ===
pub struct PingPongProcess {
    the_sender: TheSenderThread,
    the_receiver: TheReceiverThread,
}

impl PingPongProcess {
    pub fn new() -> Self {
        let mut the_sender = TheSenderThread::new();
        let mut the_receiver = TheReceiverThread::new();

        let (tx, rx) = mpsc::channel();
        the_sender.sender = Some(tx);
        the_receiver.receiver = Some(rx);

        Self {
            the_sender,
            the_receiver,
        }
    }

    pub fn start(mut self) {
        let mut sender = self.the_sender;
        let mut receiver = self.the_receiver;

        thread::Builder::new()
            .name("the_sender".to_string())
            .stack_size(sender.stack_size as usize)
            .spawn(move || {
                sender.run();
            })
            .unwrap();

        thread::Builder::new()
            .name("the_receiver".to_string())
            .stack_size(receiver.stack_size as usize)
            .spawn(move || {
                receiver.run();
            })
            .unwrap();
    }
}
