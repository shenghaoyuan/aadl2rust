
use crate::aadl_ast2rust_code::intermediate_ast::*;

use crate::ast::aadl_ast_cj::*;
use std::collections::HashMap;

// 转换CPU实现
pub fn convert_processor_implementation(
    cpu_scheduling_protocols: &mut HashMap<String, String>,
    impl_: &ComponentImplementation,
) -> Vec<Item> {
    // 从CPU实现中提取Scheduling_Protocol属性并保存
    let cpu_name = impl_.name.type_identifier.clone();

    if let PropertyClause::Properties(props) = &impl_.properties {
        for prop in props {
            if let Property::BasicProperty(bp) = prop {
                if bp.identifier.name.to_lowercase() == "scheduling_protocol" {
                    if let PropertyValue::Single(PropertyExpression::String(StringTerm::Literal(
                        scheduling_protocol,
                    ))) = &bp.value
                    {
                        cpu_scheduling_protocols
                            .insert(cpu_name.clone(), scheduling_protocol.clone());
                        return Vec::new(); // CPU实现不生成代码，只保存信息
                    }}}}}

    // 如果没有找到Scheduling_Protocol属性，使用默认值
    cpu_scheduling_protocols.insert(cpu_name.clone(), "FIFO".to_string());
    println!("CPU实现 {} 未指定调度协议，使用默认值: FIFO", cpu_name);
    Vec::new() // CPU实现不生成代码，只保存信息
}
