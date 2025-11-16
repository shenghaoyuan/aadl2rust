// 自动生成的 Rust 代码 - 来自 AADL 模型 Toy_Example（共享变量）
// 生成时间: 2025-08-16

#![allow(unused_imports)]
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use libc::{
    cpu_set_t, pthread_self, pthread_setschedparam, sched_param, sched_setaffinity, CPU_SET,
    CPU_ZERO, SCHED_FIFO,
};

include!(concat!(env!("OUT_DIR"), "/aadl_c_bindings.rs")); // 提供 user_* C 函数绑定

// ---------------- 实用工具：CPU 亲和/实时优先级 ----------------
fn set_thread_affinity(cpu: isize) {
    if cpu < 0 {
        return;
    }
    unsafe {
        let mut cpuset: cpu_set_t = std::mem::zeroed();
        CPU_ZERO(&mut cpuset);
        CPU_SET(cpu as usize, &mut cpuset);
        let _ = sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset);
    }
}

/// AADL 优先级（0..255）粗略映射到 Linux SCHED_FIFO 的 1..99 区间
fn map_priority(aadl_prio: u64) -> i32 {
    let mut p = aadl_prio as i32;
    if p < 1 {
        p = 1;
    }
    if p > 99 {
        p = 99;
    }
    p
}

fn set_rt_priority(aadl_prio: u64) {
    unsafe {
        let mut param = sched_param {
            sched_priority: map_priority(aadl_prio),
        };
        let ret = pthread_setschedparam(pthread_self(), SCHED_FIFO, &mut param);
        if ret != 0 {
            eprintln!(
                "WARN: pthread_setschedparam failed for AADL priority {} (errno={})",
                aadl_prio, ret
            );
        }
    }
}

// ---------------- AADL Data: POS ----------------
/// AADL: POS_Internal_Type = Integer
pub type PosInternalType = i32;

/// AADL: POS.Impl 的“共享变量实例”在 Rust 中用 Arc<Mutex<i32>> 表达
pub type PosShared = Arc<Mutex<PosInternalType>>;

// ---------------- AADL Subprogram Wrappers（C 绑定） ----------------
// Update(this : requires data access POS.Impl)  =>  user_update(int*)
pub mod update_spg {
    use super::{user_update, PosInternalType};
    pub fn call(pos_ref: &mut PosInternalType) {
        unsafe { user_update(pos_ref as *mut PosInternalType) };
    }
}

// Read_POS(this : requires data access POS.Impl) => user_read(int*)
pub mod read_pos_spg {
    use super::{user_read, PosInternalType};
    pub fn call(pos_ref: &mut PosInternalType) {
        unsafe { user_read(pos_ref as *mut PosInternalType) };
    }
}

// GNC_Job => user_gnc_job(void)
pub mod gnc_job_spg {
    use super::user_gnc_job;
    pub fn call() {
        unsafe { user_gnc_job() };
    }
}

// TMTC_Job => user_tmtc_job(void)
pub mod tmtc_job_spg {
    use super::user_tmtc_job;
    pub fn call() {
        unsafe { user_tmtc_job() };
    }
}

// GNC_Identity => user_gnc_identity(void)
pub mod gnc_identity_spg {
    use super::user_gnc_identity;
    pub fn call() {
        unsafe { user_gnc_identity() };
    }
}

// TMTC_Identity => user_tmtc_identity(void)
pub mod tmtc_identity_spg {
    use super::user_tmtc_identity;
    pub fn call() {
        unsafe { user_tmtc_identity() };
    }
}

// ---------------- AADL Threads ----------------
/// AADL Thread: GNC_Thread
#[derive(Debug)]
pub struct GncThread {
    // AADL feature: GNC_POS : requires data access POS.Impl
    pub pos: PosShared,
    // 绑定的 CPU
    pub cpu_id: isize,

    // --- AADL 属性（和原模型一致） ---
    pub dispatch_protocol: &'static str, // Periodic
    pub period_ms: u64,                  // 1000 ms
    pub compute_execution_time_ms: (u64, u64), // 0 .. 600 ms
    pub deadline_ms: u64,                // 1000 ms
    pub priority: u64,                   // 250
}

impl GncThread {
    pub fn new(pos: PosShared, cpu_id: isize) -> Self {
        Self {
            pos,
            cpu_id,
            dispatch_protocol: "Periodic",
            period_ms: 1000,
            compute_execution_time_ms: (0, 600),
            deadline_ms: 1000,
            priority: 250,
        }
    }

    /// 对应 AADL 的 GNC_Thread_Wrapper 调用序列：
    /// Welcome -> Update_POS(this) -> GNC_Job() -> Read_POS(this) -> Bye
    fn call_sequence(&mut self) {
        gnc_identity_spg::call(); // Welcome

        // Update_POS(this)
        {
            if let Ok(mut guard) = self.pos.lock() {
                update_spg::call(&mut *guard);
            }
        }

        // GNC_Job
        gnc_job_spg::call();

        // Read_POS(this)
        {
            if let Ok(mut guard) = self.pos.lock() {
                read_pos_spg::call(&mut *guard);
            }
        }

        gnc_identity_spg::call(); // Bye
    }

    /// 线程入口（Periodic）
    pub fn run(mut self) -> () {
        set_rt_priority(self.priority);
        set_thread_affinity(self.cpu_id);

        let period = Duration::from_millis(self.period_ms);
        loop {
            let start = Instant::now();

            // --- 调用序列（等价 AADL 的 Wrapper）---
            self.call_sequence();

            // --- 周期等待 ---
            let elapsed = start.elapsed();
            if elapsed < period {
                thread::sleep(period - elapsed);
            }
        }
    }
}

/// AADL Thread: TMTC_Thread
#[derive(Debug)]
pub struct TmtcThread {
    // AADL feature: TMTC_POS : requires data access POS.Impl
    pub pos: PosShared,
    pub cpu_id: isize,

    // --- AADL 属性（和原模型一致） ---
    pub dispatch_protocol: &'static str, // Periodic
    pub period_ms: u64,                  // 100 ms
    pub compute_execution_time_ms: (u64, u64), // 0 .. 50 ms
    pub deadline_ms: u64,                // 100 ms
    pub priority: u64,                   // 190
}

impl TmtcThread {
    pub fn new(pos: PosShared, cpu_id: isize) -> Self {
        Self {
            pos,
            cpu_id,
            dispatch_protocol: "Periodic",
            period_ms: 100,
            compute_execution_time_ms: (0, 50),
            deadline_ms: 100,
            priority: 190,
        }
    }

    /// 对应 AADL 的 TMTC_Thread_Wrapper 调用序列：
    /// Welcome -> TMTC_Job() -> Update(this) -> Bye
    fn call_sequence(&mut self) {
        tmtc_identity_spg::call(); // Welcome

        // TMTC_Job
        tmtc_job_spg::call();

        // Update(this)
        {
            if let Ok(mut guard) = self.pos.lock() {
                update_spg::call(&mut *guard);
            }
        }

        tmtc_identity_spg::call(); // Bye
    }

    /// 线程入口（Periodic）
    pub fn run(mut self) -> () {
        set_rt_priority(self.priority);
        set_thread_affinity(self.cpu_id);

        let period = Duration::from_millis(self.period_ms);
        loop {
            let start = Instant::now();

            // --- 调用序列（等价 AADL 的 Wrapper）---
            self.call_sequence();

            // --- 周期等待 ---
            let elapsed = start.elapsed();
            if elapsed < period {
                thread::sleep(period - elapsed);
            }
        }
    }
}

// ---------------- AADL Process: Toy_Example_Proc ----------------
#[derive(Debug)]
pub struct ToyExampleProc {
    // 子组件
    pub gnc_th: GncThread,
    pub tmtc_th: TmtcThread,

    // 共享数据（POS_Data : data POS.Impl）
    pub pos_data: PosShared,

    // 进程绑定 CPU（可选：分别给两个线程不同 CPU）
    pub cpu_id_for_gnc: isize,
    pub cpu_id_for_tmtc: isize,
}

impl ToyExampleProc {
    pub fn new(cpu_gnc: isize, cpu_tmtc: isize) -> Self {
        // 创建共享变量（POS.Impl -> 这里是一个 i32，初值 0）
        let pos = Arc::new(Mutex::new(0i32));

        // 线程持有共享引用
        let gnc = GncThread::new(pos.clone(), cpu_gnc);
        let tmtc = TmtcThread::new(pos.clone(), cpu_tmtc);

        Self {
            gnc_th: gnc,
            tmtc_th: tmtc,
            pos_data: pos,
            cpu_id_for_gnc: cpu_gnc,
            cpu_id_for_tmtc: cpu_tmtc,
        }
    }

    pub fn start(self) {
        let Self {
            gnc_th,
            tmtc_th,
            ..
        } = self;

        // 启动线程
        thread::Builder::new()
            .name("GNC_Thread".to_string())
            .spawn(move || {
                gnc_th.run();
            })
            .expect("spawn GNC_Thread");

        thread::Builder::new()
            .name("TMTC_Thread".to_string())
            .spawn(move || {
                tmtc_th.run();
            })
            .expect("spawn TMTC_Thread");
    }
}

// ---------------- AADL System: toy_example.native ----------------
#[derive(Debug)]
pub struct ToyExampleSystem {
    // 这里简单起见：一个进程实例，两个线程，共享同一个 POS
    // 你可以扩展为多进程并绑定不同 CPU
    pub bindings: Vec<(&'static str, isize, isize)>, // (process_name, cpu_gnc, cpu_tmtc)
}

impl ToyExampleSystem {
    pub fn new() -> Self {
        // 示例：将 GNC 绑核0，TMTC 绑核1（如果只有1个核，给同一个也没关系）
        Self {
            bindings: vec![("GNC_TMTC_POS", 0, 1)],
        }
    }

    pub fn run(self) {
        for (proc_name, cpu_gnc, cpu_tmtc) in self.bindings {
            match proc_name {
                "GNC_TMTC_POS" => {
                    let p = ToyExampleProc::new(cpu_gnc, cpu_tmtc);
                    p.start();
                }
                _ => eprintln!("Unknown process: {}", proc_name),
            }
        }

        // 系统主线程不退出
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    }
}
