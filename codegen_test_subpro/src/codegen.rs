// src/codegen.rs

use handlebars::{Handlebars, Helper, HelperResult, Output, RenderContext};
use chrono::Local;
use std::collections::BTreeMap;
use serde_json::json;
use anyhow::Result;
use crate::models::System;

// 自定义 helper：帕斯卡命名法
fn pascal_case_helper(
    h: &Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap();
    let s = param.value().as_str().unwrap();
    let result = s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<String>();
    
    out.write(&result)?;
    Ok(())
}

// 自定义 helper：驼峰命名法
fn camel_case_helper(
    h: &Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap();
    let s = param.value().as_str().unwrap();
    let mut result = String::new();
    let mut first = true;
    
    for part in s.split('_') {
        if first {
            result.push_str(part);
            first = false;
        } else {
            let mut chars = part.chars();
            if let Some(c) = chars.next() {
                result.push(c.to_ascii_uppercase());
                result.extend(chars);
            }
        }
    }
    
    out.write(&result)?;
    Ok(())
}

// 自定义 helper：蛇形命名法
fn snake_case_helper(
    h: &Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap();
    let s = param.value().as_str().unwrap();
    out.write(&s.to_lowercase())?;
    Ok(())
}

// 自定义helper：查找子程序
fn lookup_subprogram_helper(
    h: &Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let role = h.param(0).unwrap().value().as_str().unwrap();
    let _connections = h.param(1).unwrap().value();
    
    // 简化实现：实际应根据连接关系查找
    let subprogram = if role == "sender" {
        "sender_spg"
    } else {
        "receiver_spg"
    };
    
    out.write(subprogram)?;
    Ok(())
}

// 注册所有需要的 helpers
pub fn register_helpers(handlebars: &mut Handlebars) {
    handlebars.register_helper("pascalCase", Box::new(pascal_case_helper));
    handlebars.register_helper("camelCase", Box::new(camel_case_helper));
    handlebars.register_helper("snakeCase", Box::new(snake_case_helper));
    handlebars.register_helper("lookup_subprogram_by_connection", Box::new(lookup_subprogram_helper));
}

// 主生成函数
pub fn generate_rust_code(system: &System) -> Result<String> {
    let mut handlebars = Handlebars::new();
    
    // 先注册自定义 helpers
    register_helpers(&mut handlebars);
    
    // 准备模板数据
    let mut data = BTreeMap::new();
    data.insert("now", json!(Local::now().to_rfc3339()));
    data.insert("threads", json!(&system.threads));
    data.insert("connections", json!(&system.connections));
    data.insert("subprograms", json!(&system.subprograms));
    
    // 注册模板文件
    handlebars.register_template_file("system", "templates/system.hbs")?;
    
    // 调试：打印数据
    println!("Template data: {:#?}", data);
    // 渲染模板
    let output = handlebars.render("system", &data)?;
    
    Ok(output)
}