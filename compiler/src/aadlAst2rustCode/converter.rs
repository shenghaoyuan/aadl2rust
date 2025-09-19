// aadlAST2rustAST
use super::intermediate_ast::*;
use super::converter_annex::AnnexConverter;
use crate::ast::aadl_ast_cj::*;
use std::collections::HashMap;

// AADL到Rust中间表示的转换器
pub struct AadlConverter {
    type_mappings: HashMap<String, Type>, //初始是根据AADL库文件Base_Types.aadl，将AADL Data组件名称映射到对应的Rust类型，后续会根据AADL模型文件，添加新的映射关系
    port_handlers: HashMap<String, PortHandlerConfig>,
    component_types: HashMap<String, ComponentType>, // 存储组件类型信息，（为了有些情况下，需要在组件实现中，根据组件类型来获取端口信息）
    annex_converter: AnnexConverter, // Behavior Annex 转换器
    cpu_scheduling_protocols: HashMap<String, String>, // 存储CPU实现的调度协议信息
    cpu_name_to_id_mapping: HashMap<String, usize>, // 存储CPU名称到ID的映射关系
    data_comp_type: HashMap<String, String>, // 存储数据组件类型信息，key是数据组件名称，value是数据组件类型。是为了处理数据组件类型为结构体、联合体时，需要根据组件实现impl来获取属性信息
}

#[derive(Debug)]
struct PortHandlerConfig {
    // 端口处理配置
}

/// 为AadlConverter实现Default trait
/// 初始化默认的类型映射关系，包括AADL基础类型到Rust类型的映射
impl Default for AadlConverter {
    fn default() -> Self {
        let mut type_mappings = HashMap::new();
        type_mappings.insert("boolean".to_string(), Type::Named("bool".to_string()));

        type_mappings.insert("integer".to_string(), Type::Named("i32".to_string()));
        type_mappings.insert("integer_8".to_string(), Type::Named("i8".to_string()));
        type_mappings.insert("integer_16".to_string(), Type::Named("i16".to_string()));
        type_mappings.insert("integer_32".to_string(), Type::Named("i32".to_string()));
        type_mappings.insert("integer_64".to_string(), Type::Named("i64".to_string()));
        type_mappings.insert("unsigned_8".to_string(), Type::Named("u8".to_string()));
        type_mappings.insert("unsigned_16".to_string(), Type::Named("u16".to_string()));
        type_mappings.insert("unsigned_32".to_string(), Type::Named("u32".to_string()));
        type_mappings.insert("unsigned_64".to_string(), Type::Named("u64".to_string()));

        type_mappings.insert("natural".to_string(), Type::Named("usize".to_string()));

        type_mappings.insert("float".to_string(), Type::Named("f32".to_string()));
        type_mappings.insert("float_32".to_string(), Type::Named("f32".to_string()));
        type_mappings.insert("float_64".to_string(), Type::Named("f64".to_string()));

        type_mappings.insert("character".to_string(), Type::Named("char".to_string()));

        type_mappings.insert("string".to_string(), Type::Named("String".to_string()));

        Self {
            type_mappings,
            port_handlers: HashMap::new(),
            component_types: HashMap::new(),
            annex_converter: AnnexConverter::default(),
            cpu_scheduling_protocols: HashMap::new(),
            cpu_name_to_id_mapping: HashMap::new(),
            data_comp_type: HashMap::new(),
        }
    }
}

impl AadlConverter {
    // 主转换入口
    pub fn convert_package(&mut self, pkg: &Package) -> RustModule {
        // 首先收集所有组件类型信息
        self.collect_component_types(pkg);
        
        let mut module = RustModule {
            name: pkg.name.0.join("_").to_lowercase(),
            docs: vec![format!(
                "// Auto-generated from AADL package: {}",
                pkg.name.0.join("::")
            )],
            //..Default::default()
            items: Default::default(),
            attrs: Default::default(),
            vis: Visibility::Public,
        };

        // 处理公共声明
        if let Some(public_section) = &pkg.public_section {
            for decl in &public_section.declarations {
                self.convert_declaration(decl, &mut module, pkg);
            }
        }

        // 处理私有声明
        if let Some(private_section) = &pkg.private_section {
            for decl in &private_section.declarations {
                self.convert_declaration(decl, &mut module, pkg);
            }
        }

        //处理CPU和分配ID的映射关系，生成的Rust代码中，初始化<ID,调度协议>的映射关系
        self.convert_cpu_schedule_mapping(&mut module, &self.cpu_scheduling_protocols, &self.cpu_name_to_id_mapping);
        self.add_period_to_priority(&mut module, &self.cpu_scheduling_protocols);
        println!("cpu_scheduling_protocols: {:?}", self.cpu_scheduling_protocols);
        println!("cpu_name_to_id_mapping: {:?}", self.cpu_name_to_id_mapping);
        module
    }

    // 收集所有组件类型信息
    fn collect_component_types(&mut self, pkg: &Package) {
        // 处理公共声明中的组件类型
        if let Some(public_section) = &pkg.public_section {
            for decl in &public_section.declarations {
                if let AadlDeclaration::ComponentType(comp) = decl {
                    self.component_types.insert(comp.identifier.clone(), comp.clone());
                }
            }
        }

        // 处理私有声明中的组件类型
        if let Some(private_section) = &pkg.private_section {
            for decl in &private_section.declarations {
                if let AadlDeclaration::ComponentType(comp) = decl {
                    self.component_types.insert(comp.identifier.clone(), comp.clone());
                }
            }
        }
    }



    // 根据实现获取组件类型
    fn get_component_type(&self, impl_: &ComponentImplementation) -> Option<&ComponentType> {
        self.component_types.get(&impl_.name.type_identifier)
    }

    // 根据端口名称获取端口方向
    fn get_port_direction(&self, port_name: &str) -> PortDirection {
        // 遍历所有组件类型，查找包含该端口的组件
        for comp_type in self.component_types.values() {
            if let FeatureClause::Items(features) = &comp_type.features {
                for feature in features {
                    if let Feature::Port(port) = feature {
                        if port.identifier.to_lowercase() == port_name.to_lowercase() {
                            return port.direction.clone();
                        }
                    }
                }
            }
        }
        // 如果找不到，默认返回 Out
        PortDirection::Out
    }

    // 根据子程序名和端口名获取端口类型
    fn get_subprogram_port_type(&self, subprogram_name: &str, port_name: &str) -> Type {
        // 遍历所有组件类型，查找子程序类型
        for comp_type in self.component_types.values() {
            if comp_type.identifier.to_lowercase() == subprogram_name.to_lowercase() {
                // 找到子程序类型，查找其中的端口
                if let FeatureClause::Items(features) = &comp_type.features {
                    for feature in features {
                        if let Feature::Port(port) = feature {
                            if port.identifier.to_lowercase() == port_name.to_lowercase() {
                                // 找到匹配的端口，返回其类型
                                return self.convert_paramport_type(port);
                            }
                        }
                    }
                }
            }
        }
        // 如果找不到，返回默认类型
        Type::Named("i32".to_string())
    }

    // 根据类型生成合适的默认值
    fn generate_default_value_for_type(&self, port_type: &Type) -> Expr {
        match port_type {
            Type::Named(type_name) => {
                // 首先检查是否是Rust原生类型
                match type_name.as_str() {
                    "bool" => Expr::Literal(Literal::Bool(false)),
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => Expr::Literal(Literal::Int(0)),
                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => Expr::Literal(Literal::Int(0)),
                    "f32" | "f64" => Expr::Literal(Literal::Float(0.0)),
                    "char" => Expr::Literal(Literal::Char('\0')),
                    "String" => Expr::Literal(Literal::Str("".to_string())),
                    _ => {
                        // 检查是否是自定义类型，通过type_mappings查找对应的Rust类型
                        if let Some(mapped_type) = self.type_mappings.get(&type_name.to_string().to_lowercase()) {
                            // 递归调用，使用映射后的类型
                            self.generate_default_value_for_type(mapped_type)
                        } else {
                            // 如果没有找到映射，使用启发式规则
                            if type_name.to_lowercase().contains("bool") {
                                Expr::Literal(Literal::Bool(false))
                            } else {
                                Expr::Literal(Literal::Int(0)) // 默认使用0
                            }
                        }
                    }
                }
            }
            _ => Expr::Literal(Literal::Int(0)), // 对于复杂类型，默认使用0
        }
    }

    fn convert_declaration(&mut self, decl: &AadlDeclaration, module: &mut RustModule, package: &Package) {
        match decl {
            AadlDeclaration::ComponentType(comp) => {
                // 转换组件类型声明，生成对应的Rust结构体或类型定义
                module.items.extend(self.convert_component(comp, package));
            }
            AadlDeclaration::ComponentImplementation(impl_) => {
                // 转换组件实现声明，生成对应的Rust实现块
                module.items.extend(self.convert_implementation(impl_));
            }
            _ => {} // TODO:忽略其他声明类型
        }
    }

    fn convert_component(&mut self, comp: &ComponentType, package: &Package) -> Vec<Item> {
        match comp.category {
            ComponentCategory::Data => self.convert_data_component(comp),
            ComponentCategory::Thread => self.convert_thread_component(comp),
            ComponentCategory::Subprogram => self.convert_subprogram(comp, package),
            ComponentCategory::System => self.convert_system_component(comp),
            ComponentCategory::Process => self.convert_process_component(comp),
            _ => Vec::default(), //TODO:其他组件类型还需要处理
        }
    }

    fn convert_data_component(&mut self, comp: &ComponentType) -> Vec<Item> {
        let target_type = self.determine_data_type(comp);
        // 当 determine_data_type 返回空元组类型时，不继续处理
        // 当 determine_data_type 返回结构体类型时，生成结构体定义
        // 当 determine_data_type 返回联合体类型时，生成枚举定义
        // 当 determine_data_type 返回枚举类型时，生成枚举定义
        if let Type::Named(unit_type) = &target_type {
            if unit_type.to_lowercase() == "struct" {
                // 从组件属性中提取属性列表
                if let PropertyClause::Properties(props) = &comp.properties {
                    let struct_def = self.determine_struct_type(comp, props);
                    if struct_def.fields.is_empty() { //说明是通过impl中子组件来获取字段的，而不是在此时type中
                        return Vec::new();
                    } else {
                        return vec![Item::Struct(struct_def)];
                    }
                } else {
                    // 如果没有属性，返回空的结构体
                    return vec![Item::Struct(self.determine_struct_type(comp, &[]))];
                }
            }
            else if unit_type.to_lowercase() == "union" {
                // 从组件属性中提取属性列表
                if let PropertyClause::Properties(props) = &comp.properties {
                    let union_def = self.determine_union_type(comp, props);
                    if union_def.fields.is_empty() { //说明是通过impl中子组件来获取字段的，而不是在此时type中
                        return Vec::new();
                    } else {
                        return vec![Item::Union(union_def)];
                    }
                } else {
                    // 如果没有属性，返回空的枚举
                    return vec![Item::Union(self.determine_union_type(comp, &[]))];
                }
            }
            else if unit_type.to_lowercase() == "enum" {
                // 从组件属性中提取属性列表
                if let PropertyClause::Properties(props) = &comp.properties {
                    return vec![Item::Enum(self.determine_enum_type(comp, props))];
                } else {
                    // 如果没有属性，返回空的枚举
                    return vec![Item::Enum(self.determine_enum_type(comp, &[]))];
                }
            }
        }
        
        // 只有当组件标识符不存在于type_mappings中时，才添加到type_mappings中
        if !self.type_mappings.contains_key(&comp.identifier.to_lowercase()) {
            self.type_mappings.insert(comp.identifier.to_lowercase(), target_type.clone());
        }
        
        vec![Item::TypeAlias(TypeAlias {
            name: comp.identifier.clone(),
            target: target_type,
            vis: Visibility::Public,
            docs: vec![format!("// AADL Data Type: {}", comp.identifier.clone())],
        })]
    }

    fn determine_data_type(&self, comp: &ComponentType) -> Type {
        // 首先检查组件标识符是否已经存在于type_mappings中
        if let Some(existing_type) = self.type_mappings.get(&comp.identifier.to_lowercase()) {
            return existing_type.clone();
        }
        
        // 如果没有找到，则处理复杂类型
        self.determine_complex_data_type(comp)
    }

    /// 处理复杂数据类型，包括数组、结构体、联合体、枚举等
    fn determine_complex_data_type(&self, comp: &ComponentType) -> Type {
        if let PropertyClause::Properties(props) = &comp.properties {
            for prop in props {
                if let Property::BasicProperty(bp) = prop {
                    // 处理 Data_Model::Data_Representation 属性
                    println!("bp: {:?}", bp);
                    
                    // 检查属性集是否为 "Data_Model" 且属性名为 "Data_Representation"
                    if let Some(property_set) = &bp.identifier.property_set {
                        if property_set.to_lowercase() == "data_model" 
                           && bp.identifier.name.to_lowercase() == "data_representation" {
                            if let PropertyValue::Single(PropertyExpression::String(
                                StringTerm::Literal(str_val),
                            )) = &bp.value
                            {
                                println!("str_val: {:?}", str_val);
                                match str_val.to_lowercase().as_str() {
                                    "array" => {
                                        return self.determine_array_type(comp, props);
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
                                    _ => {
                                        // 使用 type_mappings 查找对应的类型，如果没有找到则使用原值
                                        return self
                                            .type_mappings
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
                        if let PropertyValue::Single(PropertyExpression::String(
                            StringTerm::Literal(str_val),
                        )) = &bp.value
                        {
                            return self
                                .type_mappings
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
    fn determine_array_type(&self, comp: &ComponentType, props: &[Property]) -> Type {
        let mut base_type = Type::Named("i32".to_string()); // 默认基础类型
        let mut dimensions = Vec::new();
        
        // 查找 Base_Type 属性
        for prop in props {
            println!("prop: {:?}", prop);
            if let Property::BasicProperty(bp) = prop {
                if bp.identifier.name.to_lowercase() == "base_type" {
                    if let PropertyValue::Single(PropertyExpression::String(
                        StringTerm::Literal(type_name),
                    )) = &bp.value
                    {
                        base_type = self
                            .type_mappings
                            .get(&type_name.to_lowercase())
                            .cloned()
                            .unwrap_or_else(|| Type::Named(type_name.clone()));
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
    fn determine_struct_type(&mut self, comp: &ComponentType, props: &[Property]) -> StructDef {
        let mut fields = Vec::new();
        let mut field_names = Vec::new();
        let mut field_types = Vec::new();
        
        // 解析字段类型和字段名
        for prop in props {
            if let Property::BasicProperty(bp) = prop {
                // 解析 Base_Type 属性获取字段类型
                if let Some(property_set) = &bp.identifier.property_set {
                    if property_set.to_lowercase() == "data_model" 
                       && bp.identifier.name.to_lowercase() == "base_type" {
                        if let PropertyValue::List(type_list) = &bp.value {
                            for type_item in type_list {
                                if let PropertyListElement::Value(PropertyExpression::ComponentClassifier(
                                    ComponentClassifierTerm { unique_component_classifier_reference }
                                )) = type_item {
                                    // 从分类器引用中提取类型名
                                    let type_name = match unique_component_classifier_reference {
                                        UniqueComponentClassifierReference::Type(impl_ref) => {
                                            impl_ref.implementation_name.type_identifier.clone()
                                        }
                                        UniqueComponentClassifierReference::Implementation(impl_ref) => {
                                            impl_ref.implementation_name.type_identifier.clone()
                                        }
                                    };
                                    
                                    // 映射到 Rust 类型
                                    let rust_type = self.type_mappings
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
                       && bp.identifier.name.to_lowercase() == "element_names" {
                        if let PropertyValue::List(name_list) = &bp.value {
                            for name_item in name_list {
                                if let PropertyListElement::Value(PropertyExpression::String(
                                    StringTerm::Literal(name)
                                )) = name_item {
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
            self.data_comp_type.insert(comp.identifier.clone(), "struct".to_string());
        }
        // 创建字段
        for (name, ty) in field_names.iter().zip(field_types.iter()) {
            fields.push(Field {
                name: name.clone(),
                ty: ty.clone(),
                docs: vec![],
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

    /// 处理联合体类型,使用枚举类型来表示，不使用union类型,避免unsafe
    fn determine_union_type(&mut self, comp: &ComponentType, props: &[Property]) -> UnionDef {
        // 解析字段类型和字段名
        let mut field_names = Vec::new();
        let mut field_types = Vec::new();
        
        for prop in props {
            if let Property::BasicProperty(bp) = prop {
                // 解析 Base_Type 属性获取字段类型
                if let Some(property_set) = &bp.identifier.property_set {
                    if property_set.to_lowercase() == "data_model" 
                       && bp.identifier.name.to_lowercase() == "base_type" {
                        if let PropertyValue::List(type_list) = &bp.value {
                            for type_item in type_list {
                                if let PropertyListElement::Value(PropertyExpression::ComponentClassifier(
                                    ComponentClassifierTerm { unique_component_classifier_reference }
                                )) = type_item {
                                    // 从分类器引用中提取类型名
                                    let type_name = match unique_component_classifier_reference {
                                        UniqueComponentClassifierReference::Type(impl_ref) => {
                                            impl_ref.implementation_name.type_identifier.clone()
                                        }
                                        UniqueComponentClassifierReference::Implementation(impl_ref) => {
                                            impl_ref.implementation_name.type_identifier.clone()
                                        }
                                    };
                                    
                                    // 映射到 Rust 类型
                                    let rust_type = self.type_mappings
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
                       && bp.identifier.name.to_lowercase() == "element_names" {
                        if let PropertyValue::List(name_list) = &bp.value {
                            for name_item in name_list {
                                if let PropertyListElement::Value(PropertyExpression::String(
                                    StringTerm::Literal(name)
                                )) = name_item {
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
            self.data_comp_type.insert(comp.identifier.clone(), "union".to_string());
        }
        
        // 创建联合体字段
        let mut fields = Vec::new();
        for (name, ty) in field_names.iter().zip(field_types.iter()) {
            fields.push(Field {
                name: name.clone(),
                ty: ty.clone(),
                docs: vec![],
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
    fn determine_enum_type(&self, comp: &ComponentType, props: &[Property]) -> EnumDef {
        // 解析枚举值名称
        let mut variant_names = Vec::new();
        
        for prop in props {
            if let Property::BasicProperty(bp) = prop {
                // 解析 Enumerators 属性获取枚举值名称
                if let Some(property_set) = &bp.identifier.property_set {
                    if property_set.to_lowercase() == "data_model" 
                       && bp.identifier.name.to_lowercase() == "enumerators" {
                        if let PropertyValue::List(name_list) = &bp.value {
                            for name_item in name_list {
                                if let PropertyListElement::Value(PropertyExpression::String(
                                    StringTerm::Literal(name)
                                )) = name_item {
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

    fn convert_thread_component(&self, comp: &ComponentType) -> Vec<Item> {
        let mut items = Vec::new();

        // 1. 结构体定义
        let mut fields = self.convert_type_features(&comp.features); //特征列表
        // 添加 CPU ID 字段
        fields.push(Field {
            name: "cpu_id".to_string(),
            ty: Type::Named("isize".to_string()),
            docs: vec!["// 结构体新增 CPU ID".to_string()],
            attrs: Vec::new(),
        });
        
        let struct_def = StructDef {
            name: format!("{}Thread", comp.identifier.to_lowercase()),
            fields, //特征列表
            properties: self.convert_properties(ComponentRef::Type(&comp)), // 属性列表
            generics: Vec::new(),
            derives: vec!["Debug".to_string()],
            docs: self.create_component_type_docs(comp),
            vis: Visibility::Public, //默认public
        };
        items.push(Item::Struct(struct_def));
        // 2. 实现块
        // if let Some(impl_block) = self.create_threadtype_impl(comp) {
        //     items.push(Item::Impl(impl_block));
        // }

        items
    }

    fn convert_process_component(&self, comp: &ComponentType) -> Vec<Item> {
        let mut items = Vec::new();

        // 1. 结构体定义
        let mut fields = self.convert_type_features(&comp.features); //特征列表
        // 添加 CPU ID 字段
        fields.push(Field {
            name: "cpu_id".to_string(),
            ty: Type::Named("isize".to_string()),
            docs: vec!["// 进程 CPU ID".to_string()],
            attrs: Vec::new(),
        });
        
        let struct_def = StructDef {
            name: format!("{}Process", comp.identifier.to_lowercase()),
            fields, //特征列表
            properties: self.convert_properties(ComponentRef::Type(&comp)), // 属性列表
            generics: Vec::new(),
            derives: vec!["Debug".to_string()],
            docs: self.create_component_type_docs(comp),
            vis: Visibility::Public, //默认public
        };
        items.push(Item::Struct(struct_def));

        items
    }

    fn convert_system_component(&self, comp: &ComponentType) -> Vec<Item> {
        let mut items = Vec::new();

        // 1. 结构体定义 - 系统类型不包含任何字段，因为字段在实现中定义
        let struct_def = StructDef {
            name: format!("{}System", comp.identifier.to_lowercase()),
            fields: vec![], // 系统类型不包含字段
            properties: self.convert_properties(ComponentRef::Type(&comp)),
            generics: Vec::new(),
            derives: vec!["Debug".to_string()],
            docs: vec![format!("// AADL System: {}", comp.identifier)],
            vis: Visibility::Public,
        };
        items.push(Item::Struct(struct_def));

        items
    }

    fn create_system_impl_block(&mut self, impl_: &ComponentImplementation) -> ImplBlock {
        let mut items = Vec::new();

        // 添加new方法
        items.push(ImplItem::Method(FunctionDef {
            name: "new".to_string(),
            params: vec![],
            return_type: Type::Named("Self".to_string()),
            body: self.create_system_new_body(impl_),
            asyncness: false,
            vis: Visibility::Public,
            docs: vec!["// Creates a new system instance".to_string()],
            attrs: Vec::new(),
        }));

        // 添加run方法
        items.push(ImplItem::Method(FunctionDef {
            name: "run".to_string(),
            params: vec![Param {
                name: "self".to_string(),
                ty: Type::Named("Self".to_string()),
            }],
            return_type: Type::Unit,
            body: self.create_system_run_body(impl_),
            asyncness: false,
            vis: Visibility::Public,
            docs: vec!["// Runs the system, starts all processes".to_string()],
            attrs: Vec::new(),
        }));

        ImplBlock {
            target: Type::Named(format!("{}System", impl_.name.type_identifier.to_lowercase())),
            generics: Vec::new(),
            items,
            trait_impl: None,
        }
    }

    fn convert_type_features(&self, features: &FeatureClause) -> Vec<Field> {
        let mut fields = Vec::new();

        if let FeatureClause::Items(feature_items) = features {
            for feature in feature_items {
                match feature {
                    Feature::Port(port) => {
                        fields.push(Field {
                            name: port.identifier.to_lowercase(),
                            ty: self.convert_port_type(&port),
                            docs: vec![format!("// Port: {} {:?}", port.identifier, port.direction)],
                            attrs: Vec::new(),
                        });
                    }
                    Feature::SubcomponentAccess(sub_access) => {
                        // 处理 requires data access 特征
                        if let SubcomponentAccessSpec::Data(data_access) = sub_access {
                            if data_access.direction == AccessDirection::Requires {
                                // 生成字段：pub GNC_POS : PosShared,
                                let field_name = data_access.identifier.to_lowercase();
                                
                                // 从分类器中提取组件名称，用于生成PosShared类型
                                if let Some(classifier) = &data_access.classifier {
                                    if let DataAccessReference::Classifier(unique_ref) = classifier {
                                        let shared_type_name = match unique_ref {
                                            UniqueComponentClassifierReference::Implementation(impl_ref) => {
                                                // 从 POS.Impl 生成 pos_shared
                                                let base_name = &impl_ref.implementation_name.type_identifier;
                                                if base_name.ends_with(".Impl") {
                                                    let prefix = &base_name[..base_name.len() - 5]; // 去掉".Impl"后缀
                                                    format!("{}Shared", prefix)
                                                } else {
                                                    // 如果没有Impl后缀，直接处理
                                                    format!("{}Shared", base_name)
                                                }
                                            }
                                            UniqueComponentClassifierReference::Type(type_ref) => {
                                                // 从 POS 生成 pos_shared
                                                let base_name = &type_ref.implementation_name.type_identifier;
                                                format!("{}Shared", base_name)
                                            }
                                        };
                                        
                                        fields.push(Field {
                                            name: field_name,
                                            ty: Type::Named(shared_type_name),
                                            docs: vec![format!("// AADL feature: {} : requires data access {}", 
                                                data_access.identifier, 
                                                match classifier {
                                                    DataAccessReference::Classifier(UniqueComponentClassifierReference::Implementation(impl_ref)) => 
                                                        impl_ref.implementation_name.type_identifier.clone(),
                                                    DataAccessReference::Classifier(UniqueComponentClassifierReference::Type(type_ref)) => 
                                                        type_ref.implementation_name.type_identifier.clone(),
                                                    _ => "Unknown".to_string(),
                                                }
                                            )],
                                            attrs: Vec::new(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        fields
    }

    fn convert_port_type(&self, port: &PortSpec) -> Type {
        // 确定通道类型（Sender/Receiver）
        let channel_type = match port.direction {
            PortDirection::In => "mpsc::Receiver",
            PortDirection::Out => "mpsc::Sender",
            PortDirection::InOut => "mpsc::Sender", //TODO:std::mpsc不支持双向通道，暂时这样写
        };

        // 确定内部数据类型
        let inner_type = match &port.port_type {
            PortType::Data { classifier } | PortType::EventData { classifier } => {
                classifier
                    .as_ref() //.as_ref() 的作用是把 Option<T> 变成 Option<&T>。它不会取得其中值的所有权，而只是"借用"里面的值。
                    .map(|c: &PortDataTypeReference| self.classifier_to_type(c)) //对 Option 类型调用 .map() 方法，用于在 Some(...) 中包裹的值c上应用一个函数。
                    .unwrap_or(Type::Named("()".to_string()))
            }
            PortType::Event => Type::Named("()".to_string()), // 事件端口固定使用单元类型
        };

        // 组合成最终类型
        //Type::Generic(channel_type.to_string(), vec![inner_type])
        Type::Generic(
            "Option".to_string(),
            vec![Type::Generic(channel_type.to_string(), vec![inner_type])],
        )
    }

    fn classifier_to_type(&self, classifier: &PortDataTypeReference) -> Type {
        match classifier {
            PortDataTypeReference::Classifier(UniqueComponentClassifierReference::Type(
                ref type_ref,
            )) => {
                // 优先查找我们所自定义类型映射规则
                self.type_mappings
                    .get(&type_ref.implementation_name.type_identifier.to_lowercase())
                    .cloned()
                    .unwrap_or_else(|| {
                        Type::Named(type_ref.implementation_name.type_identifier.clone())
                    })
            }
            _ => Type::Named("()".to_string()),
        }
    }

    // 转换AADL属性为Property列表
    fn convert_properties(&self, comp: ComponentRef<'_>) -> Vec<StruProperty> {
        let mut result = Vec::new();

        // 通过模式匹配获取属性
        let properties = match comp {
            ComponentRef::Type(component_type) => &component_type.properties,
            ComponentRef::Impl(component_impl) => &component_impl.properties,
        };

        // 原有处理逻辑
        if let PropertyClause::Properties(props) = properties {
            for prop in props {
                if let Some(converted) = self.convert_single_property(prop) {
                    result.push(converted);
                }
            }
        }

        result
        // properties
    }
    // 转换单个属性
    fn convert_single_property(&self, prop: &Property) -> Option<StruProperty> {
        let Property::BasicProperty(bp) = prop else {
            return None; // 跳过非基础属性
        };

        let docs = vec![format!("// AADL属性: {}", bp.identifier.name)];

        Some(StruProperty {
            name: bp.identifier.name.clone(),
            value: self.parse_property_value(&bp.value)?,
            docs,
        })
    }

    // 解析AADL属性值到Rust类型
    fn parse_property_value(&self, value: &PropertyValue) -> Option<StruPropertyValue> {
        match value {
            PropertyValue::Single(expr) => self.parse_property_expression(expr),
            _ => None, // 忽略其他复杂属性
        }
    }

    // 解析属性表达式为StruPropertyValue
    fn parse_property_expression(&self, expr: &PropertyExpression) -> Option<StruPropertyValue> {
        match expr {
            // 基础类型处理
            PropertyExpression::Boolean(boolean_term) => self.parse_boolean_term(boolean_term),
            PropertyExpression::Real(real_term) => self.parse_real_term(real_term),
            PropertyExpression::Integer(integer_term) => self.parse_integer_term(integer_term),
            PropertyExpression::String(string_term) => self.parse_string_term(string_term),

            // 范围类型处理
            PropertyExpression::IntegerRange(range_term) => Some(StruPropertyValue::Range(
                range_term.lower.value.parse().ok()?,
                range_term.upper.value.parse().ok()?,
                range_term.lower.unit.clone(),
            )),

            // 其他复杂类型暂不处理
            _ => None,
        }
    }

    // 布尔项解析
    fn parse_boolean_term(&self, term: &BooleanTerm) -> Option<StruPropertyValue> {
        match term {
            BooleanTerm::Literal(b) => Some(StruPropertyValue::Boolean(*b)),
            BooleanTerm::Constant(_) => None, // 常量需要查表解析，此处简化
        }
    }

    // 实数项解析
    fn parse_real_term(&self, term: &SignedRealOrConstant) -> Option<StruPropertyValue> {
        match term {
            SignedRealOrConstant::Real(signed_real) => {
                let value = signed_real.sign.as_ref().map_or(1.0, |s| match s {
                    Sign::Plus => 1.0,
                    Sign::Minus => -1.0,
                }) * signed_real.value;
                Some(StruPropertyValue::Float(value))
            }
            SignedRealOrConstant::Constant { .. } => None, // TODO:常量需要查表
        }
    }

    // 整数项解析
    fn parse_integer_term(&self, term: &SignedIntergerOrConstant) -> Option<StruPropertyValue> {
        match term {
            SignedIntergerOrConstant::Real(signed_int) => {
                let value = signed_int.sign.as_ref().map_or(1, |s| match s {
                    Sign::Plus => 1,
                    Sign::Minus => -1,
                }) * signed_int.value;
                Some(StruPropertyValue::Integer(value))
            }
            SignedIntergerOrConstant::Constant { .. } => None, // 常量需要查表
        }
    }

    // 字符串项解析
    fn parse_string_term(&self, term: &StringTerm) -> Option<StruPropertyValue> {
        match term {
            StringTerm::Literal(s) => Some(StruPropertyValue::String(s.clone())),
            StringTerm::Constant(_) => None, // 常量需要查表
        }
    }


    fn extract_period(&self, comp: &ComponentType) -> Option<u64> {
        if let PropertyClause::Properties(props) = &comp.properties {
            for prop in props {
                if let Property::BasicProperty(bp) = prop {
                    if bp.identifier.name.to_lowercase() == "period" {
                        if let PropertyValue::Single(PropertyExpression::Integer(
                            SignedIntergerOrConstant::Real(int_val),
                        )) = &bp.value
                        {
                            return Some(int_val.value as u64);
                        }
                    }
                }
            }
        }
        None
    }



    fn convert_subprogram(&self, comp: &ComponentType, package: &Package) -> Vec<Item> {
        let mut items = Vec::new();

        // 检查是否是C语言绑定的子程序
        if let Some(c_func_name) = self.extract_c_function_name(comp) {
            return self.generate_c_function_wrapper(comp, &c_func_name, package);
        }

        if let FeatureClause::Items(features) = &comp.features {
            for feature in features {
                if let Feature::Port(port) = feature {
                    items.push(Item::Function(FunctionDef {
                        name: format!("handle_{}", port.identifier),
                        params: vec![Param {
                            name: "port".to_string(),
                            ty: self.convert_port_type(&port),
                        }],
                        return_type: Type::Unit,
                        body: Block {
                            stmts: vec![Statement::Expr(Expr::Ident(format!(
                                "// Handle port: {}",
                                port.identifier
                            )))],
                            expr: None,
                        },
                        asyncness: matches!(
                            port.port_type,
                            PortType::Event | PortType::EventData { .. }
                        ),
                        vis: Visibility::Public,
                        docs: vec![
                            format!("// Port handler for {}", port.identifier),
                            format!("// Direction: {:?}", port.direction),
                        ],
                        attrs: Vec::new(),
                    }));
                }
            }
        }

        items
    }

    fn extract_c_function_name(&self, comp: &ComponentType) -> Option<String> {
        if let PropertyClause::Properties(props) = &comp.properties {
            for prop in props {
                if let Property::BasicProperty(bp) = prop {
                    if bp.identifier.name.to_lowercase() == "source_name" {
                        if let PropertyValue::Single(PropertyExpression::String(
                            StringTerm::Literal(name),
                        )) = &bp.value
                        {
                            return Some(name.clone());
                        }
                    }
                }
            }
        }
        None
    }

    fn generate_c_function_wrapper(&self, comp: &ComponentType, c_func_name: &str, package: &Package) -> Vec<Item> {
        //获取C程序源文件文件名
        let source_files = self.extract_source_files(comp);

        let mut items = Vec::new();
        let mut functions = Vec::new();
        let mut types_to_import = std::collections::HashSet::new();

        // 处理每个特征
        if let FeatureClause::Items(features) = &comp.features {
            for feature in features {
                match feature {
                    Feature::Port(port) => {
                        let (func_name, param_type) = match port.direction {
                            PortDirection::Out => (
                                "send",
                                Type::Reference(Box::new(self.convert_paramport_type(port)), true, true),
                            ),
                            PortDirection::In => (
                                "receive",
                                Type::Reference(Box::new(self.convert_paramport_type(port)), false, false),
                            ),
                            _ => continue, //
                        };

                        // 收集需要导入的类型
                        if let Type::Named(type_name) = &self.convert_paramport_type(port) {
                            if !self.is_rust_primitive_type(type_name) {
                                types_to_import.insert(type_name.clone());
                            }
                        }

                        // 创建包装函数
                        functions.push(FunctionDef {
                            name: func_name.to_string(),
                            params: vec![Param {
                                name: port.identifier.to_string().to_lowercase(),
                                ty: param_type,
                            }],
                            return_type: Type::Unit,
                            body: Block {
                                stmts: vec![Statement::Expr(Expr::Unsafe(Box::new(Block {
                                    stmts: vec![Statement::Expr(Expr::Call(
                                        Box::new(Expr::Path(
                                            vec![c_func_name.to_string()],
                                            PathType::Namespace,
                                        )),
                                        vec![Expr::Ident(port.identifier.to_string().to_lowercase())],
                                    ))],
                                    expr: None,
                                })))],
                                expr: None,
                            },
                            asyncness: false,
                            vis: Visibility::Public,
                            docs: vec![
                                format!("// Wrapper for C function {}", c_func_name),
                                format!("// Original AADL port: {}", port.identifier),
                            ],
                            attrs: Vec::new(),
                        });
                    }
                    Feature::SubcomponentAccess(sub_access) => {
                        // 处理 requires data access 特征
                        if let SubcomponentAccessSpec::Data(data_access) = sub_access {
                            if data_access.direction == AccessDirection::Requires {
                                // 从 this : requires data access POS.Impl 中提取 POS.Impl
                                if let Some(classifier) = &data_access.classifier {
                                    if let DataAccessReference::Classifier(unique_ref) = classifier {
                                        if let UniqueComponentClassifierReference::Implementation(impl_ref) = unique_ref {
                                            let data_component_name = &impl_ref.implementation_name.type_identifier;
                                            // 查找该数据组件实现中的具体数据类型
                                            if let Some(data_type) = self.find_data_type_from_implementation(data_component_name, package) {
                                                // 将数据类型添加到导入列表中
                                                let data_type_for_import = data_type.clone();
                                                types_to_import.insert(data_type_for_import);
                                                
                                                // 为 requires data access 特征生成 call 函数
                                                let call_function = FunctionDef {
                                                    name: "call".to_string(),
                                                    params: vec![Param {
                                                        name: "pos_ref".to_string(),
                                                        ty: Type::Reference(Box::new(Type::Named(data_type)), true, true), // &mut PosInternalType
                                                    }],
                                                    return_type: Type::Unit,
                                                    body: Block {
                                                        stmts: vec![Statement::Expr(Expr::Unsafe(Box::new(Block {
                                                            stmts: vec![Statement::Expr(Expr::Call(
                                                                Box::new(Expr::Path(
                                                                    vec![c_func_name.to_string()],
                                                                    PathType::Namespace,
                                                                )),
                                                                vec![Expr::Ident("pos_ref".to_string())], // 直接传递引用，让Rust编译器处理类型转换
                                                            ))],
                                                            expr: None,
                                                        })))],
                                                        expr: None,
                                                    },
                                                    asyncness: false,
                                                    vis: Visibility::Public,
                                                    docs: vec![
                                                        format!("// Call C function {} with data access reference", c_func_name),
                                                        "// Generated for requires data access feature".to_string(),
                                                        "// Note: Rust compiler will handle the reference to pointer conversion".to_string(),
                                                    ],
                                                    attrs: Vec::new(),
                                                };
                                                
                                                functions.push(call_function);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {} // 忽略其他类型的特征
                }
            }
        }


        // 如果没有通信端口，创建直接调用C函数的包装器
        if functions.is_empty() {
            functions.push(FunctionDef {
                name: "execute".to_string(),
                params: Vec::new(),
                return_type: Type::Unit,
                body: Block {
                    stmts: vec![Statement::Expr(Expr::Unsafe(Box::new(Block {
                        stmts: vec![Statement::Expr(Expr::Call(
                            Box::new(Expr::Path(
                                vec![c_func_name.to_string()],
                                PathType::Namespace,
                            )),
                            Vec::new(),
                        ))],
                        expr: None,
                    })))],
                    expr: None,
                },
                asyncness: false,
                vis: Visibility::Public,
                docs: vec![
                    format!("// Direct execution wrapper for C function {}", c_func_name),
                    "// This component has no communication ports".to_string(),
                ],
                attrs: Vec::new(),
            });
        }
        // 创建模块
        //if !functions.is_empty() 

        {
            let mut docs = vec![
                format!(
                    "// Auto-generated from AADL subprogram: {}",
                    comp.identifier
                ),
                format!("// C binding to: {}", c_func_name),
            ];
            //在注释中添加C程序源文件文件名
            if !source_files.is_empty() {
                docs.push(format!("// source_files: {}", source_files.join(", ")));
            }

            // 构建use语句
            let mut imports = vec![c_func_name.to_string()];
            imports.extend(types_to_import.into_iter());

            let use_stmt = Item::Use(UseStatement {
                path: vec!["super".to_string()],
                kind: UseKind::Nested(imports),
            });

            // 构建模块内容：先添加use语句，再添加函数
            let mut module_items = vec![use_stmt];
            module_items.extend(functions.into_iter().map(Item::Function));

            let module = RustModule {
                name: comp.identifier.to_lowercase(),
                docs: docs,
                //items: functions.into_iter().map(Item::Function).collect(),
                items: module_items,
                attrs: Default::default(),
                vis: Visibility::Public,
            };
            items.push(Item::Mod(Box::new(module)));
        }

        items
    }

    fn extract_source_files(&self, comp: &ComponentType) -> Vec<String> {
        let mut source_files = Vec::new();

        if let PropertyClause::Properties(props) = &comp.properties {
            for prop in props {
                if let Property::BasicProperty(bp) = prop {
                    if bp.identifier.name.to_lowercase() == "source_text" {
                        match &bp.value {
                            PropertyValue::Single(PropertyExpression::String(
                                StringTerm::Literal(text),
                            )) => {
                                source_files.push(text.clone());
                            }
                            PropertyValue::List(arraylist) => {
                                for item in arraylist {
                                    if let PropertyListElement::Value(PropertyExpression::String(
                                        StringTerm::Literal(text),
                                    )) = item {
                                        source_files.push(text.clone());
                                    }
                                }
                            }
                            _ => {println!("error in extract_source_files");}
                        }
                    }
                }
            }
        }

        source_files
    }

    //TODO:这是由于subprogram的feature中的参数连接，暂时还是使用端口连接（在aadl_ast中未定义参数连接方式），这里写死参数链接的类型
    fn convert_paramport_type(&self, port: &PortSpec) -> Type {
        // 直接提取分类器类型，不加任何包装
        match &port.port_type {
            PortType::Data { classifier } | PortType::EventData { classifier } => {
                classifier
                    .as_ref()
                    .map(|c| self.classifier_to_type(c))
                    .unwrap_or_else(|| {
                        // 默认类型处理，可以根据需要调整
                        match port.direction {
                            PortDirection::Out => Type::Named("i32".to_string()),
                            _ => Type::Named("()".to_string()),
                        }
                    })
            }
            PortType::Event => Type::Named("()".to_string()),
            // 其他类型不需要处理，因为此函数仅在参数连接时调用
        }
    }

    // 辅助函数：判断是否为Rust原生类型
    fn is_rust_primitive_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
                | "bool"
                | "char"
                | "str"
                | "String"
        )
    }



    fn convert_implementation(&mut self, impl_: &ComponentImplementation) -> Vec<Item> {
        match impl_.category {
            ComponentCategory::Process => self.convert_process_implementation(impl_),
            ComponentCategory::Thread => self.convert_thread_implemenation(impl_),
            ComponentCategory::System => self.convert_system_implementation(impl_),
            ComponentCategory::Data => self.convert_data_implementation(impl_),
            ComponentCategory::Processor => self.convert_processor_implementation(impl_),
            _ => Vec::default(), // 默认实现
        }
    }

    fn convert_system_implementation(&mut self, impl_: &ComponentImplementation) -> Vec<Item> {
        let mut items = Vec::new();

        // 1. 生成系统结构体
        let fields = self.get_system_fields(impl_); // 获取系统的子组件
        
        let struct_def = StructDef {
            name: format!("{}System", impl_.name.type_identifier.to_lowercase()),
            fields, // 系统的子组件
            properties: Vec::new(), // TODO
            generics: Vec::new(),
            derives: vec!["Debug".to_string()],
            docs: vec![
                format!("// System implementation: {}", impl_.name.type_identifier),
                "// Auto-generated from AADL".to_string(),
            ],
            vis: Visibility::Public,
        };
        items.push(Item::Struct(struct_def));

        // 2. 生成实现块
        items.push(Item::Impl(self.create_system_impl_block(impl_)));

        items
    }

    fn convert_data_implementation(&self, impl_: &ComponentImplementation) -> Vec<Item> {
        let mut items = Vec::new();

        // 检查子组件，判断是否为共享变量/复杂数据类型
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            let subprogram_count = subcomponents.iter()
                .filter(|sub| sub.category == ComponentCategory::Subprogram)
                .count();
            
            let data_subcomponents: Vec<_> = subcomponents.iter()
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
                    let shared_type = Type::Generic("Arc".to_string(), vec![
                        Type::Generic("Mutex".to_string(), vec![
                            Type::Named(data_type_name)
                        ])
                    ]);

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
                    eprintln!("错误：数据组件实现 {} 中有 {} 个数据子组件，暂时不支持多个共享数据", 
                        impl_.name.type_identifier, data_subcomponents.len());
                    eprintln!("请检查AADL模型，确保每个共享数据组件实现中只有一个数据子组件");
                }
            }
            else if self.data_comp_type.contains_key(&impl_.name.type_identifier) {
                //说明是复杂数据类型
                let data_type_name = self.data_comp_type.get(&impl_.name.type_identifier).unwrap();
                if data_type_name == "struct" {
                    items.push(Item::Struct(self.determine_struct_impl(impl_, subcomponents)));
                } else if data_type_name == "union" {
                    items.push(Item::Union(self.determine_union_impl(impl_, subcomponents)));
                }
            }
            
        } 

        items
    }

    /// 处理结构体类型
    fn determine_struct_impl(&self, impl_: &ComponentImplementation, subcomponents: &[Subcomponent]) -> StructDef {
        let mut fields = Vec::new();
        
        // 从子组件中解析字段类型和字段名
        for sub in subcomponents {
            // 获取字段名（子组件标识符）
            let field_name = sub.identifier.clone();
            
            // 获取字段类型
            let field_type = match &sub.classifier {
                SubcomponentClassifier::ClassifierReference(
                    UniqueComponentClassifierReference::Implementation(impl_ref)
                ) => {
                    // 从分类器引用中提取类型名
                    let type_name = impl_ref.implementation_name.type_identifier.clone();
                    
                    // 映射到 Rust 类型
                    self.type_mappings
                        .get(&type_name.to_lowercase())
                        .cloned()
                        .unwrap_or_else(|| Type::Named(type_name))
                }
                SubcomponentClassifier::ClassifierReference(
                    UniqueComponentClassifierReference::Type(type_ref)
                ) => {
                    // 从类型引用中提取类型名
                    let type_name = type_ref.implementation_name.type_identifier.clone();
                    
                    // 映射到 Rust 类型
                    self.type_mappings
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
    fn determine_union_impl(&self, impl_: &ComponentImplementation, subcomponents: &[Subcomponent]) -> UnionDef {
        let mut fields = Vec::new();
        
        // 从子组件中解析字段类型和字段名
        for sub in subcomponents {
            // 获取字段名（子组件标识符）
            let field_name = sub.identifier.clone();
            
            // 获取字段类型
            let field_type = match &sub.classifier {
                SubcomponentClassifier::ClassifierReference(
                    UniqueComponentClassifierReference::Implementation(impl_ref)
                ) => {
                    // 从分类器引用中提取类型名
                    let type_name = impl_ref.implementation_name.type_identifier.clone();
                    
                    // 映射到 Rust 类型
                    self.type_mappings
                        .get(&type_name.to_lowercase())
                        .cloned()
                        .unwrap_or_else(|| Type::Named(type_name))
                }
                SubcomponentClassifier::ClassifierReference(
                    UniqueComponentClassifierReference::Type(type_ref)
                ) => {
                    // 从类型引用中提取类型名
                    let type_name = type_ref.implementation_name.type_identifier.clone();
                    
                    // 映射到 Rust 类型
                    self.type_mappings
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


    fn convert_process_implementation(&mut self, impl_: &ComponentImplementation) -> Vec<Item> {
        let mut items = Vec::new();

        // 1. 生成进程结构体
        let mut fields = self.get_process_fields(impl_); //这里是为了取得进程的子组件
        // 添加 CPU ID 字段
        fields.push(Field {
            name: "cpu_id".to_string(),
            ty: Type::Named("isize".to_string()),
            docs: vec!["// 新增 CPU ID".to_string()],
            attrs: Vec::new(),
        });
        
        let struct_def = StructDef {
            name: format! {"{}Process",impl_.name.type_identifier.to_lowercase()},
            fields, //这里是为了取得进程的子组件
            properties: Vec::new(),                 //TODO
            generics: Vec::new(),
            derives: vec!["Debug".to_string()],
            docs: vec![
                format!("// Process implementation: {}", impl_.name.type_identifier),
                "// Auto-generated from AADL".to_string(),
            ],
            vis: Visibility::Public,
        };
        items.push(Item::Struct(struct_def));

        // 2. 生成实现块
        items.push(Item::Impl(self.create_process_impl_block(impl_)));

        items
    }

    //处理子组件（thread+data）
    fn get_process_fields(&mut self, impl_: &ComponentImplementation) -> Vec<Field> {
        let mut fields = Vec::new();

        // 1. 添加进程的端口字段（对外端口 + 内部端口）
        if let Some(comp_type) = self.get_component_type(impl_) {
            if let FeatureClause::Items(features) = &comp_type.features {
                for feature in features {
                    match feature {
                        Feature::Port(port) => {
                            // 添加对外端口
                            fields.push(Field {
                                name: port.identifier.to_lowercase(),
                                ty: self.convert_port_type(&port),
                                docs: vec![format!("// Port: {} {:?}", port.identifier, port.direction)],
                                attrs: Vec::new(),
                            });
                            
                            // 添加对应的内部端口
                            let internal_port_name = match port.direction {
                                PortDirection::In => format!("{}Send", port.identifier.to_lowercase()),
                                PortDirection::Out => format!("{}Rece", port.identifier.to_lowercase()),
                                PortDirection::InOut => format!("{}Send", port.identifier.to_lowercase()), // InOut 暂时按 In 处理
                            };
                            
                            let internal_port_type = match port.direction {
                                PortDirection::In => {
                                    // 对外是接收端口，内部需要发送端口
                                    match self.convert_port_type(&port) {
                                        Type::Generic(option_name, inner_types) if option_name == "Option" => {
                                            if let Type::Generic(channel_name, channel_args) = &inner_types[0] {
                                                if channel_name == "mpsc::Receiver" {
                                                    // 从 Option<mpsc::Receiver<T>> 转换为 Option<mpsc::Sender<T>>
                                                    Type::Generic("Option".to_string(), vec![Type::Generic("mpsc::Sender".to_string(), channel_args.clone())])
                                                } else {
                                                    Type::Generic("Option".to_string(), vec![Type::Generic(channel_name.clone(), channel_args.clone())])
                                                }
                                            } else {
                                                Type::Generic("Option".to_string(), vec![Type::Generic("mpsc::Sender".to_string(), vec![inner_types[0].clone()])])
                                            }
                                        }
                                        _ => {
                                            // 如果不是 Option 类型，创建 Option<mpsc::Sender<T>>
                                            Type::Generic("Option".to_string(), vec![Type::Generic("mpsc::Sender".to_string(), vec![self.convert_port_type(&port)])])
                                        }
                                    }
                                }
                                PortDirection::Out => {
                                    // 对外是发送端口，内部需要接收端口
                                    match self.convert_port_type(&port) {
                                        Type::Generic(option_name, inner_types) if option_name == "Option" => {
                                            if let Type::Generic(channel_name, channel_args) = &inner_types[0] {
                                                if channel_name == "mpsc::Sender" {
                                                    // 从 Option<mpsc::Sender<T>> 转换为 Option<mpsc::Receiver<T>>
                                                    Type::Generic("Option".to_string(), vec![Type::Generic("mpsc::Receiver".to_string(), channel_args.clone())])
                                                } else {
                                                    Type::Generic("Option".to_string(), vec![Type::Generic(channel_name.clone(), channel_args.clone())])
                                                }
                                            } else {
                                                Type::Generic("Option".to_string(), vec![Type::Generic("mpsc::Receiver".to_string(), vec![inner_types[0].clone()])])
                                            }
                                        }
                                        _ => {
                                            // 如果不是 Option 类型，创建 Option<mpsc::Receiver<T>>
                                            Type::Generic("Option".to_string(), vec![Type::Generic("mpsc::Receiver".to_string(), vec![self.convert_port_type(&port)])])
                                        }
                                    }
                                }
                                PortDirection::InOut => {
                                    // InOut 暂时按 In 处理
                                    match self.convert_port_type(&port) {
                                        Type::Generic(option_name, inner_types) if option_name == "Option" => {
                                            if let Type::Generic(channel_name, channel_args) = &inner_types[0] {
                                                if channel_name == "mpsc::Receiver" {
                                                    // 从 Option<mpsc::Receiver<T>> 转换为 Option<mpsc::Sender<T>>
                                                    Type::Generic("Option".to_string(), vec![Type::Generic("mpsc::Sender".to_string(), channel_args.clone())])
                                                } else {
                                                    Type::Generic("Option".to_string(), vec![Type::Generic(channel_name.clone(), channel_args.clone())])
                                                }
                                            } else {
                                                Type::Generic("Option".to_string(), vec![Type::Generic("mpsc::Sender".to_string(), vec![inner_types[0].clone()])])
                                            }
                                        }
                                        _ => {
                                            // 如果不是 Option 类型，创建 Option<mpsc::Sender<T>>
                                            Type::Generic("Option".to_string(), vec![Type::Generic("mpsc::Sender".to_string(), vec![self.convert_port_type(&port)])])
                                        }
                                    }
                                }
                            };
                            
                            fields.push(Field {
                                name: internal_port_name,
                                ty: internal_port_type,
                                docs: vec![format!("// 内部端口: {} {:?}", port.identifier, port.direction)],
                                attrs: Vec::new(),
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        // 2. 添加子组件字段
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            for sub in subcomponents {
                let type_name = match &sub.classifier {
                    SubcomponentClassifier::ClassifierReference(
                        UniqueComponentClassifierReference::Implementation(unirf),
                    ) => {
                        // 直接使用子组件标识符 + "Thread"
                        format!("{}", unirf.implementation_name.type_identifier)
                    }
                    _ => "UnsupportedComponent".to_string(),
                };

                // 根据类别决定字段类型
                let field_ty = match sub.category {
                    ComponentCategory::Thread => {
                        // 保存线程到进程的绑定关系
                        Type::Named(format!("{}Thread", type_name.to_lowercase()))
                    }
                    ComponentCategory::Data => {
                        // 直接使用原始类型名，不进行大小写转换
                        Type::Named(format!("{}Shared", type_name))
                    }
                    _ => Type::Named(format!("{}Thread", type_name.to_lowercase())),
                };

                let doc = match sub.category {
                    ComponentCategory::Thread => format!("// 子组件线程（{} : thread {}）", sub.identifier, type_name),
                    ComponentCategory::Data => {
                        // 直接使用原始类型名
                        format!("// 共享数据（{} : data {}）", sub.identifier, type_name)
                    }
                    _ => format!("// Subcomponent: {}", sub.identifier),
                };

                fields.push(Field {
                    name: sub.identifier.to_lowercase(),
                    ty: field_ty,
                    docs: vec![doc],
                    attrs: vec![Attribute {
                        name: "allow".to_string(),
                        args: vec![AttributeArg::Ident("dead_code".to_string())],
                    }],
                });
            }
        }

        fields
    }

    fn get_system_fields(&self, impl_: &ComponentImplementation) -> Vec<Field> {
        let mut fields = Vec::new();

        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            for sub in subcomponents {
                // 只处理进程组件
                if matches!(sub.category, ComponentCategory::Process) {
                    let type_name = match &sub.classifier {
                        SubcomponentClassifier::ClassifierReference(
                            UniqueComponentClassifierReference::Implementation(unirf),
                        ) => {
                            // 直接使用子组件标识符
                            format!("{}", unirf.implementation_name.type_identifier)
                        }
                        _ => "UnsupportedComponent".to_string(),
                    };

                    let field_ty = Type::Named(format!("{}Process", type_name.to_lowercase()));
                    let doc = format!("// 子组件进程（{} : process {}）", sub.identifier, type_name);

                    fields.push(Field {
                        name: sub.identifier.to_lowercase(),
                        ty: field_ty,
                        docs: vec![doc],
                        attrs: vec![Attribute {
                            name: "allow".to_string(),
                            args: vec![AttributeArg::Ident("dead_code".to_string())],
                        }],
                    });
                }
            }
        }

        fields
    }

    fn create_process_impl_block(&self, impl_: &ComponentImplementation) -> ImplBlock {
        let mut items = Vec::new();

        // 添加new方法
        items.push(ImplItem::Method(FunctionDef {
            name: "new".to_string(),
            params: vec![Param {
                name: "cpu_id".to_string(),
                ty: Type::Named("isize".to_string()),
            }],
            return_type: Type::Named("Self".to_string()),
            body: self.create_process_new_body(impl_),
            asyncness: false,
            vis: Visibility::Public,
            docs: vec!["// Creates a new process instance".to_string()],
            attrs: Vec::new(),
        }));

        // 添加start方法
        items.push(ImplItem::Method(FunctionDef {
            name: "start".to_string(),
            params: vec![Param {
                name: "self".to_string(),
                ty: Type::Named("Self".to_string()),
            }],
            return_type: Type::Unit,
            body: self.create_process_start_body(impl_),
            asyncness: false,
            vis: Visibility::Public,
            docs: vec!["// Starts all threads in the process".to_string()],
            attrs: Vec::new(),
        }));



        ImplBlock {
            target: Type::Named(format! {"{}Process",impl_.name.type_identifier.to_lowercase()}),
            generics: Vec::new(),
            items,
            trait_impl: None,
        }
    }

    fn create_process_new_body(&self, impl_: &ComponentImplementation) -> Block {
        let mut stmts = Vec::new();
        // 为每个线程收集需要注入到 new() 的共享变量参数（例如 data access 映射）
        let mut thread_extra_args: std::collections::HashMap<String, Vec<Expr>> = std::collections::HashMap::new();

        if let ConnectionClause::Items(connections) = &impl_.connections {
            for conn in connections {
                if let Connection::Access(access_conn) = conn {
                    // 仅处理 data access 映射：ComponentAccess(data) -> SubcomponentAccess{ subcomponent: thread, .. }
                    match (&access_conn.source, &access_conn.destination) {
                        (AccessEndpoint::ComponentAccess(data_name), AccessEndpoint::SubcomponentAccess { subcomponent: thread_name, .. }) => {
                            let thread_key = thread_name.to_lowercase();
                            let data_var = data_name.to_lowercase();
                            let entry = thread_extra_args.entry(thread_key).or_default();
                            // 传递克隆：pos_data.clone()
                            entry.push(Expr::MethodCall(Box::new(Expr::Ident(data_var)), "clone".to_string(), Vec::new()));
                        }
                        // 其他方向暂不处理
                        _ => {}
                    }
                }
            }
        }

        // 1. 创建子组件实例（先 Data 后 Thread，避免线程 new() 引用未声明的共享变量）
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            let mut data_inits: Vec<Statement> = Vec::new();
            let mut thread_inits: Vec<Statement> = Vec::new();

            for sub in subcomponents {
                let type_name = match &sub.classifier {
                    SubcomponentClassifier::ClassifierReference(
                        UniqueComponentClassifierReference::Type(type_ref),
                    ) => type_ref.implementation_name.type_identifier.clone(),
                    SubcomponentClassifier::ClassifierReference(
                        UniqueComponentClassifierReference::Implementation(impl_ref),
                    ) => impl_ref.implementation_name.type_identifier.clone(),
                    SubcomponentClassifier::Prototype(_) => "UnsupportedPrototype".to_string(),
                };

                let var_name = sub.identifier.to_lowercase();
                // 按类别初始化子组件：线程调用 FooThread::new(cpu_id+共享变量克隆)，数据使用 PosShared::default()
                match sub.category {
                    ComponentCategory::Data => {
                        // 直接使用原始类型名，不进行大小写转换
                        let shared_ty = format!("{}Shared", type_name);
                        // let pos: POS.ImplShared = Arc::new(Mutex::new(0));
                        let init_expr = Expr::Call(
                            Box::new(Expr::Path(vec!["Arc".to_string(), "new".to_string()], PathType::Namespace)),
                            vec![Expr::Call(
                                Box::new(Expr::Path(vec!["Mutex".to_string(), "new".to_string()], PathType::Namespace)),
                                vec![Expr::Literal(Literal::Int(0))],
                            )],
                        );
                        data_inits.push(Statement::Let(LetStmt {
                            ifmut: false,
                            name: format!("mut {}", var_name),
                            ty: Some(Type::Named(shared_ty.clone())),
                            init: Some(init_expr),
                        }));
                    }
                    ComponentCategory::Thread => {
                        // 组装 new() 实参：cpu_id + 由 access 连接推导出的共享变量克隆列表
                        let mut args = vec![Expr::Ident("cpu_id".to_string())];
                        if let Some(extra) = thread_extra_args.get(&sub.identifier.to_lowercase()) {
                            args.extend(extra.clone());
                        }
                        thread_inits.push(Statement::Let(LetStmt {
                            ifmut: false,
                            name: format!("mut {}", var_name),
                            ty: Some(Type::Named(format!("{}Thread", type_name.to_lowercase()))),
                            init: Some(Expr::Call(
                                Box::new(Expr::Path(
                                    vec![
                                        format!("{}Thread", type_name.to_lowercase()),
                                        "new".to_string(),
                                    ],
                                    PathType::Namespace,
                                )),
                                args,
                            )),
                        }));
                    }
                    _ => {
                        // 其他类别暂按线程处理
                        thread_inits.push(Statement::Let(LetStmt {
                            ifmut: false,
                            name: format!("mut {}", var_name),
                            ty: Some(Type::Named(format!("{}Thread", type_name.to_lowercase()))),
                            init: Some(Expr::Call(
                                Box::new(Expr::Path(
                                    vec![
                                        format!("{}Thread", type_name.to_lowercase()),
                                        "new".to_string(),
                                    ],
                                    PathType::Namespace,
                                )),
                                vec![Expr::Ident("cpu_id".to_string())],
                            )),
                        }));
                    }
                }
            }

            // 先共享数据，后线程
            stmts.extend(data_inits);
            stmts.extend(thread_inits);
        }

        // 2. 创建内部端口变量
        if let Some(comp_type) = self.get_component_type(impl_) {
            if let FeatureClause::Items(features) = &comp_type.features {
                for feature in features {
                    if let Feature::Port(port) = feature {
                        let internal_port_name = match port.direction {
                            PortDirection::In => format!("{}Send", port.identifier.to_lowercase()),
                            PortDirection::Out => format!("{}Rece", port.identifier.to_lowercase()),
                            PortDirection::InOut => format!("{}Send", port.identifier.to_lowercase()),
                        };
                        
                        // 创建内部端口变量，初始化为None
                        stmts.push(Statement::Let(LetStmt {
                            ifmut: true,
                            name: internal_port_name.clone(),
                            ty: None,
                            init: Some(Expr::Ident("None".to_string())),
                        }));
                    }
                }
            }
        }

        // 3. 建立连接
        if let ConnectionClause::Items(connections) = &impl_.connections {
            for conn in connections {
                if let Connection::Port(port_conn) = conn {
                    stmts.extend(self.create_channel_connection(port_conn));
                }
            }
        }

        // 3. 返回结构体实例
        let mut field_inits = Vec::new();
        
        // 添加端口字段初始化（对外端口初始化为None，内部端口使用变量）
        if let Some(comp_type) = self.get_component_type(impl_) {
            if let FeatureClause::Items(features) = &comp_type.features {
                for feature in features {
                    if let Feature::Port(port) = feature {
                        // 对外端口初始化为None
                        field_inits.push(format!("{}: None", port.identifier.to_lowercase()));
                        
                        // 内部端口使用变量名（将在连接处理中赋值）
                        let internal_port_name = match port.direction {
                            PortDirection::In => format!("{}Send", port.identifier.to_lowercase()),
                            PortDirection::Out => format!("{}Rece", port.identifier.to_lowercase()),
                            PortDirection::InOut => format!("{}Send", port.identifier.to_lowercase()),
                        };
                        
                        field_inits.push(format!("{}", internal_port_name));
                    }
                }
            }
        }
        
        // 添加子组件字段
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            for sub in subcomponents {
                field_inits.push(sub.identifier.to_lowercase());
            }
        }
        
        // 添加cpu_id字段
        field_inits.push("cpu_id".to_string());

        let all_fields = field_inits.join(", ");

        stmts.push(Statement::Expr(Expr::Ident(format!(
            "return Self {{ {} }}  //显式return",
            all_fields
        ))));

        Block { stmts, expr: None }
    }

    fn create_process_start_body(&self, impl_: &ComponentImplementation) -> Block {
        let mut stmts = Vec::new();

        // 1. 解构self，获取所有需要的字段
        let mut destructure_fields = Vec::new();
        let mut thread_fields = Vec::new();
        let mut port_fields = Vec::new();
        
        // 1.1 添加端口字段（来自features）
        if let Some(comp_type) = self.get_component_type(impl_) {
            if let FeatureClause::Items(features) = &comp_type.features {
                for feature in features {
                    if let Feature::Port(port) = feature {
                        // 添加对外端口
                        let port_name = port.identifier.to_lowercase();
                        destructure_fields.push(port_name.clone());
                        port_fields.push(port_name);
                        
                        // 添加内部端口
                        let internal_port_name = match port.direction {
                            PortDirection::In => format!("{}Send", port.identifier.to_lowercase()),
                            PortDirection::Out => format!("{}Rece", port.identifier.to_lowercase()),
                            PortDirection::InOut => format!("{}Send", port.identifier.to_lowercase()),
                        };
                        destructure_fields.push(internal_port_name.clone());
                        port_fields.push(internal_port_name);
                    }
                }
            }
        }
        
        // 1.2 添加子组件字段
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            for sub in subcomponents {
                let var_name = sub.identifier.to_lowercase();
                destructure_fields.push(var_name.clone());
                
                match sub.category {
                    ComponentCategory::Thread => {
                        thread_fields.push(var_name);
                    }
                    ComponentCategory::Data => {
                        // 数据组件可能作为端口使用
                        port_fields.push(var_name);
                    }
                    _ => {}
                }
            }
        }
        
        // 1.3 添加cpu_id字段
        destructure_fields.push("cpu_id".to_string());

        // 创建解构语句：let Self { port1, port1Send, th_c, cpu_id, .. } = self;
        let destructure_stmt = Statement::Let(LetStmt {
            ifmut: false,
            name: format!("Self {{ {}, .. }}", destructure_fields.join(", ")),
            ty: None,
            init: Some(Expr::Ident("self".to_string())),
        });
        stmts.push(destructure_stmt);

        // 2. 启动所有线程子组件（使用解构后的变量）
        for thread_name in thread_fields {
            // 构建线程闭包（使用move语义）
            let closure = Expr::Closure(
                Vec::new(), // 无参数
                Box::new(Expr::MethodCall(
                    Box::new(Expr::Ident(thread_name.clone())),
                    "run".to_string(),
                    Vec::new(),
                )),
            );

            // 构建线程构建器表达式链
            let builder_chain = vec![
                BuilderMethod::Named(format!("\"{}\".to_string()", thread_name)),
                BuilderMethod::Spawn {
                    closure: Box::new(closure),
                    move_kw: false,
                },
            ];

            stmts.push(Statement::Expr(Expr::MethodCall(
                Box::new(Expr::BuilderChain(builder_chain)),
                "unwrap".to_string(),
                Vec::new(),
            )));
        }

        // 3. 启动数据转发循环（使用解构后的变量）
        let forwarding_tasks = self.create_data_forwarding_tasks(impl_);
        for (src_field, dst_field) in forwarding_tasks {
            // 创建接收端变量：let evenementRece_rx = evenementRece.unwrap();
            let rx_var_name = format!("{}_rx", src_field);
            stmts.push(Statement::Let(LetStmt {
                ifmut: false,
                name: rx_var_name.clone(),
                ty: None,
                init: Some(Expr::MethodCall(
                    Box::new(Expr::Ident(src_field.clone())),
                    "unwrap".to_string(),
                    Vec::new(),
                )),
            }));

            // 创建转发线程
            let forwarding_loop = self.create_single_forwarding_thread(&rx_var_name, &dst_field);
            let closure = Expr::Closure(
                Vec::new(),
                Box::new(Expr::Block(Block {
                    stmts: forwarding_loop,
                    expr: None,
                })),
            );

            // 构建线程构建器表达式链
            let builder_chain = vec![
                BuilderMethod::Named(format!("\"data_forwarder_{}\".to_string()", src_field)),
                BuilderMethod::Spawn {
                    closure: Box::new(closure),
                    move_kw: true, // 添加 move 关键字
                },
            ];

            stmts.push(Statement::Expr(Expr::MethodCall(
                Box::new(Expr::BuilderChain(builder_chain)),
                "unwrap".to_string(),
                Vec::new(),
            )));
        }

        Block { stmts, expr: None }
    }

    /// 创建数据转发任务列表
    fn create_data_forwarding_tasks(&self, impl_: &ComponentImplementation) -> Vec<(String, String)> {
        let mut forwarding_tasks = Vec::new();
        
        if let ConnectionClause::Items(connections) = &impl_.connections {
            for conn in connections {
                if let Connection::Port(port_conn) = conn {
                    // 解析源和目标端口
                    let (src_field, dst_field) = match (&port_conn.source, &port_conn.destination) {
                        // 进程端口到子组件端口
                        (
                            PortEndpoint::ComponentPort(src_port),
                            PortEndpoint::SubcomponentPort {
                                subcomponent: dst_comp,
                                port: dst_port,
                            },
                        ) => {
                            // 对于进程端口，应该使用内部端口字段名（如 evenementSend）
                            let src_field = format!("{}", src_port.to_lowercase());
                            let dst_field = format!("{}Send", src_port.to_lowercase());
                            (src_field, dst_field)
                        }
                        // 子组件端口到进程端口
                        (
                            PortEndpoint::SubcomponentPort {
                                subcomponent: src_comp,
                                port: src_port,
                            },
                            PortEndpoint::ComponentPort(dst_port),
                        ) => {
                            let src_field = format!("{}Rece", dst_port.to_lowercase());
                            // 对于进程端口，应该使用内部端口字段名（如 evenementRece）
                            let dst_field = format!("{}", dst_port.to_lowercase());
                            (src_field, dst_field)
                        }
                        _ => continue,
                    };
                    
                    forwarding_tasks.push((src_field, dst_field));
                }
            }
        }
        
        forwarding_tasks
    }

    /// 创建单个转发线程的代码
    fn create_single_forwarding_thread(&self, rx_var_name: &str, dst_field: &str) -> Vec<Statement> {
        let mut stmts = Vec::new();
        
        // 创建转发循环：loop { if let Ok(msg) = rx_var_name.try_recv() { ... } }
        let loop_body = vec![
            Statement::Expr(Expr::IfLet {
                pattern: "Ok(msg)".to_string(),
                value: Box::new(Expr::MethodCall(
                    Box::new(Expr::Ident(rx_var_name.to_string())),
                    "try_recv".to_string(),
                    Vec::new(),
                )),
                then_branch: Block {
                    stmts: vec![
                        Statement::Expr(Expr::IfLet {
                            pattern: "Some(tx)".to_string(),
                            value: Box::new(Expr::Reference(
                                Box::new(Expr::Ident(dst_field.to_string())),
                                true,
                                false,
                            )),
                            then_branch: Block {
                                stmts: vec![
                                    Statement::Let(LetStmt {
                                        ifmut: false,
                                        name: "_".to_string(),
                                        ty: None,
                                        init: Some(Expr::MethodCall(
                                            Box::new(Expr::Ident("tx".to_string())),
                                            "send".to_string(),
                                            vec![Expr::Ident("msg".to_string())],
                                        )),
                                    }),
                                ],
                                expr: None,
                            },
                            else_branch: None,
                        }),
                    ],
                    expr: None,
                },
                else_branch: None,
            }),
            // 添加睡眠以避免CPU占用过高
            Statement::Expr(Expr::MethodCall(
                Box::new(Expr::Path(
                    vec!["std".to_string(), "thread".to_string(), "sleep".to_string()],
                    PathType::Namespace,
                )),
                "".to_string(),
                vec![Expr::MethodCall(
                    Box::new(Expr::Path(
                        vec!["std".to_string(), "time".to_string(), "Duration".to_string(), "from_millis".to_string()],
                        PathType::Namespace,
                    )),
                    "".to_string(),
                    vec![Expr::Literal(Literal::Int(1))],
                )],
            )),
        ];
        
        // 创建无限循环
        stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
            stmts: loop_body,
            expr: None,
        }))));
        
        stmts
    }



    fn create_channel_connection(&self, conn: &PortConnection) -> Vec<Statement> {
        let mut stmts = Vec::new();

        // 这里简化处理，实际应根据连接类型创建适当的channel
        stmts.push(Statement::Let(LetStmt {
            ifmut: false,
            name: "channel".to_string(),
            ty: None, //这里的通道类型由编译器自动推导
            init: Some(Expr::Call(
                Box::new(Expr::Path(
                    vec!["mpsc".to_string(), "channel".to_string()],
                    PathType::Namespace,
                )),
                Vec::new(),
            )),
        }));

        // 处理源端和目标端
        match (&conn.source, &conn.destination) {
            (
                PortEndpoint::SubcomponentPort {
                    subcomponent: src_comp,
                    port: src_port,
                },
                PortEndpoint::SubcomponentPort {
                    subcomponent: dst_comp,
                    port: dst_port,
                },
            ) => {
                // 分配发送端
                stmts.push(Statement::Expr(Expr::MethodCall(
                    Box::new(Expr::Ident(format!(
                        "{}.{}",
                        src_comp.to_lowercase(),
                        src_port.to_lowercase()
                    ))),
                    "send".to_string(), //这个关键字的固定的，例如cnx: port the_sender.p -> the_receiver.p;，前者发送，后者接收
                    //vec![Expr::Ident("channel.0".to_string())],
                    vec![Expr::Call(
                        Box::new(Expr::Path(vec!["Some".to_string()], PathType::Member)),
                        vec![Expr::Ident("channel.0".to_string())],
                    )],
                )));

                // 分配接收端
                stmts.push(Statement::Expr(Expr::MethodCall(
                    Box::new(Expr::Ident(format!(
                        "{}.{}",
                        dst_comp.to_lowercase(),
                        dst_port.to_lowercase()
                    ))),
                    "receive".to_string(),
                    //vec![Expr::Ident("channel.1".to_string())],
                    vec![Expr::Call(
                        Box::new(Expr::Path(vec!["Some".to_string()], PathType::Member)),
                        vec![Expr::Ident("channel.1".to_string())],
                    )],
                )));
            }
            (
                PortEndpoint::ComponentPort(port_name),
                PortEndpoint::SubcomponentPort {
                    subcomponent: dst_comp,
                    port: dst_port,
                },
            ) => {
                // 处理组件端口到子组件端口的连接
                // 根据端口方向确定内部端口名称
                let internal_port_name = match self.get_port_direction(port_name) {
                    PortDirection::In => format!("{}Send", port_name.to_lowercase()),
                    PortDirection::Out => format!("{}Send", port_name.to_lowercase()), // 输出端口生成 Send
                    PortDirection::InOut => format!("{}Send", port_name.to_lowercase()), // InOut 暂时按 In 处理
                };
                
                // 直接赋值给内部端口变量
                stmts.push(Statement::Expr(Expr::BinaryOp(
                    Box::new(Expr::Ident(internal_port_name)),
                    "=".to_string(),
                    Box::new(Expr::Call(
                        Box::new(Expr::Path(vec!["Some".to_string()], PathType::Member)),
                        vec![Expr::Ident("channel.0".to_string())],
                    )),
                )));

                stmts.push(Statement::Expr(Expr::MethodCall(
                    Box::new(Expr::Ident(format!("{}.{}", dst_comp, dst_port))),
                    "receive".to_string(),
                    vec![Expr::Call(
                        Box::new(Expr::Path(vec!["Some".to_string()], PathType::Member)),
                        vec![Expr::Ident("channel.1".to_string())],
                    )],
                )));
            }
            (
                PortEndpoint::SubcomponentPort {
                    subcomponent: src_comp,
                    port: src_port,
                },
                PortEndpoint::ComponentPort(port_name),
            ) => {
                // 处理子组件端口到组件端口的连接（如 th_c.evenement -> evenement）
                // 发送端给线程
                stmts.push(Statement::Expr(Expr::MethodCall(
                    Box::new(Expr::Ident(format!("{}.{}", src_comp, src_port))),
                    "send".to_string(),
                    vec![Expr::Call(
                        Box::new(Expr::Path(vec!["Some".to_string()], PathType::Member)),
                        vec![Expr::Ident("channel.0".to_string())],
                    )],
                )));

                // 接收端给内部端口
                //似乎这种的分配没必要，一定是Rece
                let internal_port_name = match self.get_port_direction(port_name) {
                    PortDirection::In => format!("{}Send", port_name.to_lowercase()),
                    PortDirection::Out => format!("{}Rece", port_name.to_lowercase()), // 输出端口生成 Send
                    PortDirection::InOut => format!("{}Send", port_name.to_lowercase()), // InOut 暂时按 In 处理
                };
                
                // 直接赋值给内部端口变量
                stmts.push(Statement::Expr(Expr::BinaryOp(
                    Box::new(Expr::Ident(internal_port_name)),
                    "=".to_string(),
                    Box::new(Expr::Call(
                        Box::new(Expr::Path(vec!["Some".to_string()], PathType::Member)),
                        vec![Expr::Ident("channel.1".to_string())],
                    )),
                )));
            }
            // 可以继续添加其他端点类型的组合处理
            _ => {
                // 对于不支持的连接类型，生成TODO注释
                stmts.push(Statement::Expr(Expr::Ident(format!(
                    "// TODO: Unsupported connection type: {:?} -> {:?}",
                    conn.source, conn.destination
                ))));
            }
        }

        stmts
    }

    fn create_component_type_docs(&self, comp: &ComponentType) -> Vec<String> {
        let docs = vec![format!(
            "// AADL {:?}: {}",
            comp.category,
            comp.identifier.to_lowercase()
        )];

        docs
    }

    fn create_component_impl_docs(&self, impl_: &ComponentImplementation) -> Vec<String> {
        let docs = vec![format!(
            "// AADL {:?}: {}",
            impl_.category,
            impl_.name.type_identifier.to_lowercase()
        )];

        docs
    }

    fn convert_thread_implemenation(&mut self, impl_: &ComponentImplementation) -> Vec<Item> {
        let mut items = Vec::new();

        // 1. 结构体定义
        let fields = Vec::new(); //对于线程来说是特征列表,thread_impl没有特征
        // 添加 CPU ID 字段
        // fields.push(Field {
        //     name: "cpu_id".to_string(),
        //     ty: Type::Named("isize".to_string()),
        //     docs: vec!["// 新增 CPU ID".to_string()],
        //     attrs: Vec::new(),
        // });
        
        let struct_def = StructDef {
            name: format!("{}Thread", impl_.name.type_identifier.to_lowercase()),
            fields, //对于线程来说是特征列表,thread_impl没有特征
            properties: self.convert_properties(ComponentRef::Impl(&impl_)), // 属性列表
            generics: Vec::new(),
            derives: vec!["Debug".to_string()],
            docs: self.create_component_impl_docs(impl_),
            vis: Visibility::Public, //默认public
        };
        items.push(Item::Struct(struct_def));

        // 2. 实现块（包含run方法）
        let impl_block = ImplBlock {
            target: Type::Named(format!(
                "{}Thread",
                impl_.name.type_identifier.to_lowercase()
            )),
            generics: Vec::new(),
            items: vec![
                // run方法
                ImplItem::Method(FunctionDef {
                    name: "run".to_string(),
                    params: vec![Param {
                        name: "".to_string(),
                        ty: Type::Reference(Box::new(Type::Named("self".to_string())), false, true),
                    }],
                    return_type: Type::Unit,
                    body: self.create_thread_run_body(impl_),
                    asyncness: false,
                    vis: Visibility::Public,
                    docs: vec![
                        "// Thread execution entry point".to_string(),
                        format!(
                            "// Period: {:?} ms",
                            self.extract_property_value(impl_, "period")
                        ),
                    ],
                    attrs: Vec::new(),
                }),
            ],
            trait_impl: None,
        };
        items.push(Item::Impl(impl_block));

        items
    }

    /// 创建线程的 run() 方法体
    /// 该方法生成线程的执行逻辑，包括：
    /// 1. 线程优先级和CPU亲和性设置
    /// 2. 根据调度协议生成不同的执行逻辑
    /// 3. 子程序调用处理（参数端口、共享变量、普通调用）
    fn create_thread_run_body(&mut self, impl_: &ComponentImplementation) -> Block {
        let mut stmts = Vec::new();
        
        //======================= 线程优先级设置 ========================
        // 检查是否有优先级属性
        let priority = self.extract_property_value(impl_, "priority");
        let period = self.extract_property_value(impl_, "period");
        
        // 如果线程有 priority 属性，则设置线程优先级
        if let Some(priority) = priority {
            // 添加优先级设置代码 - 使用unsafe块和完整的错误处理
            stmts.push(Statement::Expr(Expr::Unsafe(Box::new(Block {
                stmts: vec![
                    // let mut param = sched_param { sched_priority: self.priority as i32 };
                    Statement::Let(LetStmt {
                        ifmut: true,
                        name: "param".to_string(),
                        ty: Some(Type::Named("sched_param".to_string())),
                        init: Some(Expr::Ident(format!("sched_param {{ sched_priority: {} }}", priority as i32))),
                    }),
                    // let ret = pthread_setschedparam(pthread_self(), *, &mut param);
                    Statement::Let(LetStmt {
                        ifmut: false,
                        name: "ret".to_string(),
                        ty: None,
                        init: Some(Expr::Call(
                            Box::new(Expr::Path(
                                vec!["pthread_setschedparam".to_string()],
                                PathType::Namespace,
                            )),
                            vec![
                                Expr::Call(
                                    Box::new(Expr::Path(
                                        vec!["pthread_self".to_string()],
                                        PathType::Namespace,
                                    )),
                                    Vec::new(),
                                ),
                                Expr::MethodCall(
                                    Box::new(Expr::MethodCall(
                                        Box::new(Expr::Path(
                                            vec!["*CPU_ID_TO_SCHED_POLICY".to_string()],
                                            PathType::Namespace,
                                        )),
                                        "get".to_string(),
                                        vec![Expr::Reference(
                                            Box::new(Expr::Path(
                                                vec!["self".to_string(), "cpu_id".to_string()],
                                                PathType::Member,
                                            )),
                                            true,
                                            false,
                                        )],
                                    )),
                                    "unwrap_or".to_string(),
                                    vec![Expr::Reference(
                                        Box::new(Expr::Path(
                                            vec!["SCHED_FIFO".to_string()],
                                            PathType::Namespace,
                                        )),
                                        true,
                                        false,
                                    )],
                                ),
                                Expr::Reference(
                                    Box::new(Expr::Ident("param".to_string())),
                                    true,
                                    true,
                                ),
                            ],
                        )),
                    }),
                    // if ret != 0 { eprintln!("..."); }
                    Statement::Expr(Expr::If {
                        condition: Box::new(Expr::BinaryOp(
                            Box::new(Expr::Ident("ret".to_string())),
                            "!=".to_string(),
                            Box::new(Expr::Literal(Literal::Int(0))),
                        )),
                        then_branch: Block {
                            stmts: vec![
                                Statement::Expr(Expr::Call(
                                    Box::new(Expr::Path(
                                        vec!["eprintln!".to_string()],
                                        PathType::Namespace,
                                    )),
                                    vec![
                                        Expr::Literal(Literal::Str(format!("{}Thread: Failed to set thread priority: {{}}", impl_.name.type_identifier.to_lowercase()))),
                                        Expr::Ident("ret".to_string()),
                                    ],
                                )),
                            ],
                            expr: None,
                        },
                        else_branch: None,
                    }),
                ],
                expr: None,
            }))));
        } else if let Some(period) = period {
            // 如果没有优先级但有周期，则根据周期计算优先级(RMS)
            stmts.push(Statement::Expr(Expr::Unsafe(Box::new(Block {
                stmts: vec![
                    // let prio = period_to_priority(self.period as f64);
                    Statement::Let(LetStmt {
                        ifmut: false,
                        name: "prio".to_string(),
                        ty: None,
                        init: Some(Expr::Call(
                            Box::new(Expr::Path(
                                vec!["period_to_priority".to_string()],
                                PathType::Namespace,
                            )),
                            vec![Expr::Ident("self.period as f64".to_string())],
                        )),
                    }),
                    // let mut param: sched_param = sched_param { sched_priority: prio };
                    Statement::Let(LetStmt {
                        ifmut: true,
                        name: "param".to_string(),
                        ty: Some(Type::Named("sched_param".to_string())),
                        init: Some(Expr::Ident("sched_param { sched_priority: prio }".to_string())),
                    }),
                    // let ret = pthread_setschedparam(pthread_self(), *, &mut param);
                    Statement::Let(LetStmt {
                        ifmut: false,
                        name: "ret".to_string(),
                        ty: None,
                        init: Some(Expr::Call(
                            Box::new(Expr::Path(
                                vec!["pthread_setschedparam".to_string()],
                                PathType::Namespace,
                            )),
                            vec![
                                Expr::Call(
                                    Box::new(Expr::Path(
                                        vec!["pthread_self".to_string()],
                                        PathType::Namespace,
                                    )),
                                    Vec::new(),
                                ),
                                Expr::MethodCall(
                                    Box::new(Expr::MethodCall(
                                        Box::new(Expr::Path(
                                            vec!["*CPU_ID_TO_SCHED_POLICY".to_string()],
                                            PathType::Namespace,
                                        )),
                                        "get".to_string(),
                                        vec![Expr::Reference(
                                            Box::new(Expr::Path(
                                                vec!["self".to_string(), "cpu_id".to_string()],
                                                PathType::Member,
                                            )),
                                            true,
                                            false,
                                        )],
                                    )),
                                    "unwrap_or".to_string(),
                                    vec![Expr::Reference(
                                        Box::new(Expr::Path(
                                            vec!["SCHED_FIFO".to_string()],
                                            PathType::Namespace,
                                        )),
                                        true,
                                        false,
                                    )],
                                ),
                                Expr::Reference(
                                    Box::new(Expr::Ident("param".to_string())),
                                    true,
                                    true,
                                ),
                            ],
                        )),
                    }),
                    // if ret != 0 { eprintln!("..."); }
                    Statement::Expr(Expr::If {
                        condition: Box::new(Expr::BinaryOp(
                            Box::new(Expr::Ident("ret".to_string())),
                            "!=".to_string(),
                            Box::new(Expr::Literal(Literal::Int(0))),
                        )),
                        then_branch: Block {
                            stmts: vec![
                                Statement::Expr(Expr::Call(
                                    Box::new(Expr::Path(
                                        vec!["eprintln!".to_string()],
                                        PathType::Namespace,
                                    )),
                                    vec![
                                        Expr::Literal(Literal::Str(format!("{}Thread: Failed to set thread priority from period: {{}}", impl_.name.type_identifier.to_lowercase()))),
                                        Expr::Ident("ret".to_string()),
                                    ],
                                )),
                            ],
                            expr: None,
                        },
                        else_branch: None,
                    }),
                ],
                expr: None,
            }))));
        }

        // ==================== 步骤 0.5: CPU亲和性设置 ====================
        // 如果 cpu_id > -1，则设置线程绑定到指定CPU
        stmts.push(Statement::Expr(Expr::If {
            condition: Box::new(Expr::BinaryOp(
                Box::new(Expr::Path(
                    vec!["self".to_string(), "cpu_id".to_string()],
                    PathType::Member,
                )),
                ">".to_string(),
                Box::new(Expr::Literal(Literal::Int(-1))),
            )),
            then_branch: Block {
                stmts: vec![
                    Statement::Expr(Expr::Call(
                        Box::new(Expr::Path(
                            vec!["set_thread_affinity".to_string()],
                            PathType::Namespace,
                        )),
                        vec![Expr::Path(
                            vec!["self".to_string(), "cpu_id".to_string()],
                            PathType::Member,
                        )],
                    )),
                ],
                expr: None,
            },
            else_branch: None,
        }));

        // ==================== 步骤 1: 获取调度协议 ====================
        let dispatch_protocol = self.extract_dispatch_protocol(impl_);
        
        // ==================== 步骤 2: 根据调度协议生成不同的执行逻辑 ====================
        match dispatch_protocol.as_deref() {
            Some("Periodic") => {
                // 周期性调度：生成周期性执行循环
                stmts.extend(self.create_periodic_execution_logic(impl_));
            }
            Some("Aperiodic") => {
                // 非周期性调度：生成事件驱动执行逻辑
                stmts.extend(self.create_aperiodic_execution_logic(impl_));
            }
            Some("Sporadic") => {
                // 偶发调度：生成偶发执行逻辑
                stmts.extend(self.create_sporadic_execution_logic(impl_));
            }
            Some("Timed") => {
                // 定时调度：生成定时执行逻辑
                stmts.extend(self.create_timed_execution_logic(impl_));
            }
            _ => {
                // 默认使用周期性调度
                stmts.extend(self.create_periodic_execution_logic(impl_));
            }
        }

        Block { stmts, expr: None }
    }

    /// 创建周期性执行逻辑
    fn create_periodic_execution_logic(&mut self, impl_: &ComponentImplementation) -> Vec<Statement> {
        let mut stmts = Vec::new();
        
        // 从AADL属性中提取周期值，默认为2000ms
        let period = self.extract_property_value(impl_, "period").unwrap_or(2000);
        stmts.push(Statement::Let(LetStmt {
            ifmut: false,
            name: "period".to_string(),
            ty: Some(Type::Path(vec![
                "std".to_string(),
                "time".to_string(),
                "Duration".to_string(),
            ])),
            init: Some(Expr::Call(
                Box::new(Expr::Path(
                    vec!["Duration".to_string(), "from_millis".to_string()],
                    PathType::Namespace,
                )),
                vec![Expr::Literal(Literal::Int(period as i64))],
            )),
        }));

        // 检查是否有Behavior Annex
        if let Some(annex_stmts) = self.annex_converter.generate_annex_code(impl_) {
            // 如果有Behavior Annex，使用它
            stmts.extend(annex_stmts);
        } else {
            // 否则使用原来的子程序调用处理代码
            let port_handling_stmts = self.create_subprogram_call_logic(impl_);

            // 生成周期性执行的主循环
            stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
                stmts: vec![
                    // 记录循环开始时间
                    Statement::Let(LetStmt {
                        ifmut: false,
                        name: "start".to_string(),
                        ty: None,
                        init: Some(Expr::Call(
                            Box::new(Expr::Path(
                                vec!["Instant".to_string(), "now".to_string()],
                                PathType::Namespace,
                            )),
                            Vec::new(),
                        )),
                    }),
                    // 执行子程序调用处理块
                    Statement::Expr(Expr::Block(Block {
                        stmts: port_handling_stmts.clone(),
                        expr: None,
                    })),
                    // 计算执行时间
                    Statement::Let(LetStmt {
                        ifmut: false,
                        name: "elapsed".to_string(),
                        ty: None,
                        init: Some(Expr::MethodCall(
                            Box::new(Expr::Ident("start".to_string())),
                            "elapsed".to_string(),
                            Vec::new(),
                        )),
                    }),
                    // 睡眠剩余时间，确保周期性执行
                    Statement::Expr(Expr::MethodCall(
                        Box::new(Expr::Path(
                            vec!["std".to_string(), "thread".to_string(), "sleep".to_string()],
                            PathType::Namespace,
                        )),
                        "".to_string(),
                        vec![Expr::MethodCall(
                            Box::new(Expr::Ident("period".to_string())),
                            "saturating_sub".to_string(),
                            vec![Expr::Ident("elapsed".to_string())],
                        )],
                    )),
                ],
                expr: None,
            }))));
        }

        stmts
    }

    /// 创建非周期性执行逻辑
    fn create_aperiodic_execution_logic(&self, impl_: &ComponentImplementation) -> Vec<Statement> {
        let mut stmts = Vec::new();

        // 获取事件端口信息（事件端口或事件数据端口）
        let event_ports = self.extract_event_ports(impl_);
        
        // 如果没有找到事件端口，则从参数连接中获取接收端口作为备选
        let receive_ports = if !event_ports.is_empty() {
            event_ports
        } else {
            let subprogram_calls = self.extract_subprogram_calls(impl_);
            subprogram_calls.iter()
                .filter(|(_, _, _, is_send, _)| !is_send)
                .map(|(_, _, thread_port_name, _, _)| thread_port_name.clone())
                .collect()
        };

        // 检查是否有需要端口数据的子程序调用
        let subprogram_calls = self.extract_subprogram_calls(impl_);
        let has_receiving_subprograms = subprogram_calls.iter().any(|(_, _, _, is_send, _)| !is_send);

        // 生成偶发执行逻辑 - 事件驱动，等待消息
        stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
            stmts: vec![
                // 检查是否有接收端口，如果有则等待消息
                // 动态获取第一个接收端口
                Statement::Expr(Expr::IfLet {
                    pattern: "Some(receiver)".to_string(),
                    value: Box::new(Expr::Reference(
                        Box::new(Expr::Path(
                            vec!["self".to_string(), 
                                if !receive_ports.is_empty() { 
                                    receive_ports[0].to_lowercase() 
                                } else { 
                                    "error_sink".to_string() 
                                }],
                            PathType::Member,
                        )),
                        true,
                        false,
                    )),
                    then_branch: Block {
                        stmts: vec![
                            // 阻塞等待消息
                            Statement::Expr(Expr::Match {
                                expr: Box::new(Expr::MethodCall(
                                    Box::new(Expr::Ident("receiver".to_string())),
                                    "recv".to_string(),
                                    Vec::new(),
                                )),
                                arms: vec![
                                    // Ok(val) => 处理接收到的消息
                                    MatchArm {
                                        pattern: "Ok(val)".to_string(),
                                        guard: None,
                                        body: Block {
                                            stmts: vec![
                                                // 执行子程序调用处理，传递已读取的数据
                                                Statement::Expr(Expr::Block(Block {
                                                    stmts: self.create_subprogram_call_logic_with_data(impl_, has_receiving_subprograms),
                                                    expr: None,
                                                })),                                               
                                            ],
                                            expr: None,
                                        },
                                    },
                                    // Err(_) => 通道关闭，退出循环
                                    MatchArm {
                                        pattern: "Err(_)".to_string(),
                                        guard: None,
                                        body: Block {
                                            stmts: vec![
                                                Statement::Expr(Expr::Call(
                                                    Box::new(Expr::Path(
                                                        vec!["eprintln!".to_string()],
                                                        PathType::Namespace,
                                                    )),
                                                    vec![Expr::Literal(Literal::Str(format!("{}Thread: channel closed", impl_.name.type_identifier.to_lowercase())))],
                                                )),
                                                Statement::Expr(Expr::Ident("return".to_string())),
                                            ],
                                            expr: None,
                                        },
                                    },
                                ],
                            }),
                        ],
                        expr: None,
                    },
                    else_branch: None,
                }),
            ],
            expr: None,
        }))));

        stmts
    }

    /// 创建偶发执行逻辑
    fn create_sporadic_execution_logic(&self, impl_: &ComponentImplementation) -> Vec<Statement> {
        let mut stmts = Vec::new();
        
        // 从AADL属性中提取最小间隔时间，默认为1000ms
        let min_interval = self.extract_property_value(impl_, "period").unwrap_or(1000);
        stmts.push(Statement::Let(LetStmt {
            ifmut: false,
            name: "min_interarrival".to_string(),
            ty: Some(Type::Path(vec![
                "std".to_string(),
                "time".to_string(),
                "Duration".to_string(),
            ])),
            init: Some(Expr::Call(
                Box::new(Expr::Path(
                    vec!["Duration".to_string(), "from_millis".to_string()],
                    PathType::Namespace,
                )),
                vec![Expr::Literal(Literal::Int(min_interval as i64))],
            )),
        }));

        // 初始化上次调度时间
        stmts.push(Statement::Let(LetStmt {
            ifmut: true,
            name: "last_dispatch".to_string(),
            ty: Some(Type::Path(vec![
                "std".to_string(),
                "time".to_string(),
                "Instant".to_string(),
            ])),
            init: Some(Expr::Call(
                Box::new(Expr::Path(
                    vec!["Instant".to_string(), "now".to_string()],
                    PathType::Namespace,
                )),
                Vec::new(),
            )),
        }));

        // 获取事件端口信息（事件端口或事件数据端口）
        let event_ports = self.extract_event_ports(impl_);
        
        // 如果没有找到事件端口，则从参数连接中获取接收端口作为备选
        let receive_ports = if !event_ports.is_empty() {
            event_ports
        } else {
            let subprogram_calls = self.extract_subprogram_calls(impl_);
            subprogram_calls.iter()
                .filter(|(_, _, _, is_send, _)| !is_send)
                .map(|(_, _, thread_port_name, _, _)| thread_port_name.clone())
                .collect()
        };

        // 检查是否有需要端口数据的子程序调用
        let subprogram_calls = self.extract_subprogram_calls(impl_);
        let has_receiving_subprograms = subprogram_calls.iter().any(|(_, _, _, is_send, _)| !is_send);

        // 生成偶发执行逻辑 - 事件驱动，等待消息
        stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
            stmts: vec![
                // 检查是否有接收端口，如果有则等待消息
                // 动态获取第一个接收端口
                Statement::Expr(Expr::IfLet {
                    pattern: "Some(receiver)".to_string(),
                    value: Box::new(Expr::Reference(
                        Box::new(Expr::Path(
                            vec!["self".to_string(), 
                                if !receive_ports.is_empty() { 
                                    receive_ports[0].to_lowercase() 
                                } else { 
                                    "error_sink".to_string() 
                                }],
                            PathType::Member,
                        )),
                        true,
                        false,
                    )),
                    then_branch: Block {
                        stmts: vec![
                            // 阻塞等待消息
                            Statement::Expr(Expr::Match {
                                expr: Box::new(Expr::MethodCall(
                                    Box::new(Expr::Ident("receiver".to_string())),
                                    "recv".to_string(),
                                    Vec::new(),
                                )),
                                arms: vec![
                                    // Ok(val) => 处理接收到的消息
                                    MatchArm {
                                        pattern: "Ok(val)".to_string(),
                                        guard: None,
                                        body: Block {
                                            stmts: vec![
                                                // 记录当前时间
                                                Statement::Let(LetStmt {
                                                    ifmut: false,
                                                    name: "now".to_string(),
                                                    ty: None,
                                                    init: Some(Expr::Call(
                                                        Box::new(Expr::Path(
                                                            vec!["Instant".to_string(), "now".to_string()],
                                                            PathType::Namespace,
                                                        )),
                                                        Vec::new(),
                                                    )),
                                                }),
                                                // 计算距离上次调度的时间间隔
                                                Statement::Let(LetStmt {
                                                    ifmut: false,
                                                    name: "elapsed".to_string(),
                                                    ty: None,
                                                    init: Some(Expr::MethodCall(
                                                        Box::new(Expr::Ident("now".to_string())),
                                                        "duration_since".to_string(),
                                                        vec![Expr::Ident("last_dispatch".to_string())],
                                                    )),
                                                }),
                                                // 如果比最小间隔快，等待补足
                                                Statement::Expr(Expr::If {
                                                    condition: Box::new(Expr::BinaryOp(
                                                        Box::new(Expr::Ident("elapsed".to_string())),
                                                        "<".to_string(),
                                                        Box::new(Expr::Ident("min_interarrival".to_string())),
                                                    )),
                                                    then_branch: Block {
                                                        stmts: vec![
                                                            Statement::Expr(Expr::MethodCall(
                                                                Box::new(Expr::Path(
                                                                    vec!["std".to_string(), "thread".to_string(), "sleep".to_string()],
                                                                    PathType::Namespace,
                                                                )),
                                                                "".to_string(),
                                                                vec![Expr::BinaryOp(
                                                                    Box::new(Expr::Ident("min_interarrival".to_string())),
                                                                    "-".to_string(),
                                                                    Box::new(Expr::Ident("elapsed".to_string())),
                                                                )],
                                                            )),
                                                        ],
                                                        expr: None,
                                                    },
                                                    else_branch: None,
                                                }),
                                                // 执行子程序调用处理，传递已读取的数据
                                                Statement::Expr(Expr::Block(Block {
                                                    stmts: self.create_subprogram_call_logic_with_data(impl_, has_receiving_subprograms),
                                                    expr: None,
                                                })),
                                                // 更新上次调度时间
                                                Statement::Expr(Expr::Assign(
                                                    Box::new(Expr::Ident("last_dispatch".to_string())),
                                                    Box::new(Expr::Call(
                                                        Box::new(Expr::Path(
                                                            vec!["Instant".to_string(), "now".to_string()],
                                                            PathType::Namespace,
                                                        )),
                                                        Vec::new(),
                                                    ))
                                                )),                                                
                                            ],
                                            expr: None,
                                        },
                                    },
                                    // Err(_) => 通道关闭，退出循环
                                    MatchArm {
                                        pattern: "Err(_)".to_string(),
                                        guard: None,
                                        body: Block {
                                            stmts: vec![
                                                Statement::Expr(Expr::Call(
                                                    Box::new(Expr::Path(
                                                        vec!["eprintln!".to_string()],
                                                        PathType::Namespace,
                                                    )),
                                                    vec![Expr::Literal(Literal::Str(format!("{}Thread: channel closed", impl_.name.type_identifier.to_lowercase())))],
                                                )),
                                                Statement::Expr(Expr::Ident("return".to_string())),
                                            ],
                                            expr: None,
                                        },
                                    },
                                ],
                            }),
                        ],
                        expr: None,
                    },
                    else_branch: None,
                }),
            ],
            expr: None,
        }))));

        stmts
    }

    /// 创建定时执行逻辑
    fn create_timed_execution_logic(&self, impl_: &ComponentImplementation) -> Vec<Statement> {
        let mut stmts = Vec::new();
        
        // 从AADL属性中提取最小间隔时间，默认为1000ms
        let period = self.extract_property_value(impl_, "period").unwrap_or(1000);
        stmts.push(Statement::Let(LetStmt {
            ifmut: false,
            name: "period".to_string(),
            ty: Some(Type::Path(vec![
                "std".to_string(),
                "time".to_string(),
                "Duration".to_string(),
            ])),
            init: Some(Expr::Call(
                Box::new(Expr::Path(
                    vec!["Duration".to_string(), "from_millis".to_string()],
                    PathType::Namespace,
                )),
                vec![Expr::Literal(Literal::Int(period as i64))],
            )),
        }));

        // 获取事件端口信息（事件端口或事件数据端口）
        let event_ports = self.extract_event_ports(impl_);
        
        // 如果没有找到事件端口，则从参数连接中获取接收端口作为备选
        let receive_ports = if !event_ports.is_empty() {
            event_ports
        } else {
            let subprogram_calls = self.extract_subprogram_calls(impl_);
            subprogram_calls.iter()
                .filter(|(_, _, _, is_send, _)| !is_send)
                .map(|(_, _, thread_port_name, _, _)| thread_port_name.clone())
                .collect()
        };

        // 检查是否有需要端口数据的子程序调用
        let subprogram_calls = self.extract_subprogram_calls(impl_);
        let has_receiving_subprograms = subprogram_calls.iter().any(|(_, _, _, is_send, _)| !is_send);

        // 生成定时执行逻辑 - 使用 recv_timeout 处理超时
        stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
            stmts: vec![
                // 检查是否有接收端口，如果有则等待消息
                // 动态获取第一个接收端口
                Statement::Expr(Expr::IfLet {
                    pattern: "Some(receiver)".to_string(),
                    value: Box::new(Expr::Reference(
                        Box::new(Expr::Path(
                            vec!["self".to_string(), 
                                if !receive_ports.is_empty() { 
                                    receive_ports[0].to_lowercase() 
                                } else { 
                                    "error_sink".to_string() 
                                }],
                            PathType::Member,
                        )),
                        true,
                        false,
                    )),
                    then_branch: Block {
                        stmts: vec![
                            // 使用 recv_timeout 等待消息，支持超时处理
                            Statement::Expr(Expr::Match {
                                expr: Box::new(Expr::MethodCall(
                                    Box::new(Expr::Ident("receiver".to_string())),
                                    "recv_timeout".to_string(),
                                    vec![Expr::Ident("period".to_string())],
                                )),
                                arms: vec![
                                    // Ok(val) => 正常触发，处理接收到的消息
                                    MatchArm {
                                        pattern: "Ok(val)".to_string(),
                                        guard: None,
                                        body: Block {
                                            stmts: vec![
                                                // --- Compute Entrypoint (正常触发) ---
                                                Statement::Expr(Expr::Block(Block {
                                                    stmts: self.create_subprogram_call_logic_with_data(impl_, has_receiving_subprograms),
                                                    expr: None,
                                                })),
                                            ],
                                            expr: None,
                                        },
                                    },
                                    // Err(RecvTimeoutError::Timeout) => 超时触发
                                    MatchArm {
                                        pattern: "Err(std::sync::mpsc::RecvTimeoutError::Timeout)".to_string(),
                                        guard: None,
                                        body: Block {
                                            stmts: vec![
                                                // --- Recover Entrypoint (超时触发) ---
                                                Statement::Expr(Expr::Call(
                                                    Box::new(Expr::Path(
                                                        vec!["eprintln!".to_string()],
                                                        PathType::Namespace,
                                                    )),
                                                    vec![Expr::Literal(Literal::Str(format!("{}Thread: timeout dispatch → Recover_Entrypoint", impl_.name.type_identifier.to_lowercase())))],
                                                )),
                                                // recover_entrypoint();
                                                Statement::Expr(Expr::Ident("// recover_entrypoint();".to_string())),
                                            ],
                                            expr: None,
                                        },
                                    },
                                    // Err(_) => 通道关闭，退出循环
                                    MatchArm {
                                        pattern: "Err(_)".to_string(),
                                        guard: None,
                                        body: Block {
                                            stmts: vec![
                                                Statement::Expr(Expr::Call(
                                                    Box::new(Expr::Path(
                                                        vec!["eprintln!".to_string()],
                                                        PathType::Namespace,
                                                    )),
                                                    vec![Expr::Literal(Literal::Str(format!("{}Thread: channel closed", impl_.name.type_identifier.to_lowercase())))],
                                                )),
                                                Statement::Expr(Expr::Ident("return".to_string())),
                                            ],
                                            expr: None,
                                        },
                                    },
                                ],
                            }),
                        ],
                        expr: None,
                    },
                    else_branch: None,
                }),
            ],
            expr: None,
        }))));

        stmts
    }
    
    /// 创建子程序调用处理逻辑（提取公共部分）
    fn create_subprogram_call_logic(&self, impl_: &ComponentImplementation) -> Vec<Statement> {
        self.create_subprogram_call_logic_with_data(impl_, false)
    }

    /// 创建子程序调用处理逻辑（带数据参数版本）
    fn create_subprogram_call_logic_with_data(&self, impl_: &ComponentImplementation, has_receiving_subprograms: bool) -> Vec<Statement> {
        let mut port_handling_stmts = Vec::new();

        // 提取有参数端口的子程序调用信息
        let subprogram_calls = self.extract_subprogram_calls(impl_);
        
        // 从AADL的calls部分提取子程序调用序列
        let mut mycalls_sequence = Vec::new();
        if let CallSequenceClause::Items(calls_clause) = &impl_.calls {
            for call_clause in calls_clause {
                for subprocall in &call_clause.calls {
                    if let CalledSubprogram::Classifier(
                        UniqueComponentClassifierReference::Implementation(temp),
                    ) = &subprocall.called
                    {
                        let subprogram_name = temp.implementation_name.type_identifier.to_lowercase();
                        mycalls_sequence.push((subprocall.identifier.clone(), subprogram_name));
                    }
                }
            }
        }
        
        // 提取共享变量访问信息
        let data_access_calls = self.extract_data_access_calls(impl_);
        
        // 创建子程序调用映射
        let mut shared_var_subprograms = std::collections::HashMap::new();
        for (subprogram_name, _, shared_var_field) in &data_access_calls {
            shared_var_subprograms.insert(subprogram_name.clone(), shared_var_field.clone());
        }
        
        // 创建有参数端口的子程序集合
        let subprograms_with_ports: std::collections::HashSet<String> = subprogram_calls.iter()
            .map(|(_, spg_name, _, _, _)| spg_name.clone())
            .collect();
        
        // 添加调用序列注释
        if !mycalls_sequence.is_empty() {
            let call_sequence = mycalls_sequence.iter()
                .map(|(call_id, _)| format!("{}()", call_id))
                .collect::<Vec<_>>()
                .join(" -> ");
            
            port_handling_stmts.push(Statement::Expr(Expr::Ident(format!(
                "// --- 调用序列（等价 AADL 的 Wrapper）---\n                           // {}",
                call_sequence
            ))));
        }

        // 根据Mycalls中的顺序处理所有子程序调用
        for (call_id, subprogram_name) in mycalls_sequence {
            let has_parameter_ports = subprograms_with_ports.contains(&subprogram_name);
            
            port_handling_stmts.push(Statement::Expr(Expr::Ident(format!("// {}", call_id))));
            
            if has_parameter_ports {
                // 有参数端口的子程序处理
                if let Some((_, _, thread_port_name, is_send, port_type)) = subprogram_calls.iter()
                    .find(|(_, spg_name, _, _, _)| spg_name == &subprogram_name) {
                    
                    if *is_send {
                        // 发送模式
                        let mut send_stmts = Vec::new();
                        
                        // 根据端口类型生成合适的默认值
                        let default_value = self.generate_default_value_for_type(port_type);
                        send_stmts.push(Statement::Let(LetStmt {
                            name: "val".to_string(),
                            ty: None,
                            init: Some(default_value),
                            ifmut: true,
                        }));
                        
                        send_stmts.push(Statement::Expr(Expr::Call(
                            Box::new(Expr::Path(
                                vec![subprogram_name.clone(), "send".to_string()],
                                PathType::Namespace,
                            )),
                            vec![Expr::Reference(
                                Box::new(Expr::Ident("val".to_string())),
                                true,
                                true,
                            )],
                        )));
                        
                        send_stmts.push(Statement::Expr(Expr::MethodCall(
                            Box::new(Expr::MethodCall(
                                Box::new(Expr::Ident("sender".to_string())),
                                "send".to_string(),
                                vec![Expr::Ident("val".to_string())],
                            )),
                            "unwrap".to_string(),
                            Vec::new(),
                        )));
                        
                        port_handling_stmts.push(Statement::Expr(Expr::IfLet {
                            pattern: "Some(sender)".to_string(),
                            value: Box::new(Expr::Reference(
                                Box::new(Expr::Path(
                                    vec!["self".to_string(), thread_port_name.clone()],
                                    PathType::Member,
                                )),
                                true,
                                false,
                            )),
                            then_branch: Block {
                                stmts: send_stmts,
                                expr: None,
                            },
                            else_branch: None,
                        }));
                    } else {
                        // 接收模式
                        if has_receiving_subprograms {
                            // 如果有接收子程序且有已读取的数据，直接使用数据
                            port_handling_stmts.push(Statement::Expr(Expr::Call(
                                Box::new(Expr::Path(
                                    vec![subprogram_name.clone(), "receive".to_string()],
                                    PathType::Namespace,
                                )),
                                vec![Expr::Ident("val".to_string())],
                            )));
                        } else {
                            // 如果没有已读取的数据，则使用原来的try_recv逻辑
                            let mut receive_stmts = Vec::new();

                            let match_expr = Expr::Match {
                                expr: Box::new(Expr::MethodCall(
                                    Box::new(Expr::Ident("receiver".to_string())),
                                    "try_recv".to_string(),
                                    Vec::new(),
                                )),
                                arms: vec![
                                    MatchArm {
                                        pattern: "Ok(val)".to_string(),
                                        guard: None,
                                        body: Block {
                                            stmts: vec![Statement::Expr(Expr::Call(
                                                Box::new(Expr::Path(
                                                    vec![subprogram_name.clone(), "receive".to_string()],
                                                    PathType::Namespace,
                                                )),
                                                vec![Expr::Ident("val".to_string())],
                                            ))],
                                            expr: None,
                                        },
                                    },
                                    MatchArm {
                                        pattern: "Err(mpsc::TryRecvError::Empty)".to_string(),
                                        guard: None,
                                        body: Block { stmts: vec![], expr: None },
                                    },
                                    MatchArm {
                                        pattern: "Err(mpsc::TryRecvError::Disconnected)".to_string(),
                                        guard: None,
                                        body: Block {
                                            stmts: vec![Statement::Expr(Expr::Call(
                                                Box::new(Expr::Path(
                                                    vec!["eprintln!".to_string()],
                                                    PathType::Namespace,
                                                )),
                                                vec![Expr::Literal(Literal::Str("channel closed".to_string()))],
                                            ))],
                                            expr: None,
                                        },
                                    },
                                ],
                            };

                            receive_stmts.push(Statement::Expr(match_expr));

                            port_handling_stmts.push(Statement::Expr(Expr::IfLet {
                                pattern: "Some(receiver)".to_string(),
                                value: Box::new(Expr::Reference(
                                    Box::new(Expr::Path(
                                        vec!["self".to_string(), thread_port_name.clone()],
                                        PathType::Member,
                                    )),
                                    true,
                                    false,
                                )),
                                then_branch: Block {
                                    stmts: receive_stmts,
                                    expr: None,
                                },
                                else_branch: None,
                            }));
                        }
                    }
                }
            } else if let Some(shared_var_field) = shared_var_subprograms.get(&subprogram_name) {
                // 使用共享变量的子程序
                let mut lock_stmts = Vec::new();
                
                lock_stmts.push(Statement::Expr(Expr::Block(Block {
                    stmts: vec![
                        Statement::Expr(Expr::IfLet {
                            pattern: "Ok(mut guard)".to_string(),
                            value: Box::new(Expr::MethodCall(
                                Box::new(Expr::Path(
                                    vec!["self".to_string(), shared_var_field.clone()],
                                    PathType::Member,
                                )),
                                "lock".to_string(),
                                Vec::new(),
                            )),
                            then_branch: Block {
                                stmts: vec![
                                    Statement::Expr(Expr::Call(
                                        Box::new(Expr::Path(
                                            vec![subprogram_name.clone(), "call".to_string()],
                                            PathType::Namespace,
                                        )),
                                        vec![Expr::Reference(
                                            Box::new(Expr::Ident("guard".to_string())),
                                            true,
                                            true,
                                        )],
                                    )),
                                ],
                                expr: None,
                            },
                            else_branch: None,
                        }),
                    ],
                    expr: None,
                })));
                
                port_handling_stmts.push(Statement::Expr(Expr::Block(Block {
                    stmts: lock_stmts,
                    expr: None,
                })));
            } else {
                // 没有参数端口的普通子程序
                port_handling_stmts.push(Statement::Expr(Expr::Call(
                    Box::new(Expr::Path(
                        vec![subprogram_name.clone(), "execute".to_string()],
                        PathType::Namespace,
                    )),
                    Vec::new(),
                )));
            }
        }

        port_handling_stmts
    }

    // 辅助函数：提取属性值
    fn extract_property_value(&self, impl_: &ComponentImplementation, name: &str) -> Option<u64> {
        let target_name = name.to_lowercase();
        for prop in self.convert_properties(ComponentRef::Impl(impl_)) {
            if prop.name.to_lowercase() == target_name {
                match prop.value {
                    StruPropertyValue::Integer(val) => return Some(val as u64),
                    StruPropertyValue::Duration(val, unit) => {
                        println!(
                            "Warning: Found duration {} {} for property {}, expected integer",
                            val, unit, name
                        );
                        return Some(val); // 假设duration的数值部分可用
                    }
                    _ => {
                        println!("Warning: Property {} has unsupported type", name);
                        return None;
                    }
                }
            }
        }
        None
    }

    // 辅助函数：提取调度协议
    fn extract_dispatch_protocol(&self, impl_: &ComponentImplementation) -> Option<String> {
        let target_name = "dispatch_protocol";
        for prop in self.convert_properties(ComponentRef::Impl(impl_)) {
            if prop.name.to_lowercase() == target_name {
                match prop.value {
                    StruPropertyValue::String(val) => return Some(val),
                    _ => {
                        println!("Warning: Property {} has unsupported type", target_name);
                        return None;
                    }
                }
            }
        }
        None
    }
    //连接关系解析函数
    fn extract_subprogram_calls(
        &self,
        impl_: &ComponentImplementation,
    ) -> Vec<(String, String, String, bool, Type)> {
        let mut calls = Vec::new();

        // 解析calls部分,获得calls中调用子程序的identifier
        if let CallSequenceClause::Items(calls_clause) = &impl_.calls {
            for call_clause in calls_clause {
                for subprocall in &call_clause.calls {
                    if let CalledSubprogram::Classifier(
                        UniqueComponentClassifierReference::Implementation(temp),
                    ) = &subprocall.called
                    {
                        let subprogram_name =
                            temp.implementation_name.type_identifier.to_lowercase(); //calls中调用的子程序具体真实名称
                        let subprogram_identifier = subprocall.identifier.to_lowercase(); //calls中给调用子程序的标识符，例如P_Spg

                        //解析connections部分
                        if let ConnectionClause::Items(connections) = &impl_.connections {
                            for conn in connections {
                                if let Connection::Parameter(port_conn) = conn {
                                    //针对发送
                                    if let ParameterEndpoint::SubprogramCallParameter {
                                        call_identifier,
                                        parameter,
                                    } = &port_conn.source
                                    //这里针对"发送"连接，判断的是"源端口"的信息
                                    {
                                        let sou_parameter = parameter.to_lowercase();
                                        if subprogram_identifier == call_identifier.to_lowercase() {
                                            if let ParameterEndpoint::ComponentParameter {
                                                parameter,
                                                data_subcomponent,
                                            } = &port_conn.destination
                                            {
                                                let thread_port_name = parameter.to_lowercase();
                                                let port_type = self.get_subprogram_port_type(&subprogram_name, &sou_parameter);
                                                calls.push((
                                                    sou_parameter.to_lowercase(),    // 子程序端口名
                                                    subprogram_name.to_lowercase(),  // 子程序名
                                                    thread_port_name.to_lowercase(), // 线程端口名
                                                    true,
                                                    port_type,
                                                ));
                                            }
                                        }
                                    }
                                    //针对接收
                                    if let ParameterEndpoint::SubprogramCallParameter {
                                        call_identifier,
                                        parameter,
                                    } = &port_conn.destination
                                    //这里针对"接收"连接，判断的是"目的端口"的信息
                                    {
                                        let des_parameter = parameter.to_lowercase();
                                        if subprogram_identifier == call_identifier.to_lowercase() {
                                            if let ParameterEndpoint::ComponentParameter {
                                                parameter,
                                                data_subcomponent,
                                            } = &port_conn.source
                                            {
                                                let thread_port_name = parameter.to_lowercase();
                                                let port_type = self.get_subprogram_port_type(&subprogram_name, &des_parameter);
                                                calls.push((
                                                    des_parameter.to_lowercase(),
                                                    subprogram_name.to_lowercase(),
                                                    thread_port_name.to_lowercase(),
                                                    false,
                                                    port_type,
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        calls
    }
    
    // 提取事件端口和事件数据端口
    fn extract_event_ports(&self, impl_: &ComponentImplementation) -> Vec<String> {
        let mut event_ports = Vec::new();
        
        // 从组件类型中获取端口定义
        if let Some(comp_type) = self.get_component_type(impl_) {
            if let FeatureClause::Items(features) = &comp_type.features {
                for feature in features {
                    if let Feature::Port(port) = feature {
                        // 检查是否为事件端口或事件数据端口，且为输入方向
                        if matches!(port.port_type, PortType::Event | PortType::EventData { .. }) 
                           && port.direction == PortDirection::In {
                            event_ports.push(port.identifier.clone());
                        }
                    }
                }
            }
        }
        
        event_ports
    }


    
    // 提取系统实现中的处理器绑定信息
    fn extract_processor_bindings(&self, impl_: &ComponentImplementation) -> Vec<(String, String)> {
        let mut bindings = Vec::new();
        
        if let PropertyClause::Properties(properties) = &impl_.properties {
            for property in properties {
                if let Property::BasicProperty(basic_prop) = property {
                    if basic_prop.identifier.name.to_lowercase() == "actual_processor_binding" {
                        if let PropertyValue::Single(PropertyExpression::Reference(ref_term)) = &basic_prop.value {
                            if let Some(applies_to) = &ref_term.applies_to {
                                // 格式: (进程名, CPU标识符)
                                bindings.push((applies_to.clone(), ref_term.identifier.clone()));
                            }
                        }
                    }
                }
            }
        }
        
        bindings
    }
    // 创建系统实例中new()方法
    fn create_system_new_body(&mut self, impl_: &ComponentImplementation) -> Block {
        let mut stmts = Vec::new();

        // 1. 提取处理器绑定信息并创建CPU映射
        let processor_bindings = self.extract_processor_bindings(impl_);
        
        // 为每个唯一的CPU名称分配一个ID（如果还没有分配的话）
        for (_, cpu_name) in &processor_bindings {
            if !self.cpu_name_to_id_mapping.contains_key(cpu_name) {
                let next_id = self.cpu_name_to_id_mapping.len();
                self.cpu_name_to_id_mapping.insert(cpu_name.clone(), next_id);
            }
        }
        
        // 如果没有处理器绑定，默认使用CPU 0
        if self.cpu_name_to_id_mapping.is_empty() {
            self.cpu_name_to_id_mapping.insert("default".to_string(), 0);
        }

        // 2. 创建子组件实例 - 只处理进程组件
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            for sub in subcomponents {
                // 只处理进程组件
                if matches!(sub.category, ComponentCategory::Process) {
                    let var_name = sub.identifier.to_lowercase();
                    let type_name = match &sub.classifier {
                        SubcomponentClassifier::ClassifierReference(
                            UniqueComponentClassifierReference::Implementation(unirf),
                        ) => {
                            format!("{}", unirf.implementation_name.type_identifier)
                        }
                        _ => "UnsupportedComponent".to_string(),
                    };

                    // 查找该进程的CPU绑定
                    let cpu_id = processor_bindings.iter()
                        .find(|(process_name, _)| process_name == &sub.identifier)
                        .and_then(|(_, cpu_name)| {
                            self.cpu_name_to_id_mapping.get(cpu_name).copied()
                        })
                        .unwrap_or(0); // 默认使用CPU 0
                    
                    let creation_stmt = format!("let mut {}: {}Process = {}Process::new({})", 
                        var_name, type_name.to_lowercase(), type_name.to_lowercase(), cpu_id);
                    stmts.push(Statement::Expr(Expr::Ident(creation_stmt)));
                }
            }
        }

        // 2. 构建连接（如果有的话）
        if let ConnectionClause::Items(connections) = &impl_.connections {
            for conn in connections {
                match conn {
                    Connection::Port(port_conn) => {
                        // 处理端口连接，使用与进程相同的逻辑
                        stmts.extend(self.create_channel_connection(port_conn));
                    }
                    _ => {
                        // 对于其他类型的连接，生成TODO注释
                        stmts.push(Statement::Expr(Expr::Ident(format!(
                            "// TODO: Unsupported connection type in system: {:?}",
                            conn
                        ))));
                    }
                }
            }
        }

        // 3. 构建返回语句
        let mut field_names = Vec::new();
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            for sub in subcomponents {
                // 只包含进程组件的字段名
                if matches!(sub.category, ComponentCategory::Process) {
                    field_names.push(sub.identifier.to_lowercase());
                }
            }
        }

        let fields_str = field_names.join(", ");
        stmts.push(Statement::Expr(Expr::Ident(format!("return Self {{ {} }}  //显式return", fields_str))));

        Block { stmts, expr: None }
    }
    
    // 创建系统实例中run()方法
    fn create_system_run_body(&self, impl_: &ComponentImplementation) -> Block {
        let mut stmts = Vec::new();

        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            for sub in subcomponents {
                // 仅对进程子组件启动，其他子组件忽略
                if let ComponentCategory::Process = sub.category {
                    let var_name = sub.identifier.to_lowercase();

                    // 构建进程启动语句
                    let start_stmt = format!("self.{}.start()", var_name);
                    stmts.push(Statement::Expr(Expr::Ident(start_stmt)));
                }
            }
        }

        Block { stmts, expr: None }
    }
    
    /// 从数据组件实现名称中查找具体的数据类型
    /// 例如：从 POS.Impl 中找到 Field : data POS_Internal_Type 中的 POS_Internal_Type
    fn find_data_type_from_implementation(&self, impl_name: &str, package: &Package) -> Option<String> {
        // 在 Package 中查找组件实现
        if let Some(public_section) = &package.public_section {
            for decl in &public_section.declarations {
                if let AadlDeclaration::ComponentImplementation(impl_) = decl {
                    //println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!impl_.name.type_identifier: {}", impl_.name.type_identifier);
                    // 检查实现名称是否匹配：impl_name 可能是 "POS.Impl"，而 type_identifier 是 "POS"
                    // 所以需要检查 impl_name 是否以 type_identifier 开头
                    if impl_name.starts_with(&impl_.name.type_identifier) {
                        // 找到匹配的组件实现，查找其中的数据子组件
                        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
                            for sub in subcomponents {
                                if sub.category == ComponentCategory::Data {
                                    // 从数据子组件中提取类型名
                                    if let SubcomponentClassifier::ClassifierReference(
                                        UniqueComponentClassifierReference::Implementation(unirf),
                                    ) = &sub.classifier {
                                        return Some(unirf.implementation_name.type_identifier.clone());
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
                if let AadlDeclaration::ComponentImplementation(impl_) = decl {
                    // 检查实现名称是否匹配：impl_name 可能是 "POS.Impl"，而 type_identifier 是 "POS"
                    // 所以需要检查 impl_name 是否以 type_identifier 开头
                    if impl_name.starts_with(&impl_.name.type_identifier) {
                        // 找到匹配的组件实现，查找其中的数据子组件
                        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
                            for sub in subcomponents {
                                if sub.category == ComponentCategory::Data {
                                    // 从数据子组件中提取类型名
                                    if let SubcomponentClassifier::ClassifierReference(
                                        UniqueComponentClassifierReference::Implementation(unirf),
                                    ) = &sub.classifier {
                                        return Some(unirf.implementation_name.type_identifier.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
    

    
    /// 提取data access连接，识别哪些子程序使用共享变量
    /// 返回：(子程序名, 共享变量名, 共享变量字段名)
    fn extract_data_access_calls(&self, impl_: &ComponentImplementation) -> Vec<(String, String, String)> {
        let mut data_access_calls = Vec::new();
        
        // 首先从Mycalls中提取调用标识符到子程序名的映射
        let mut call_id_to_subprogram = std::collections::HashMap::new();
        if let CallSequenceClause::Items(calls_clause) = &impl_.calls {
            for call_clause in calls_clause {
                for subprocall in &call_clause.calls {
                    if let CalledSubprogram::Classifier(
                        UniqueComponentClassifierReference::Implementation(temp),
                    ) = &subprocall.called
                    {
                        let subprogram_name = temp.implementation_name.type_identifier.to_lowercase();
                        call_id_to_subprogram.insert(subprocall.identifier.to_lowercase(), subprogram_name);
                    }
                }
            }
        }
        
        if let ConnectionClause::Items(connections) = &impl_.connections {
            for conn in connections {
                if let Connection::Access(access_conn) = conn {
                    // 处理 data access 连接：ComponentAccess(data) -> SubcomponentAccess{ subcomponent: thread, .. }
                    match (&access_conn.source, &access_conn.destination) {
                        (AccessEndpoint::ComponentAccess(data_name), AccessEndpoint::SubcomponentAccess { subcomponent: call_identifier, .. }) => {
                            // 从调用标识符中提取子程序名
                            if let Some(subprogram_name) = call_id_to_subprogram.get(&call_identifier.to_lowercase()) {
                                // 从数据名称中提取共享变量字段名
                                let shared_var_field = data_name.to_lowercase();
                                
                                // 共享变量名（用于注释）
                                let shared_var_name = data_name.clone();
                                
                                data_access_calls.push((subprogram_name.clone(), shared_var_name, shared_var_field));
                            }
                        }
                        _ => {} // 其他方向的连接暂不处理
                    }
                }
            }
        }
        
        data_access_calls
    }

    // 生成CPU调度策略映射的静态代码
    
    // 转换CPU实现
    fn convert_processor_implementation(&mut self, impl_: &ComponentImplementation) -> Vec<Item> {
        // 从CPU实现中提取Scheduling_Protocol属性并保存
        let cpu_name = impl_.name.type_identifier.clone();
        
        if let PropertyClause::Properties(props) = &impl_.properties {
            for prop in props {
                if let Property::BasicProperty(bp) = prop {
                    if bp.identifier.name.to_lowercase() == "scheduling_protocol" {
                        if let PropertyValue::Single(PropertyExpression::String(
                            StringTerm::Literal(scheduling_protocol),
                        )) = &bp.value
                        {
                            self.cpu_scheduling_protocols.insert(cpu_name.clone(), scheduling_protocol.clone());
                            return Vec::new(); // CPU实现不生成代码，只保存信息
                        }
                    }
                }
            }
        }
        
        // 如果没有找到Scheduling_Protocol属性，使用默认值
        self.cpu_scheduling_protocols.insert(cpu_name.clone(), "FIFO".to_string());
        println!("CPU实现 {} 未指定调度协议，使用默认值: FIFO", cpu_name);
        
        Vec::new() // CPU实现不生成代码，只保存信息
    }
    fn convert_cpu_schedule_mapping(&self, module: &mut RustModule, cpu_scheduling_protocols: &HashMap<String, String>, cpu_name_to_id_mapping: &HashMap<String, usize>) {
        // 如果没有CPU映射信息，则不生成代码
        if cpu_name_to_id_mapping.is_empty() {
            return;
        }

        // 生成map.insert语句
        let mut map_insertions = Vec::new();
        
        for (cpu_name, cpu_id) in cpu_name_to_id_mapping {
            // 获取该CPU的调度协议
            let scheduling_protocol = cpu_scheduling_protocols.get(cpu_name)
                .map(|s| s.as_str())
                .unwrap_or("FIFO"); // 默认使用FIFO
            
            // 将调度协议转换为对应的常量
            let sched_constant = match scheduling_protocol.to_uppercase().as_str() {
                "POSIX_1003_HIGHEST_PRIORITY_FIRST_PROTOCOL" | "HPF" => "SCHED_FIFO",
                "ROUND_ROBIN_PROTOCOL" | "RR" => "SCHED_RR", 
                "EDF" | "EARLIEST_DEADLINE_FIRST_PROTOCOL" => "SCHED_DEADLINE",
                "RATE_MONOTONIC_PROTOCOL" | "RMS" | "RM" => "SCHED_FIFO",
                "DEADLINE_MONOTONIC_PROTOCOL" | "DM" | "DMS"=> "SCHED_FIFO",
                _ => "SCHED_FIFO", // 默认值
            };
            
            // 生成 map.insert(cpu_id, sched_constant);
            map_insertions.push(Statement::Expr(Expr::MethodCall(
                Box::new(Expr::Ident("map".to_string())),
                "insert".to_string(),
                vec![
                    Expr::Literal(Literal::Int(*cpu_id as i64)),
                    Expr::Path(vec![sched_constant.to_string()], PathType::Namespace),
                ],
            )));
        }

        // 构建初始化块的代码
        let mut init_stmts = Vec::new();
        
        // let mut map = HashMap::new();
        init_stmts.push(Statement::Let(LetStmt {
            ifmut: true,
            name: "map".to_string(),
            ty: Some(Type::Generic("HashMap".to_string(), vec![
                Type::Named("isize".to_string()),
                Type::Named("i32".to_string()),
            ])),
            init: Some(Expr::Call(
                Box::new(Expr::Path(vec!["HashMap".to_string(), "new".to_string()], PathType::Namespace)),
                Vec::new(),
            )),
        }));
        
        // 添加map.insert语句
        init_stmts.extend(map_insertions);
        
        // map // 返回map
        init_stmts.push(Statement::Expr(Expr::Ident("return map".to_string())));

        // 创建 LazyStaticDef
        let lazy_static_def = LazyStaticDef {
            name: "CPU_ID_TO_SCHED_POLICY".to_string(),
            ty: Type::Generic("HashMap".to_string(), vec![
                Type::Named("isize".to_string()),
                Type::Named("i32".to_string()),
            ]),
            init: Block {
                stmts: init_stmts,
                expr: None,
            },
            vis: Visibility::Public,
            docs: vec![
                "// CPU ID到调度策略的映射".to_string(),
            ],
        };

        // 将 LazyStatic 添加到模块中
        module.items.push(Item::LazyStatic(lazy_static_def));
    }

    /// 添加 period_to_priority 函数到模块中
    /// 该函数根据周期计算优先级：prio(P)=max(1,min(99,99−⌊k⋅log10(P)⌋))
    /// 只有在检测到 RMS 或 DMS 调度协议时才生成此函数
    fn add_period_to_priority(&self, module: &mut RustModule, cpu_scheduling_protocols: &HashMap<String, String>) {
        // 检查是否有 RMS 或 DMS 调度协议
        let has_rms_or_dms = cpu_scheduling_protocols.values().any(|protocol| {
            let protocol_upper = protocol.to_uppercase();
            protocol_upper.contains("RATE_MONOTONIC") || 
            protocol_upper.contains("RMS") || 
            protocol_upper.contains("RM") ||
            protocol_upper.contains("DEADLINE_MONOTONIC") ||
            protocol_upper.contains("DMS") ||
            protocol_upper.contains("DM")
        });
        
        // 如果没有 RMS 或 DMS 调度协议，则不生成函数
        if !has_rms_or_dms {
            return;
        }
        
        // 构建函数体
        let mut body_stmts = Vec::new();
        
        // let k = 10.0; // 每增加一个数量级，优先级下降10
        body_stmts.push(Statement::Let(LetStmt {
            ifmut: false,
            name: "k".to_string(),
            ty: Some(Type::Named("f64".to_string())),
            init: Some(Expr::Ident("10.0".to_string())),
        }));
        
        // let raw = 99.0 - (k * period_ms.log10()).floor();
        body_stmts.push(Statement::Let(LetStmt {
            ifmut: false,
            name: "raw".to_string(),
            ty: Some(Type::Named("f64".to_string())),
            init: Some(Expr::BinaryOp(
                Box::new(Expr::Ident("99.0".to_string())),
                "-".to_string(),
                Box::new(Expr::MethodCall(
                    Box::new(Expr::BinaryOp(
                        Box::new(Expr::Ident("k".to_string())),
                        "*".to_string(),
                        Box::new(Expr::MethodCall(
                            Box::new(Expr::Ident("period_ms".to_string())),
                            "log10".to_string(),
                            Vec::new(),
                        )),
                    )),
                    "floor".to_string(),
                    Vec::new(),
                )),
            )),
        }));
        
        // raw.max(1.0).min(99.0) as i32
        body_stmts.push(Statement::Expr(Expr::Ident("return raw.max(1.0).min(99.0) as i32".to_string())));
        
        // 创建函数定义
        let function_def = FunctionDef {
            name: "period_to_priority".to_string(),
            params: vec![Param {
                name: "period_ms".to_string(),
                ty: Type::Named("f64".to_string()),
            }],
            return_type: Type::Named("i32".to_string()),
            body: Block {
                stmts: body_stmts,
                expr: None,
            },
            asyncness: false,
            vis: Visibility::Public,
            docs: vec![
                "// prio(P)=max(1,min(99,99−⌊k⋅log10(P)⌋))".to_string(),
                "// 根据周期计算优先级，周期越短优先级越高".to_string(),
                "// 用于 RMS (Rate Monotonic Scheduling) 和 DMS (Deadline Monotonic Scheduling)".to_string(),
            ],
            attrs: Vec::new(),
        };
        
        // 将函数添加到模块中
        module.items.push(Item::Function(function_def));
    }

}
