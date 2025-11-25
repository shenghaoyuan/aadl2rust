// ---------------- System ----------------
pub trait System {
    fn new() -> Self
        where Self: Sized;
    fn run(self);
}

// ---------------- Process ----------------
pub trait Process {
    fn new(cpu_id: isize) -> Self
        where Self: Sized;
    fn start(self);
}

// ---------------- Thread ----------------
pub trait Thread {
    fn new(cpu_id: isize) -> Self
        where Self: Sized;
    fn run(self);
}

pub trait Device {
    fn new() -> Self
        where Self: Sized;
    fn run(self);
}