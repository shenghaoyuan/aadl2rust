pub mod aadlight_parser;
mod ast;
pub mod transform;
pub mod printmessage;


use aadlight_parser::AADLParser;
use pest::{Parser};
use std::fs;
use printmessage::*;

fn main() {
    let path = "pingpong.aadl";
    let aadl_input = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("读取文件失败: {}", err);
            return;
        }
    };
    
    match AADLParser::parse(aadlight_parser::Rule::file, &aadl_input) {
        Ok(pairs) => {
            println!("=== 解析成功，共 {} 个pair ===", pairs.clone().count()); // 调试输出

            // 转换到AST
            let ast = transform::AADLTransformer::transform_file(pairs.clone().collect());
            println!("=== 转换得到 {} 个package ===", ast.len()); // 调试输出

            // 打印AST
            println!("\n================================== AST ==================================");
            print_ast(ast);
            
            //打印原始解析树
            println!("\n================================== Parse Tree ==================================");
            for pair in pairs {
                print_pair(pair, 0);
            }
        }
        Err(e) => {
            eprintln!("解析失败: {}", e);
        }
    }
}
    