use crate::aadlAst2rustCode::intermediate_ast::*;

use crate::aadlAst2rustCode::converter::AadlConverter;
use crate::ast::aadl_ast_cj::*;

pub fn convert_system_component(temp_converter: &AadlConverter, comp: &ComponentType) -> Vec<Item> {
    let mut items = Vec::new();

    // 1. 结构体定义 - 系统类型不包含任何字段，因为字段在实现中定义
    let struct_def = StructDef {
        name: format!("{}System", comp.identifier.to_lowercase()),
        fields: vec![], // 系统类型不包含字段
        properties: temp_converter.convert_properties(ComponentRef::Type(&comp)), //TODO:这里似乎不需要
        generics: Vec::new(),
        derives: vec!["Debug".to_string()],
        docs: vec![format!("// AADL System: {}", comp.identifier)],
        vis: Visibility::Public,
    };
    items.push(Item::Struct(struct_def));

    items
}
