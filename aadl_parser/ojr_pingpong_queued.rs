//! 自动生成的Rust代码 - 来自AADL包: ojr_pingpong::queued

use Data_Model::*;

/// root 组件类型 - 自动生成自AADL Unknown
pub struct root {
    // 基础组件字段
}

impl root {
    pub fn new() -> Self {
        Self {}
    }
}

/// root.impl 实现 - 自动生成自AADL root.impl
pub struct root.impl {
    /// Processor子组件: the_cpu
    pub the_cpu: cpu.impl,
    /// Process子组件: the_proc
    pub the_proc: proc.impl,
    /// Memory子组件: the_mem
    pub the_mem: mem.impl,
}

impl root.impl {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            the_cpu: cpu.impl::new(),
            the_proc: proc.impl::new(),
            the_mem: mem.impl::new(),
        }
    }

}

/// cpu 组件类型 - 自动生成自AADL Processor
pub struct cpu {
    // 基础组件字段
}

impl cpu {
    pub fn new() -> Self {
        Self {}
    }
}

/// cpu.impl 实现 - 自动生成自AADL cpu.impl
pub struct cpu.impl {
}

impl cpu.impl {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
        }
    }

}

/// proc 组件类型 - 自动生成自AADL Process
pub struct proc {
    pub threads: Vec<Thread>,
    pub address_space: MemoryRegion,
}

impl proc {
    pub fn new() -> Self {
        Self {
            threads: Vec::new(),
            address_space: MemoryRegion::new(),
        }
    }

    /// 启动所有线程
    pub fn start_all_threads(&mut self) {
        for thread in &mut self.threads {
            thread.run();
        }
    }
}

/// proc.impl 实现 - 自动生成自AADL proc.impl
pub struct proc.impl {
    /// Thread子组件: the_sender
    pub the_sender: sender.impl,
    /// Thread子组件: the_receiver
    pub the_receiver: receiver.impl,
}

impl proc.impl {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            the_sender: sender.impl::new(),
            the_receiver: receiver.impl::new(),
        }
    }

    /// 设置组件间连接
    pub fn setup_connections(&mut self) {
        self.the_sender.connect(&mut self.the_receiver);  // 直接连接
    }
}

/// mem 组件类型 - 自动生成自AADL Memory
pub struct mem {
    // 基础组件字段
}

impl mem {
    pub fn new() -> Self {
        Self {}
    }
}

/// mem.impl 实现 - 自动生成自AADL mem.impl
pub struct mem.impl {
}

impl mem.impl {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
        }
    }

}

/// sender 组件类型 - 自动生成自AADL Thread
pub struct sender {
    /// Out端口: "" "p"
    pub p: Port<>,
    pub is_running: bool,
}

impl sender {
    /// 创建新线程实例
    pub fn new() -> Self {
        Self {
            p: Port::new(PortDirection::Out),
            is_running: false,
        }
    }

    /// 周期性线程运行方法，周期: 2000Ms
    pub fn run(&mut self) {
        self.is_running = true;
        // 周期性执行逻辑
    }
}

/// sender.impl 实现 - 自动生成自AADL sender.impl
pub struct sender.impl {
}

impl sender.impl {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
        }
    }

    /// 设置组件间连接
    pub fn setup_connections(&mut self) {
        self..bind(&mut self.);  // 参数绑定
    }

    /// 执行调用序列
    pub fn execute_call_sequence(&self) {
    }
}

/// sender_spg 组件类型 - 自动生成自AADL Subprogram
pub struct sender_spg {
    // 基础组件字段
}

impl sender_spg {
    pub fn new() -> Self {
        Self {}
    }
}

/// receiver 组件类型 - 自动生成自AADL Thread
pub struct receiver {
    /// In端口: "" "p"
    pub p: Port<>,
    pub is_running: bool,
}

impl receiver {
    /// 创建新线程实例
    pub fn new() -> Self {
        Self {
            p: Port::new(PortDirection::In),
            is_running: false,
        }
    }

    /// 周期性线程运行方法，周期: 1000Ms
    pub fn run(&mut self) {
        self.is_running = true;
        // 周期性执行逻辑
    }
}

/// receiver.impl 实现 - 自动生成自AADL receiver.impl
pub struct receiver.impl {
}

impl receiver.impl {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
        }
    }

    /// 设置组件间连接
    pub fn setup_connections(&mut self) {
        self..bind(&mut self.);  // 参数绑定
    }

    /// 执行调用序列
    pub fn execute_call_sequence(&self) {
    }
}

/// receiver_spg 组件类型 - 自动生成自AADL Subprogram
pub struct receiver_spg {
    // 基础组件字段
}

impl receiver_spg {
    pub fn new() -> Self {
        Self {}
    }
}

/// Integer 组件类型 - 自动生成自AADL Data
pub struct Integer {
    // 基础组件字段
}

impl Integer {
    pub fn new() -> Self {
        Self {}
    }
}

