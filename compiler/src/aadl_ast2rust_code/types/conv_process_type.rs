
use crate::aadl_ast2rust_code::intermediate_ast::*;

use crate::aadl_ast2rust_code::converter::AadlConverter;
use crate::ast::aadl_ast_cj::*;
use crate::aadl_ast2rust_code::tool::*;

pub fn convert_process_component(
    temp_converter: &AadlConverter,
    comp: &ComponentType,
) -> Vec<Item> {
    let mut items = Vec::new();

    // 1. 结构体定义
    let mut fields = temp_converter.convert_type_features(&comp.features, comp.identifier.clone()); //特征列表
    
    // 添加 CPU ID 字段
    fields.push(Field {
        name: "cpu_id".to_string(),
        ty: Type::Named("isize".to_string()),
        docs: vec!["// 进程 CPU ID".to_string()],
        attrs: Vec::new(),
    });

    let struct_def = StructDef {
        name: format!("{}Process", to_upper_camel_case(&comp.identifier)),
        fields,                                                                   //特征列表
        properties: temp_converter.convert_properties(ComponentRef::Type(comp)), // 属性列表，TODO:这个似乎没有作用，因为目前的例子中进程没有属性
        generics: Vec::new(),
        derives: vec!["Debug".to_string()],
        docs: temp_converter.create_component_type_docs(comp),
        vis: Visibility::Public, //默认public
    };
    items.push(Item::Struct(struct_def));

    items
}
