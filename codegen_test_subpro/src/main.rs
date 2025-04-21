mod models;
mod codegen;

use crate::models::{System, Thread, Connection, Subprogram};
use anyhow::Result;

fn main() -> Result<()> {
    let pingpong_java = r#"
public class PingPong {
    public static int count = 0;
    public static void Send(int val) {
        System.out.println("[PING] " + count);
        count = count + 1;
        val = count;
    }
    public static void Receive(int val) {
        if (val != 0) {
            System.out.println("[PONG] " + val);
        }
    }
}"#;

    let system: System = System {
        threads: vec![
            Thread {
                name: "the_sender".to_string(),
                id: 0,
                compute_execution_time: 1,
                period: 2000,
                priority: 5,
                data_size: 4,  // i32大小
                stack_size: 40000,
                code_size: 40,
                dispatch_protocol: "Periodic".to_string(),
            },
            Thread {
                name: "the_receiver".to_string(),
                id: 1,
                compute_execution_time: 1,
                period: 1000,
                priority: 10,
                data_size: 4,
                stack_size: 40000,
                code_size: 40,
                dispatch_protocol: "Periodic".to_string(),
            },
        ],
        connections: vec![
            Connection {
                sender: "the_sender".to_string(),
                receiver: "the_receiver".to_string(),
                port: "p".to_string(),
            },
            // Connection {
            //     sender: "sender_spg".to_string(),
            //     receiver: "the_sender".to_string(),
            //     port: "result".to_string(),
            // },
            // Connection {
            //     sender: "the_receiver".to_string(),
            //     receiver: "receiver_spg".to_string(),
            //     port: "input".to_string(),
            // },
        ],
        subprograms: vec![
            Subprogram {
                name: "sender_spg".to_string(),
                source_name: "PingPong.Send".to_string(),
                source_code: pingpong_java.to_string(),
                supported_languages: "Java".to_string(),
            },
            Subprogram {
                name: "receiver_spg".to_string(),
                source_name: "PingPong.Receive".to_string(),
                source_code: pingpong_java.to_string(),
                supported_languages: "Java".to_string(),
            },
        ],
    };

    let rust_code: String = codegen::generate_rust_code(&system)?;
    println!("{}", rust_code);
    std::fs::write("generated_system.rs", rust_code)?;
    Ok(())
}