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
            // 格式化输出：缩进 + 规则类型 + 精简后的文本内容
            let content = pair.as_str().trim();
            let truncated_content = if content.len() > 30 {
                format!("{}...", &content[..30])
            } else {
                content.to_string()
            };
            
            println!(
                "{}{:<25} {:<30} ",
                "  ".repeat(indent),
                format!("{:?}:", pair.as_rule()),
                truncated_content,
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
        system MySystem
        end MySystem;

        system implementation MySystemimpl
        subcomponents
            thecpu: processor cpuimpl;
            theproc: process procimpl;
        end MySystemimpl;
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