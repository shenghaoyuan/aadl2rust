use crate::aadlAst2rustCode::intermediate_ast::*;
use crate::ast::aadl_ast_cj::*;
use std::collections::HashMap;

pub fn convert_data_implementation(
    type_mappings: &HashMap<String, Type>,
    data_comp_type: &HashMap<String, String>,
    impl_: &ComponentImplementation,
) -> Vec<Item> {
    let mut items = Vec::new();

    // 检查子组件，判断是否为共享变量/复杂数据类型
    if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
        let subprogram_count = subcomponents
            .iter()
            .filter(|sub| sub.category == ComponentCategory::Subprogram)
            .count();

        let data_subcomponents: Vec<_> = subcomponents
            .iter()
            .filter(|sub| sub.category == ComponentCategory::Data)
            .collect();

        // 如果有多个子程序，说明是共享变量；暂时不支持Data中有大于1个共享数据
        if subprogram_count > 1 {
            if data_subcomponents.len() == 1 {
                // 获取数据子组件的类型名（用于Arc<Mutex<T>>中的T）
                let data_type_name = match &data_subcomponents[0].classifier {
                    SubcomponentClassifier::ClassifierReference(
                        UniqueComponentClassifierReference::Implementation(unirf),
                    ) => {
                        format!("{}", unirf.implementation_name.type_identifier)
                    }
                    _ => "UnknownType".to_string(),
                };

                // 生成共享变量类型定义：从数据组件实现名称（如POS.Impl）提取POS部分，然后加上Shared
                let shared_type_name = {
                    // 从 impl_.name.type_identifier 中提取实现名称（去掉可能的Impl后缀）
                    let impl_name = &impl_.name.type_identifier;
                    format!("{}Shared", impl_name)
                };

                // 生成 Arc<Mutex<T>> 类型
                let shared_type = Type::Generic(
                    "Arc".to_string(),
                    vec![Type::Generic(
                        "Mutex".to_string(),
                        vec![Type::Named(data_type_name)],
                    )],
                );

                let type_alias = TypeAlias {
                    name: shared_type_name,
                    target: shared_type,
                    vis: Visibility::Public,
                    docs: vec![
                        format!("// Shared data type for {}", impl_.name.type_identifier),
                        "// Auto-generated from AADL data implementation".to_string(),
                    ],
                };

                items.push(Item::TypeAlias(type_alias));
            } else if data_subcomponents.len() > 1 {
                // 输出报错信息：不支持多个共享数据
                eprintln!(
                    "错误：数据组件实现 {} 中有 {} 个数据子组件，暂时不支持多个共享数据",
                    impl_.name.type_identifier,
                    data_subcomponents.len()
                );
                eprintln!("请检查AADL模型，确保每个共享数据组件实现中只有一个数据子组件");
            }
        } else if data_comp_type.contains_key(&impl_.name.type_identifier) {
            //说明是复杂数据类型
            let data_type_name = data_comp_type.get(&impl_.name.type_identifier).unwrap();
            if data_type_name == "struct" {
                items.push(Item::Struct(determine_struct_impl(
                    type_mappings,
                    impl_,
                    subcomponents,
                )));
            } else if data_type_name == "union" {
                items.push(Item::Union(determine_union_impl(
                    type_mappings,
                    impl_,
                    subcomponents,
                )));
            }
        }
    }

    items
}

/// 处理结构体类型
fn determine_struct_impl(
    type_mappings: &HashMap<String, Type>,
    impl_: &ComponentImplementation,
    subcomponents: &[Subcomponent],
) -> StructDef {
    let mut fields = Vec::new();

    // 从子组件中解析字段类型和字段名
    for sub in subcomponents {
        // 获取字段名（子组件标识符）
        let field_name = sub.identifier.clone();

        // 获取字段类型
        let field_type = match &sub.classifier {
            SubcomponentClassifier::ClassifierReference(
                UniqueComponentClassifierReference::Implementation(impl_ref),
            ) => {
                // 从分类器引用中提取类型名
                let type_name = impl_ref.implementation_name.type_identifier.clone();

                // 映射到 Rust 类型
                type_mappings
                    .get(&type_name.to_lowercase())
                    .cloned()
                    .unwrap_or_else(|| Type::Named(type_name))
            }
            SubcomponentClassifier::ClassifierReference(
                UniqueComponentClassifierReference::Type(type_ref),
            ) => {
                // 从类型引用中提取类型名
                let type_name = type_ref.implementation_name.type_identifier.clone();

                // 映射到 Rust 类型
                type_mappings
                    .get(&type_name.to_lowercase())
                    .cloned()
                    .unwrap_or_else(|| Type::Named(type_name))
            }
            SubcomponentClassifier::Prototype(prototype_name) => {
                // 处理原型引用
                Type::Named(prototype_name.clone())
            }
        };

        // 创建字段
        fields.push(Field {
            name: field_name,
            ty: field_type,
            docs: vec![format!("// 子组件字段: {}", sub.identifier)],
            attrs: vec![],
        });
    }

    // 创建结构体定义
    StructDef {
        name: impl_.name.type_identifier.clone(),
        fields,
        properties: vec![],
        generics: vec![],
        derives: vec!["Debug".to_string(), "Clone".to_string()],
        docs: vec![format!("// AADL Struct: {}", impl_.name.type_identifier)],
        vis: Visibility::Public,
    }
}

/// 处理联合体类型,使用枚举类型来表示，不使用union类型,避免unsafe
fn determine_union_impl(
    type_mappings: &HashMap<String, Type>,
    impl_: &ComponentImplementation,
    subcomponents: &[Subcomponent],
) -> UnionDef {
    let mut fields = Vec::new();

    // 从子组件中解析字段类型和字段名
    for sub in subcomponents {
        // 获取字段名（子组件标识符）
        let field_name = sub.identifier.clone();

        // 获取字段类型
        let field_type = match &sub.classifier {
            SubcomponentClassifier::ClassifierReference(
                UniqueComponentClassifierReference::Implementation(impl_ref),
            ) => {
                // 从分类器引用中提取类型名
                let type_name = impl_ref.implementation_name.type_identifier.clone();

                // 映射到 Rust 类型
                type_mappings
                    .get(&type_name.to_lowercase())
                    .cloned()
                    .unwrap_or_else(|| Type::Named(type_name))
            }
            SubcomponentClassifier::ClassifierReference(
                UniqueComponentClassifierReference::Type(type_ref),
            ) => {
                // 从类型引用中提取类型名
                let type_name = type_ref.implementation_name.type_identifier.clone();

                // 映射到 Rust 类型
                type_mappings
                    .get(&type_name.to_lowercase())
                    .cloned()
                    .unwrap_or_else(|| Type::Named(type_name))
            }
            SubcomponentClassifier::Prototype(prototype_name) => {
                // 处理原型引用
                Type::Named(prototype_name.clone())
            }
        };

        // 创建字段
        fields.push(Field {
            name: field_name,
            ty: field_type,
            docs: vec![format!("// 联合体字段: {}", sub.identifier)],
            attrs: vec![],
        });
    }

    // 创建联合体定义
    UnionDef {
        name: impl_.name.type_identifier.clone(),
        fields,
        properties: vec![],
        generics: vec![],
        derives: vec!["Debug".to_string(), "Clone".to_string()],
        docs: vec![format!("// AADL Union: {}", impl_.name.type_identifier)],
        vis: Visibility::Public,
    }
}
