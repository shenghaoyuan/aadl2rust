pub mod aadlight_parser;

use aadlight_parser::AADLParser;
use pest::{iterators::Pair, Parser};

// fn print_pair(pair: Pair<aadlight_parser::Rule>, indent: usize) {
//     println!(
//         "{}{:?}: {:?}",
//         "  ".repeat(indent),
//         pair.as_rule(),
//         pair.as_str()
//     );
//     for inner in pair.into_inner() {
//         print_pair(inner, indent + 1);
//     }
// }

fn print_pair(pair: Pair<aadlight_parser::Rule>, indent: usize) {
    // 跳过空白和注释节点
    match pair.as_rule() {
        aadlight_parser::Rule::WHITESPACE | aadlight_parser::Rule::COMMENT => return,
        _ => {
            // 获取位置信息
            let span = pair.as_span();
            let (start_line, _) = span.start_pos().line_col();
            let (end_line, _) = span.end_pos().line_col();
            
            // 格式化输出
            let content = pair.as_str().trim();
            let truncated_content = if content.len() > 30 {
                format!("{}...", &content[..30])
            } else {
                content.to_string()
            };
            
            println!(
                "{}{:<25} {:<30} (lines {}-{})",
                "  ".repeat(indent),
                format!("{:?}:", pair.as_rule()),
                truncated_content,
                start_line,
                end_line
            );

            // 递归处理子节点
            for inner in pair.into_inner() {
                print_pair(inner, indent + 1);
            }
        }
    }
}
fn main() {
    let aadl_input = r#"
        package ojr_pingpong::queued
public
with Data_Model;
pingpong renames package ojrpingpong::queued;
  system root
  end root;

  system implementation root.impl
  subcomponents
    the_cpu: processor cpu.impl;
    the_proc: process proc.impl;
    the_mem: memory mem.impl;
  end root.impl;

  processor cpu
  end cpu;

  processor implementation cpu.impl
  end cpu.impl;



  process proc
  end proc;

  process implementation proc.impl
  subcomponents
    the_sender: thread sender.impl;
    the_receiver: thread receiver.impl;
  connections
    cnx: port the_sender.p -> the_receiver.p;
  end proc.impl;

  memory mem
  end mem;

  memory implementation mem.impl
  properties
    Memory_Size => 200 KByte;
  end mem.impl;

  thread sender
  features
    p: out event data port Integer;
  properties
    Dispatch_Protocol => Periodic;
    Compute_Execution_Time => 0 ms .. 1 ms;
    Period => 2000 Ms;
    Priority => 5;
    Data_Size => 40000 bytes;
    Stack_Size => 40000 bytes;
    Code_Size => 40 bytes;
  end sender;

  thread implementation sender.impl
  calls
    call : { c : subprogram sender_spg;};
  connections
    cnx: parameter c.result -> p;
  properties
    Compute_Entrypoint_Call_Sequence => reference (call);
  end sender.impl;

  subprogram sender_spg
  features
    result : out parameter Integer;
  properties 
  	source_text => ("");
    source_name => "PingPong.Send"; 
    source_language => (java);
  end sender_spg;

  thread receiver
  features
    p: in event data port Integer;
  properties
    Dispatch_Protocol => Periodic;
    Compute_Execution_Time => 0 ms .. 1 ms;
    Period => 1000 Ms;
    Priority => 10;
    Data_Size => 40000 bytes;
    Stack_Size => 40000 bytes;
    Code_Size => 40 bytes;
  end receiver;

  thread implementation receiver.impl
  calls
    call : { c : subprogram receiver_spg;};
  connections
    cnx: parameter p -> c.input;
  properties
    Compute_Entrypoint_Call_Sequence => reference (call);
  end receiver.impl;

  subprogram receiver_spg
  features
    input : in parameter Integer;
  properties 
  	source_text => ("");
    source_name => "PingPong.Receive"; 
    source_language => (java); 
  end receiver_spg;

  data Integer
  properties
    Source_Name => "PingPongType"; 
    Data_Model::Initial_Value => ("new PingPongType()");
  end Integer;
	
end ojr_pingpong_queued;
    "#;

    match AADLParser::parse(aadlight_parser::Rule::file, aadl_input) {
        Ok(pairs) => {
            for pair in pairs {
                print_pair(pair, 0);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}