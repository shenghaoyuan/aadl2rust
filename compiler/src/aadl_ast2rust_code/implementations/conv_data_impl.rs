#![allow(clippy::all)]
use crate::aadl_ast2rust_code::intermediate_ast::*;
use crate::ast::aadl_ast_cj::*;
use std::collections::HashMap;


pub fn convert_data_implementation(
    type_mappings: &HashMap<String, Type>,
    data_comp_type: &HashMap<String, String>,
    impl_: &ComponentImplementation,
    package: &Package,
) -> Vec<Item> {
    let mut items = Vec::new();
    
    // 检查子组件，判断是否为共享变量/复杂数据类型,它们都具有subcomponents
    if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {

        // 过滤出data子组件
        let data_subcomponents: Vec<_> = subcomponents
            .iter()
            .filter(|sub| sub.category == ComponentCategory::Data)
            .cloned()
            .collect();

        if data_comp_type.contains_key(&impl_.name.type_identifier) {
            //说明是复杂数据类型
            let data_type_name = data_comp_type.get(&impl_.name.type_identifier).unwrap();
            if data_type_name == "struct" {
                items.push(Item::Struct(determine_struct_impl(
                    type_mappings,
                    impl_,
                    data_subcomponents.as_slice(),
                )));
            } else if data_type_name == "union" {
                items.push(Item::Union(determine_union_impl(
                    type_mappings,
                    impl_,
                    data_subcomponents.as_slice(),
                )));
            } else if data_type_name == "taggedunion" {
                items.push(Item::Enum(determine_taggedunion_impl(
                    type_mappings,
                    impl_,
                    data_subcomponents.as_slice(),
                )));
            }
        }

        // 过滤出subprogram子组件
        let subprogram_subcomponents: Vec<_> = subcomponents
            .iter()
            .filter(|sub| sub.category == ComponentCategory::Subprogram)
            .cloned()
            .collect();
        // 针对subprogram_subcomponents的每一个，查看它的
        let mut subprogram_methods = Vec::new();
        for sub in &subprogram_subcomponents {
            // 获取子程序的实现引用
            if let SubcomponentClassifier::ClassifierReference(
                UniqueComponentClassifierReference::Implementation(impl_ref),
            ) = &sub.classifier
            {
                let subprogram_impl_name = &impl_ref.implementation_name.type_identifier;
                // println!(
                //     "数据组件实现 {} 包含子程序子组件: {}",
                //     impl_.name.type_identifier, subprogram_impl_name
                // );
                // 这里可以根据需要生成与子程序相关的代码,子程序名称是 subprogram_impl_name的全小写

                // 生成子程序调用方法
                let method_name = subprogram_impl_name.to_lowercase();
                let mut method_body = Vec::new();

                // 生成一次调用，参数是全部的data_subcomponents
                let mut call_args = Vec::new();
                for data_sub in &data_subcomponents {
                    let field_name = data_sub.identifier.clone();
                    call_args.push(Expr::Reference(
                        Box::new(Expr::Path(
                            vec!["self".to_string(), field_name],
                            PathType::Member,
                        )),
                        true, // &
                        true, // mut
                    ));
                }

                let call_expr = Expr::Call(
                    Box::new(Expr::Path(
                        vec![subprogram_impl_name.to_lowercase(), "call".to_string()],
                        PathType::Namespace,
                    )),
                    call_args,
                );
                method_body.push(Statement::Expr(call_expr));

                let method = ImplItem::Method(FunctionDef {
                    name: method_name.clone(),
                    params: vec![Param {
                        name: "self".to_string(),
                        ty: Type::Reference(Box::new(Type::Named("Self".to_string())), true, true),
                    }],
                    return_type: Type::Unit,
                    body: Block { stmts: method_body, expr: None },
                    asyncness: false,
                    vis: Visibility::Public,
                    docs: vec![format!("/// {} : provides subprogram access {};", method_name, subprogram_impl_name)],
                    attrs: Vec::new(),
                });

                subprogram_methods.push(method);
            }
        }

        // 检查该 data 是否为共享变量（被某个 process 使用）
        let is_shared_data = is_shared_data_component(package, &impl_.name.type_identifier);
        // 如果是共享变量，才生成 new() 方法用于初始化字段
        if is_shared_data {
            let mut field_initializations = Vec::new();

            for sub in &data_subcomponents {
                let field_name = sub.identifier.clone();
                // 为简单起见，使用 0 作为默认值，用户可以根据需要修改
                field_initializations.push(format!("            {}: 0", field_name));
            }

            let struct_init_code = format!("return {} {{\n{}\n        }}", impl_.name.type_identifier, field_initializations.join(",\n"));

            let new_method = ImplItem::Method(FunctionDef {
                name: "new".to_string(),
                params: vec![],
                return_type: Type::Named("Self".to_string()),
                body: Block {
                    stmts: vec![Statement::Expr(Expr::Ident(struct_init_code))],
                    expr: None,
                },
                asyncness: false,
                vis: Visibility::Public,
                docs: vec![
                    format!("// Creates a new instance of {}", impl_.name.type_identifier),
                ],
                attrs: Vec::new(),
            });
            subprogram_methods.push(new_method);
        }

        // 如果有子程序方法，生成 impl 块
        if !subprogram_methods.is_empty() {
            let impl_block = ImplBlock {
                target: Type::Named(impl_.name.type_identifier.clone()),
                generics: Vec::new(),
                items: subprogram_methods,
                trait_impl: None,
            };
            items.push(Item::Impl(impl_block));
        }

        if is_shared_data {
            // 生成共享变量类型
            let shared_type_name = format!("{}Shared", impl_.name.type_identifier);
            let shared_type = Type::Generic(
                "Arc".to_string(),
                vec![Type::Generic("Mutex".to_string(), vec![Type::Named(impl_.name.type_identifier.clone())])],
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
        }

        // let subprogram_count = subcomponents
        //     .iter()
        //     .filter(|sub| sub.category == ComponentCategory::Subprogram)
        //     .count();

        // 错误的认知，已修正：如果有多个子程序，说明是共享变量；暂时不支持Data中有大于1个共享数据
        // if subprogram_count > 1 {
        //     if data_subcomponents.len() == 1 {
        //         // 获取数据子组件的类型名（用于Arc<Mutex<T>>中的T）
        //         let data_type_name = match &data_subcomponents[0].classifier {
        //             SubcomponentClassifier::ClassifierReference(
        //                 UniqueComponentClassifierReference::Implementation(unirf),
        //             ) => {
        //                 format!("{}", unirf.implementation_name.type_identifier)
        //             }
        //             _ => "UnknownType".to_string(),
        //         };

        //         // 生成共享变量类型定义：从数据组件实现名称（如POS.Impl）提取POS部分，然后加上Shared
        //         let shared_type_name = {
        //             // 从 impl_.name.type_identifier 中提取实现名称（去掉可能的Impl后缀）
        //             let impl_name = &impl_.name.type_identifier;
        //             format!("{}Shared", impl_name)
        //         };

        //         // 生成 Arc<Mutex<T>> 类型
        //         let shared_type = Type::Generic(
        //             "Arc".to_string(),
        //             vec![Type::Generic(
        //                 "Mutex".to_string(),
        //                 vec![Type::Named(data_type_name)],
        //             )],
        //         );

        //         let type_alias = TypeAlias {
        //             name: shared_type_name,
        //             target: shared_type,
        //             vis: Visibility::Public,
        //             docs: vec![
        //                 format!("// Shared data type for {}", impl_.name.type_identifier),
        //                 "// Auto-generated from AADL data implementation".to_string(),
        //             ],
        //         };

        //         items.push(Item::TypeAlias(type_alias));
        //     } else if data_subcomponents.len() > 1 {
        //         // 输出报错信息：不支持多个共享数据
        //         eprintln!(
        //             "错误：数据组件实现 {} 中有 {} 个数据子组件，暂时不支持多个共享数据",
        //             impl_.name.type_identifier,
        //             data_subcomponents.len()
        //         );
        //         eprintln!("请检查AADL模型,确保每个共享数据组件实现中只有一个数据子组件");
        //     }
        // }
    }

    items
}

/// 检查数据组件是否为共享变量（被某个 process 使用）
fn is_shared_data_component(package: &Package, data_impl_name: &str) -> bool {
    // 在 public 部分查找
    if let Some(public_section) = &package.public_section {
        for decl in &public_section.declarations {
            if let AadlDeclaration::ComponentImplementation(process_impl) = decl {
                if process_impl.category == ComponentCategory::Process {
                    if let SubcomponentClause::Items(process_subcomponents) = &process_impl.subcomponents {
                        for sub in process_subcomponents {
                            if sub.category == ComponentCategory::Data {
                                if let SubcomponentClassifier::ClassifierReference(
                                    UniqueComponentClassifierReference::Implementation(data_ref),
                                ) = &sub.classifier
                                {
                                    if data_ref.implementation_name.type_identifier == data_impl_name {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 在私有部分也查找
    if let Some(private_section) = &package.private_section {
        for decl in &private_section.declarations {
            if let AadlDeclaration::ComponentImplementation(process_impl) = decl {
                if process_impl.category == ComponentCategory::Process {
                    if let SubcomponentClause::Items(process_subcomponents) = &process_impl.subcomponents {
                        for sub in process_subcomponents {
                            if sub.category == ComponentCategory::Data {
                                if let SubcomponentClassifier::ClassifierReference(
                                    UniqueComponentClassifierReference::Implementation(data_ref),
                                ) = &sub.classifier
                                {
                                    if data_ref.implementation_name.type_identifier == data_impl_name {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
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
            _ => Type::Named("UnknownType".to_string()),
            // SubcomponentClassifier::ClassifierReference(
            //     UniqueComponentClassifierReference::Type(type_ref),
            // ) => {
            //     // 从类型引用中提取类型名
            //     let type_name = type_ref.implementation_name.type_identifier.clone();

            //     // 映射到 Rust 类型
            //     type_mappings
            //         .get(&type_name.to_lowercase())
            //         .cloned()
            //         .unwrap_or_else(|| Type::Named(type_name))
            // }
            // SubcomponentClassifier::Prototype(prototype_name) => {
            //     // 处理原型引用
            //     Type::Named(prototype_name.clone())
            // }
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

/// 处理联合体类型,unsafe
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
            _ => Type::Named("UnknownType".to_string()),
            // SubcomponentClassifier::ClassifierReference(
            //     UniqueComponentClassifierReference::Type(type_ref),
            // ) => {
            //     // 从类型引用中提取类型名
            //     let type_name = type_ref.implementation_name.type_identifier.clone();

            //     // 映射到 Rust 类型
            //     type_mappings
            //         .get(&type_name.to_lowercase())
            //         .cloned()
            //         .unwrap_or_else(|| Type::Named(type_name))
            // }
            // SubcomponentClassifier::Prototype(prototype_name) => {
            //     // 处理原型引用
            //     Type::Named(prototype_name.clone())
            // }
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

/// 处理带标签的联合体类型，从实现中的子组件生成带类型的枚举
fn determine_taggedunion_impl(
    type_mappings: &HashMap<String, Type>,
    impl_: &ComponentImplementation,
    subcomponents: &[Subcomponent],
) -> EnumDef {
    let mut variants = Vec::new();

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
            _ => Type::Named("UnknownType".to_string()),
            // SubcomponentClassifier::ClassifierReference(
            //     UniqueComponentClassifierReference::Type(type_ref),
            // ) => {
            //     // 从类型引用中提取类型名
            //     let type_name = type_ref.implementation_name.type_identifier.clone();

            //     // 映射到 Rust 类型
            //     type_mappings
            //         .get(&type_name.to_lowercase())
            //         .cloned()
            //         .unwrap_or_else(|| Type::Named(type_name))
            // }
            // SubcomponentClassifier::Prototype(prototype_name) => {
            //     // 处理原型引用
            //     Type::Named(prototype_name.clone())
            // }
        };

        // 将字段名首字母大写，例如 "f1" -> "F1" 
        let mut chars = field_name.chars();
        let variant_name = match chars.next() {
            None => "Default".to_string(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        };

        // 创建枚举变体（带数据类型）
        variants.push(Variant {
            name: variant_name,
            data: Some(vec![field_type]), // 带标签的联合体变体包含数据类型
            docs: vec![format!("// 标记联合体字段: {}", sub.identifier)],
        });
    }

    // 创建枚举定义
    EnumDef {
        name: impl_.name.type_identifier.clone(),
        variants,
        generics: vec![],
        derives: vec!["Debug".to_string(), "Clone".to_string()],
        docs: vec![format!("// AADL Tagged Union: {}", impl_.name.type_identifier)],
        vis: Visibility::Public,
    }
}