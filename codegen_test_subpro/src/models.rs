use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Thread {
    pub name: String,
    pub id: u32,
    pub period: u32,
    pub priority: u32,
    pub compute_execution_time: u32,
    pub data_size: u32,
    pub stack_size: u32,
    pub code_size: u32,
    pub dispatch_protocol: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Connection {
    pub sender: String,
    pub receiver: String,
    pub port: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Subprogram {
    pub name: String,
    pub source_name: String,
    pub source_code: String,  // 存储原始Java代码
    pub supported_languages: String,
}

#[derive(Debug, Serialize)]
pub struct System {
    pub threads: Vec<Thread>,
    pub connections: Vec<Connection>,
    pub subprograms: Vec<Subprogram>,
}