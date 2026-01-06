use crate::aadl_ast2rust_code::intermediate_ast::*;
use crate::ast::aadl_ast_cj::*;
use std::collections::HashMap;

pub fn convert_data_component(
    type_mappings: &mut HashMap<String, Type>,
    comp: &ComponentType,
    data_comp_type: &mut HashMap<String, String>,
) -> Vec<Item> {
    let target_type = determine_data_type(type_mappings, comp);
    // 当 determine_data_type 返回结构体类型时，生成结构体定义
    // 当 determine_data_type 返回联合体类型时，生成枚举定义
    // 当 determine_data_type 返回枚举类型时，生成枚举定义
    //TODO:这里需要处理，当确定为复杂类型时，需要将组件标识符添加到type_mappings中
    if let Type::Named(unit_type) = &target_type {
        if unit_type.to_lowercase() == "struct" {
            // 从组件属性中提取属性列表
            if let PropertyClause::Properties(props) = &comp.properties {
                let struct_def = determine_struct_type(type_mappings, comp, props, data_comp_type);

                // 只有当组件标识符不存在于type_mappings中时，才添加到type_mappings中
                // !不添加，因为"struct"没有用，需要用别名。
                // if !type_mappings.contains_key(&comp.identifier.to_lowercase()) {
                //     type_mappings.insert(comp.identifier.to_lowercase(), target_type.clone());
                // }

                if struct_def.fields.is_empty() {
                    //说明是通过impl中子组件来获取字段的，而不是在此时type中
                    return Vec::new();
                } else {
                    return vec![Item::Struct(struct_def)];
                }
            }
            // } else { //这里的else是多余的，当comp.properties为空时，determine_struct_type的返回值就是空
            //     // 如果没有属性，返回空的结构体
            //     return vec![Item::Struct(determine_struct_type(
            //         type_mappings,
            //         comp,
            //         &[],
            //         data_comp_type,
            //     ))];
            // }
        } else if unit_type.to_lowercase() == "union" {
            // 从组件属性中提取属性列表
            if let PropertyClause::Properties(props) = &comp.properties {
                let union_def = determine_union_type(type_mappings, comp, props, data_comp_type);
                if union_def.fields.is_empty() {
                    //说明是通过impl中子组件来获取字段的，而不是在此时type中
                    return Vec::new();
                } else {
                    return vec![Item::Union(union_def)];
                }
            }
            //这里的else是多余的，当comp.properties为空时，determine_union_type的返回值就是空
            // else {
            //     // 如果没有属性，返回空的枚举
            //     return vec![Item::Union(determine_union_type(
            //         type_mappings,
            //         comp,
            //         &[],
            //         data_comp_type,
            //     ))];
            // }
        } else if unit_type.to_lowercase() == "enum" {
            // 从组件属性中提取属性列表
            if let PropertyClause::Properties(props) = &comp.properties {
                return vec![Item::Enum(determine_enum_type(comp, props))];
            }
            // } else {
            //     // 如果没有属性，返回空的枚举
            //     return vec![Item::Enum(determine_enum_type(comp, &[]))];
            // }
        } else if unit_type.to_lowercase() == "taggedunion" {
            // 从组件属性中提取属性列表
            if let PropertyClause::Properties(props) = &comp.properties {
                let taggedunion_def =
                    determine_taggedunion_type(type_mappings, comp, props, data_comp_type);

                // 只有当组件标识符不存在于type_mappings中时，才添加到type_mappings中
                // if !type_mappings.contains_key(&comp.identifier.to_lowercase()) {
                //     type_mappings.insert(comp.identifier.to_lowercase(), target_type.clone());
                // }

                if taggedunion_def.variants.is_empty() {
                    //说明是通过impl中子组件来获取字段的，而不是在此时type中
                    return Vec::new();
                } else {
                    return vec![Item::Enum(taggedunion_def)];
                }
            }
            //这里的else是多余的，当comp.properties为空时，determine_taggedunion_type的返回值就是空
            // else {
            //     // 如果没有属性，返回空的标记联合体
            //     return vec![Item::Enum(determine_taggedunion_type(
            //         type_mappings,
            //         comp,
            //         &[],
            //         data_comp_type,
            //     ))];
            // }
        }
    }
    // 只有当组件标识符不存在于type_mappings中时，才添加到type_mappings中
    if !type_mappings.contains_key(&comp.identifier.to_lowercase()) {
        //println!("3333comp.identifier: {:?}", comp.identifier.to_lowercase());
        type_mappings.insert(comp.identifier.to_lowercase(), target_type.clone());
    }

    vec![Item::TypeAlias(TypeAlias {
        name: comp.identifier.clone(),
        target: target_type,
        vis: Visibility::Public,
        docs: vec![format!("// AADL Data Type: {}", comp.identifier.clone())],
    })]
}

fn determine_data_type(type_mappings: &HashMap<String, Type>, comp: &ComponentType) -> Type {
    // 首先检查组件标识符是否已经存在于type_mappings中
    if let Some(existing_type) = type_mappings.get(&comp.identifier.to_lowercase()) {
        return existing_type.clone();
    }

    // 如果没有找到，则处理复杂类型
    determine_complex_data_type(type_mappings, comp)
}

/// 处理复杂数据类型，包括数组、结构体、联合体、枚举等
fn determine_complex_data_type(
    type_mappings: &HashMap<String, Type>,
    comp: &ComponentType,
) -> Type {
    if let PropertyClause::Properties(props) = &comp.properties {
        for prop in props {
            if let Property::BasicProperty(bp) = prop {
                // 处理 Data_Model::Data_Representation 属性
                // 检查属性集是否为 "Data_Model" 且属性名为 "Data_Representation"
                if let Some(property_set) = &bp.identifier.property_set {
                    if property_set.to_lowercase() == "data_model"
                        && bp.identifier.name.to_lowercase() == "data_representation"
                    {
                        if let PropertyValue::Single(PropertyExpression::String(
                            StringTerm::Literal(str_val),
                        )) = &bp.value
                        {
                            match str_val.to_lowercase().as_str() {
                                "array" => {
                                    return determine_array_type(type_mappings, props);
                                }
                                "struct" => {
                                    return Type::Named("struct".to_string());
                                }
                                "union" => {
                                    return Type::Named("union".to_string());
                                }
                                "enum" => {
                                    return Type::Named("enum".to_string());
                                }
                                "taggedunion" => {
                                    return Type::Named("taggedunion".to_string());
                                }
                                _ => {
                                    // 使用 type_mappings 查找对应的类型，如果没有找到则使用原值
                                    return type_mappings
                                        .get(&str_val.to_string().to_lowercase())
                                        .cloned()
                                        .unwrap_or_else(|| Type::Named(str_val.to_string()));
                                }
                            }
                        }
                    }
                }

                // 处理 type_source_name 属性，用于指定数据类型
                if bp.identifier.name.to_lowercase() == "type_source_name" {
                    if let PropertyValue::Single(PropertyExpression::String(StringTerm::Literal(
                        str_val,
                    ))) = &bp.value
                    {
                        return type_mappings
                            .get(&str_val.to_string().to_lowercase())
                            .cloned()
                            .unwrap_or_else(|| Type::Named(str_val.to_string()));
                    }
                }
            }
        }
    }
    Type::Named("()".to_string())
}

/// 处理数组类型
fn determine_array_type(type_mappings: &HashMap<String, Type>, props: &[Property]) -> Type {
    let mut base_type = Type::Named("i32".to_string()); // 默认基础类型
    let mut dimensions = Vec::new();

    // 查找 Base_Type 属性
    for prop in props {
        //println!("prop: {:?}", prop);
        if let Property::BasicProperty(bp) = prop {
            if bp.identifier.name.to_lowercase() == "base_type" {
                if let PropertyValue::Single(PropertyExpression::ComponentClassifier(
                    ComponentClassifierTerm {
                        unique_component_classifier_reference:uccr,
                    },
                )) = &bp.value
                {
                    if let UniqueComponentClassifierReference::Type(impl_ref) = uccr {
                        let type_name = impl_ref.implementation_name.type_identifier.clone();
                        base_type = type_mappings
                            .get(&type_name.to_lowercase())
                            .cloned()
                            //.unwrap()
                            .expect("type_mappings must contain the base type");
                            //.unwrap_or_else(|| Type::Named(type_name.clone()));
                        }
                }
            }

            // 查找 Dimension 属性
            if bp.identifier.name.to_lowercase() == "dimension" {
                match &bp.value {
                    PropertyValue::Single(PropertyExpression::Integer(
                        SignedIntergerOrConstant::Real(int_val),
                    )) => {
                        dimensions.push(int_val.value as usize);
                    }
                    PropertyValue::List(dim_list) => {
                        for dim_item in dim_list {
                            if let PropertyListElement::Value(PropertyExpression::Integer(
                                SignedIntergerOrConstant::Real(int_val),
                            )) = dim_item
                            {
                                dimensions.push(int_val.value as usize);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 如果没有找到维度信息，默认为一维数组
    if dimensions.is_empty() {
        dimensions.push(1);
    }

    // 构建数组类型：从内到外构建嵌套数组
    let mut array_type = base_type;
    for &dim in dimensions.iter().rev() {
        array_type = Type::Array(Box::new(array_type), dim);
    }

    array_type
}

/// 处理结构体类型
fn determine_struct_type(
    type_mappings: &HashMap<String, Type>,
    comp: &ComponentType,
    props: &[Property],
    data_comp_type: &mut HashMap<String, String>,
) -> StructDef {
    let mut fields = Vec::new();
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();

    // 解析字段类型和字段名
    for prop in props {
        if let Property::BasicProperty(bp) = prop {
            // 解析 Base_Type 属性获取字段类型
            if let Some(property_set) = &bp.identifier.property_set {
                if property_set.to_lowercase() == "data_model"
                    && bp.identifier.name.to_lowercase() == "base_type"
                {
                    if let PropertyValue::List(type_list) = &bp.value {
                        for type_item in type_list {
                            if let PropertyListElement::Value(
                                PropertyExpression::ComponentClassifier(ComponentClassifierTerm {
                                    unique_component_classifier_reference,
                                }),
                            ) = type_item
                            {
                                // 从分类器引用中提取类型名
                                // 不会有实现引用
                                // 使用match,不使用if
                                let type_name = match unique_component_classifier_reference {
                                    UniqueComponentClassifierReference::Type(impl_ref) => impl_ref.implementation_name.type_identifier.clone(),
                                    UniqueComponentClassifierReference::Implementation(_impl_ref) => "".to_string(),
                                };
                                // let type_name = if let UniqueComponentClassifierReference::Type(impl_ref) = unique_component_classifier_reference {
                                //     impl_ref.implementation_name.type_identifier.clone()
                                // } else {
                                //     "".to_string()
                                // };

                                // 映射到 Rust 类型
                                let rust_type = type_mappings
                                    .get(&type_name.to_string().to_lowercase())
                                    .cloned()
                                    .unwrap_or_else(|| Type::Named(type_name));

                                field_types.push(rust_type);
                            }
                        }
                    }
                }
            }

            // 解析 Element_Names 属性获取字段名
            if let Some(property_set) = &bp.identifier.property_set {
                if property_set.to_lowercase() == "data_model"
                    && bp.identifier.name.to_lowercase() == "element_names"
                {
                    if let PropertyValue::List(name_list) = &bp.value {
                        for name_item in name_list {
                            if let PropertyListElement::Value(PropertyExpression::String(
                                StringTerm::Literal(name),
                            )) = name_item
                            {
                                field_names.push(name.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    // 判断是否获取到字段信息，理论上二者同时有或同时无
    if field_names.is_empty() || field_types.is_empty() {
        //说明没有获取到字段信息，需要根据组件实现impl来获取属性信息
        //存储信息到全局数据结构中
        data_comp_type.insert(comp.identifier.clone(), "struct".to_string());
    }
    // 创建字段
    for (name, ty) in field_names.iter().zip(field_types.iter()) {
        fields.push(Field {
            name: name.clone(),
            ty: ty.clone(),
            docs: vec!["".to_string()],
            attrs: vec![],
        });
    }

    // 创建结构体定义
    StructDef {
        name: comp.identifier.clone(),
        fields,
        properties: vec![],
        generics: vec![],
        derives: vec!["Debug".to_string(), "Clone".to_string()],
        docs: vec![format!("// AADL Struct: {}", comp.identifier)],
        vis: Visibility::Public,
    }
}

/// 处理联合体类型,unsafe联合体
fn determine_union_type(
    type_mappings: &HashMap<String, Type>,
    comp: &ComponentType,
    props: &[Property],
    data_comp_type: &mut HashMap<String, String>,
) -> UnionDef {
    // 解析字段类型和字段名
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();

    for prop in props {
        if let Property::BasicProperty(bp) = prop {
            // 解析 Base_Type 属性获取字段类型
            if let Some(property_set) = &bp.identifier.property_set {
                if property_set.to_lowercase() == "data_model"
                    && bp.identifier.name.to_lowercase() == "base_type"
                {
                    if let PropertyValue::List(type_list) = &bp.value {
                        for type_item in type_list {
                            if let PropertyListElement::Value(
                                PropertyExpression::ComponentClassifier(ComponentClassifierTerm {
                                    unique_component_classifier_reference,
                                }),
                            ) = type_item
                            {
                                // 从分类器引用中提取类型名
                                // 不会有实现引用
                                // 使用match,不使用if
                                let type_name = match unique_component_classifier_reference {
                                    UniqueComponentClassifierReference::Type(impl_ref) => impl_ref.implementation_name.type_identifier.clone(),
                                    UniqueComponentClassifierReference::Implementation(_impl_ref) => "".to_string(),
                                };
                                // let type_name = if let UniqueComponentClassifierReference::Type(impl_ref) = unique_component_classifier_reference {
                                //     impl_ref.implementation_name.type_identifier.clone()
                                // } else {
                                //     "".to_string()
                                // };

                                // 映射到 Rust 类型
                                let rust_type = type_mappings
                                    .get(&type_name.to_string().to_lowercase())
                                    .cloned()
                                    .unwrap_or_else(|| Type::Named(type_name));

                                field_types.push(rust_type);
                            }
                        }
                    }
                }
            }

            // 解析 Element_Names 属性获取字段名
            if let Some(property_set) = &bp.identifier.property_set {
                if property_set.to_lowercase() == "data_model"
                    && bp.identifier.name.to_lowercase() == "element_names"
                {
                    if let PropertyValue::List(name_list) = &bp.value {
                        for name_item in name_list {
                            if let PropertyListElement::Value(PropertyExpression::String(
                                StringTerm::Literal(name),
                            )) = name_item
                            {
                                field_names.push(name.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // 判断是否获取到字段信息，理论上二者同时有或同时无
    if field_names.is_empty() || field_types.is_empty() {
        //说明没有获取到字段信息，需要根据组件实现impl来获取属性信息
        //存储信息到全局数据结构中
        data_comp_type.insert(comp.identifier.clone(), "union".to_string());
    }

    // 创建联合体字段
    let mut fields = Vec::new();
    for (name, ty) in field_names.iter().zip(field_types.iter()) {
        fields.push(Field {
            name: name.clone(),
            ty: ty.clone(),
            docs: vec!["".to_string()],
            attrs: vec![],
        });
    }

    // 创建联合体定义
    UnionDef {
        name: comp.identifier.clone(),
        fields,
        properties: vec![],
        generics: vec![],
        derives: vec!["Debug".to_string(), "Clone".to_string()],
        docs: vec![format!("// AADL Union: {}", comp.identifier)],
        vis: Visibility::Public,
    }
}

/// 处理枚举类型
fn determine_enum_type(comp: &ComponentType, props: &[Property]) -> EnumDef {
    // 解析枚举值名称
    let mut variant_names = Vec::new();

    for prop in props {
        if let Property::BasicProperty(bp) = prop {
            // 解析 Enumerators 属性获取枚举值名称
            if let Some(property_set) = &bp.identifier.property_set {
                if property_set.to_lowercase() == "data_model"
                    && bp.identifier.name.to_lowercase() == "enumerators"
                {
                    if let PropertyValue::List(name_list) = &bp.value {
                        for name_item in name_list {
                            if let PropertyListElement::Value(PropertyExpression::String(
                                StringTerm::Literal(name),
                            )) = name_item
                            {
                                variant_names.push(name.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // 创建枚举变体（无数据类型）
    let mut variants = Vec::new();
    for name in variant_names {
        variants.push(Variant {
            name: name.clone(),
            data: None, // 枚举变体不包含数据类型
            docs: vec![],
        });
    }

    // 创建枚举定义
    EnumDef {
        name: comp.identifier.clone(),
        variants,
        generics: vec![],
        derives: vec!["Debug".to_string(), "Clone".to_string()],
        docs: vec![format!("// AADL Enum: {}", comp.identifier)],
        vis: Visibility::Public,
    }
}

/// 处理带标签的联合体类型，生成带类型的枚举
fn determine_taggedunion_type(
    type_mappings: &HashMap<String, Type>,
    comp: &ComponentType,
    props: &[Property],
    data_comp_type: &mut HashMap<String, String>,
) -> EnumDef {
    // 解析字段名和字段类型
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();

    for prop in props {
        if let Property::BasicProperty(bp) = prop {
            // 解析 Base_Type 属性获取字段类型
            if let Some(property_set) = &bp.identifier.property_set {
                if property_set.to_lowercase() == "data_model"
                    && bp.identifier.name.to_lowercase() == "base_type"
                {
                    if let PropertyValue::List(type_list) = &bp.value {
                        for type_item in type_list {
                            if let PropertyListElement::Value(
                                PropertyExpression::ComponentClassifier(ComponentClassifierTerm {
                                    unique_component_classifier_reference,
                                }),
                            ) = type_item
                            {
                                // 从分类器引用中提取类型名
                                // 不会有实现引用
                                
                                // 使用match,不使用if
                                let type_name = match unique_component_classifier_reference {
                                    UniqueComponentClassifierReference::Type(impl_ref) => impl_ref.implementation_name.type_identifier.clone(),
                                    UniqueComponentClassifierReference::Implementation(_impl_ref) => "".to_string(),
                                };
                                //  let type_name = if let UniqueComponentClassifierReference::Type(impl_ref) = unique_component_classifier_reference {
                                //     impl_ref.implementation_name.type_identifier.clone()
                                // } else {
                                //     "".to_string()
                                // };

                                // 映射到 Rust 类型
                                let rust_type = type_mappings
                                    .get(&type_name.to_string().to_lowercase())
                                    .cloned()
                                    .unwrap_or_else(|| Type::Named(type_name));

                                field_types.push(rust_type);
                            }
                        }
                    }
                }
            }

            // 解析 Element_Names 属性获取字段名
            if let Some(property_set) = &bp.identifier.property_set {
                if property_set.to_lowercase() == "data_model"
                    && bp.identifier.name.to_lowercase() == "element_names"
                {
                    if let PropertyValue::List(name_list) = &bp.value {
                        for name_item in name_list {
                            if let PropertyListElement::Value(PropertyExpression::String(
                                StringTerm::Literal(name),
                            )) = name_item
                            {
                                field_names.push(name.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // 判断是否获取到字段信息，理论上二者同时有或同时无
    if field_names.is_empty() || field_types.is_empty() {
        //说明没有获取到字段信息，需要根据组件实现impl来获取属性信息
        //存储信息到全局数据结构中
        data_comp_type.insert(comp.identifier.clone(), "taggedunion".to_string());
    }

    // 创建枚举变体（带数据类型）
    let mut variants = Vec::new();
    for (name, ty) in field_names.iter().zip(field_types.iter()) {
        // 将字段名首字母大写，例如 "f1" -> "F1" ,不考虑name为空的情况
        
        let mut chars = name.chars();
        let variant_name = match chars.next() {
            None => "Default".to_string(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        };

        variants.push(Variant {
            name: variant_name,
            data: Some(vec![ty.clone()]), // 带标签的联合体变体包含数据类型
            docs: vec![],
        });
    }

    // 创建枚举定义
    EnumDef {
        name: comp.identifier.clone(),
        variants,
        generics: vec![],
        derives: vec!["Debug".to_string(), "Clone".to_string()],
        docs: vec![format!("// AADL Tagged Union: {}", comp.identifier)],
        vis: Visibility::Public,
    }
}
