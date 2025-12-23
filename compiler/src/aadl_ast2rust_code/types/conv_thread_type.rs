use crate::aadl_ast2rust_code::intermediate_ast::*;

use crate::aadl_ast2rust_code::converter::AadlConverter;
use crate::ast::aadl_ast_cj::*;
use std::collections::HashMap;

pub fn convert_thread_component(
    temp_converter: &mut AadlConverter,
    comp: &ComponentType,
) -> Vec<Item> {
    let mut items = Vec::new();

    // 1. 结构体定义
    let mut fields = temp_converter.convert_type_features(&comp.features, comp.identifier.clone()); //特征列表
                                                                                                    // 将属性也整合为字段（不再单独放入properties）
                                                                                                    // 收集属性：既向 fields 添加类型字段，又把值记录到 thread_field_values
    let mut value_map: HashMap<String, StruPropertyValue> = HashMap::new();
    let mut type_map: HashMap<String, Type> = HashMap::new();

    // 将特征字段也加入到 thread_field_values 和 thread_field_types 中，根据字段类型决定值，记录Shared的字段类型
    for field in &fields {
        let field_value = match &field.ty {
            Type::Named(type_name) => {
                if type_name.ends_with("Shared") {
                    StruPropertyValue::Custom(field.name.clone())
                } else {
                    StruPropertyValue::None
                }
            }
            _ => StruPropertyValue::None,
        };
        value_map.insert(field.name.clone(), field_value);
        type_map.insert(field.name.clone(), field.ty.clone());
    }
    if let PropertyClause::Properties(props) = &comp.properties {
        for prop in props {
            if let Property::BasicProperty(bp) = prop {
                if let Some(val) = temp_converter.parse_property_value(&bp.value) {
                    let name_lc = bp.identifier.name.to_lowercase();
                    let ty_name = temp_converter.type_for_property(&val);
                    fields.push(Field {
                        name: name_lc.clone(),
                        ty: Type::Named(ty_name),
                        docs: vec![format!("// AADL属性: {}", bp.identifier.name)],
                        attrs: Vec::new(),
                    });
                    value_map.insert(name_lc, val);
                }
            }
        }
    }
    // 添加 CPU ID 字段
    fields.push(Field {
        name: "cpu_id".to_string(),
        ty: Type::Named("isize".to_string()),
        docs: vec!["// 结构体新增 CPU ID".to_string()],
        attrs: Vec::new(),
    });

    let struct_name = format!("{}Thread", comp.identifier.to_lowercase());
    // 将字段对应的属性值保存起来（仅保存存在值的属性字段）
    if !value_map.is_empty() {
        temp_converter
            .thread_field_values
            .insert(struct_name.clone(), value_map);
    }
    // 将字段对应的类型保存起来（仅保存存在值的属性字段）
    if !type_map.is_empty() {
        temp_converter
            .thread_field_types
            .insert(struct_name.clone(), type_map);
    }

    let struct_def = StructDef {
        name: struct_name,
        fields,                 //特征列表
        properties: Vec::new(), // 属性字段已整合进 fields
        generics: Vec::new(),
        derives: vec!["Debug".to_string()],
        docs: temp_converter.create_component_type_docs(comp),
        vis: Visibility::Public, //默认public
    };
    items.push(Item::Struct(struct_def));
    // 2. 实现块
    // if let Some(impl_block) = self.create_threadtype_impl(comp) {
    //     items.push(Item::Impl(impl_block));
    // }

    items
}
