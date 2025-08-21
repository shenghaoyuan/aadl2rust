// src/aadl_to_rust/converter.rs
// aadlAST2rustAST
use super::intermediate_ast::*;
use crate::ast::aadl_ast_cj::*;
use std::{collections::HashMap, default};

// AADL到Rust中间表示的转换器
pub struct AadlConverter {
    type_mappings: HashMap<String, Type>,
    port_handlers: HashMap<String, PortHandlerConfig>,
    component_types: HashMap<String, ComponentType>, // 存储组件类型信息，（为了有些情况下，需要在组件实现中，根据组件类型来获取端口信息）
}

#[derive(Debug)]
struct PortHandlerConfig {
    // 端口处理配置
}

impl Default for AadlConverter {
    fn default() -> Self {
        let mut type_mappings = HashMap::new();
        type_mappings.insert("Integer".to_string(), Type::Named("i32".to_string()));
        type_mappings.insert("String".to_string(), Type::Named("String".to_string()));
        type_mappings.insert("Boolean".to_string(), Type::Named("bool".to_string()));

        Self {
            type_mappings,
            port_handlers: HashMap::new(),
            component_types: HashMap::new(),
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

    fn convert_declaration(&self, decl: &AadlDeclaration, module: &mut RustModule, package: &Package) {
        match decl {
            AadlDeclaration::ComponentType(comp) => {
                module.items.extend(self.convert_component(comp, package));
            }
            AadlDeclaration::ComponentImplementation(impl_) => {
                module.items.extend(self.convert_implementation(impl_));
            }
            _ => {} // TODO:忽略其他声明类型
        }
    }

    fn convert_component(&self, comp: &ComponentType, package: &Package) -> Vec<Item> {
        match comp.category {
            ComponentCategory::Data => self.convert_data_component(comp),
            ComponentCategory::Thread => self.convert_thread_component(comp),
            ComponentCategory::Subprogram => self.convert_subprogram(comp, package),
            ComponentCategory::System => self.convert_system_component(comp),
            _ => Vec::default(), //TODO:进程还需要处理
        }
    }

    fn convert_data_component(&self, comp: &ComponentType) -> Vec<Item> {
        let target_type = self.determine_data_type(comp);
        
        // 当 determine_data_type 返回空元组类型时，不继续处理
        if let Type::Named(unit_type) = &target_type {
            if unit_type == "()" {
                return Vec::new();
            }
        }
        
        vec![Item::TypeAlias(TypeAlias {
            name: comp.identifier.clone(),
            target: target_type,
            vis: Visibility::Public,
            docs: vec![format!("// AADL Data Type: {}", comp.identifier.clone())],
        })]
    }

    fn determine_data_type(&self, comp: &ComponentType) -> Type {
        if let PropertyClause::Properties(props) = &comp.properties {
            for prop in props {
                if let Property::BasicProperty(bp) = prop {
                    // 处理 Data_Model::Data_Representation 属性
                    if bp.identifier.name.to_lowercase() == "data_model" {
                        if let Some(property_set) = &bp.identifier.property_set {
                            if property_set.to_lowercase() == "data_representation" {
                                if let PropertyValue::Single(PropertyExpression::String(
                                    StringTerm::Literal(str_val),
                                )) = &bp.value
                                {
                                    // 使用 type_mappings 查找对应的类型，如果没有找到则使用原值
                                    return self
                                        .type_mappings
                                        .get(&str_val.to_string())
                                        .cloned()
                                        .unwrap_or_else(|| Type::Named(str_val.to_string()));
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
                                .get(&str_val.to_string())
                                .cloned()
                                .unwrap_or_else(|| Type::Named(str_val.to_string()));
                            //没有在 type_mappings 中找到对应映射时，直接使用这个原始值作为类型名
                        }
                    }
                }
            }
        }
        Type::Named("()".to_string())
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
        if let Some(impl_block) = self.create_threadtype_impl(comp) {
            items.push(Item::Impl(impl_block));
        }

        items
    }

    fn convert_system_component(&self, comp: &ComponentType) -> Vec<Item> {
        let mut items = Vec::new();

        // 1. 结构体定义
        let mut fields = vec![
            Field {
                name: "processes".to_string(),
                ty: Type::Named("Vec<(String, isize)>".to_string()), // (进程名, CPU编号)
                docs: vec!["// 进程和CPU的对应关系".to_string()],
                attrs: Vec::new(),
            },
        ];
        
        let struct_def = StructDef {
            name: format!("{}System", comp.identifier.to_lowercase()),
            fields,
            properties: self.convert_properties(ComponentRef::Type(&comp)),
            generics: Vec::new(),
            derives: vec!["Debug".to_string()],
            docs: vec![format!("// AADL System: {}", comp.identifier)],
            vis: Visibility::Public,
        };
        items.push(Item::Struct(struct_def));

        items
    }

    fn create_system_impl_block(&self, impl_: &ComponentImplementation) -> ImplBlock {
        ImplBlock {
            target: Type::Named(format!("{}System", impl_.name.type_identifier.to_lowercase())),
            generics: Vec::new(),
            items: vec![
                ImplItem::Method(FunctionDef {
                    name: "new".to_string(),
                    params: Vec::new(),
                    return_type: Type::Named("Self".to_string()),
                    body: self.create_system_new_body(impl_),
                    asyncness: false,
                    vis: Visibility::Public,
                    docs: vec!["// 创建系统实例".to_string()],
                    attrs: Vec::new(),
                }),
                ImplItem::Method(FunctionDef {
                    name: "run".to_string(),
                    params: vec![Param {
                        name: "self".to_string(),
                        ty: Type::Named("Self".to_string()),
                    }],
                    return_type: Type::Unit,
                    body: self.create_system_run_body(impl_),
                    asyncness: false,
                    vis: Visibility::Public,
                    docs: vec!["// 运行系统，启动所有进程".to_string()],
                    attrs: Vec::new(),
                }),
            ],
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
                    .map(|c| self.classifier_to_type(c)) //对 Option 类型调用 .map() 方法，用于在 Some(...) 中包裹的值c上应用一个函数。
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
                    .get(&type_ref.implementation_name.type_identifier)
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

    fn create_threadtype_impl(&self, comp: &ComponentType) -> Option<ImplBlock> {
        // 如果未提取到 period，说明不是周期性函数(也可能是period在实现中不在原型里)，则提前返回 None
        let period = self.extract_period(comp)?;
        Some(ImplBlock {
            target: Type::Named(format!("{}Thread", comp.identifier.to_lowercase())),
            generics: Vec::new(),
            items: vec![ImplItem::Method(FunctionDef {
                name: "run".to_string(),
                params: vec![Param {
                    name: "self".to_string(),
                    ty: Type::Reference(
                        Box::new(Type::Named(format!(
                            "{}Thread",
                            comp.identifier.to_lowercase()
                        ))),
                        true,
                        true,
                    ),
                }],
                return_type: Type::Unit,
                body: self.create_thread_body(period),
                asyncness: true,
                vis: Visibility::Public,
                docs: vec![
                    "// Thread execution entry point".to_string(),
                    format!("// Period: {}ms", period),
                ],
                attrs: Vec::new(),
            })],
            trait_impl: None,
        })
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

    fn create_thread_body(&self, period_ms: u64) -> Block {
        Block {
            stmts: vec![
                Statement::Let(LetStmt {
                    ifmut: false,
                    name: "interval".to_string(),
                    ty: Some(Type::Path(vec![
                        "tokio".to_string(),
                        "time".to_string(),
                        "Interval".to_string(),
                    ])),
                    init: Some(Expr::Call(
                        Box::new(Expr::Path(
                            vec![
                                "tokio".to_string(),
                                "time".to_string(),
                                "interval".to_string(),
                            ],
                            PathType::Namespace,
                        )),
                        vec![Expr::Call(
                            Box::new(Expr::Path(
                                vec!["Duration".to_string(), "from_millis".to_string()],
                                PathType::Namespace,
                            )),
                            vec![Expr::Literal(Literal::Int(period_ms as i64))],
                        )],
                    )),
                }),
                Statement::Expr(Expr::Loop(Box::new(Block {
                    stmts: vec![
                        Statement::Expr(Expr::MethodCall(
                            Box::new(Expr::Ident("interval".to_string())),
                            "tick".to_string(),
                            Vec::new(),
                        )),
                        Statement::Expr(Expr::Await(Box::new(Expr::Ident("_".to_string())))),
                    ],
                    expr: None,
                }))),
            ],
            expr: None,
        }
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
                            _ => {}
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

    fn convert_generic_component(&self, comp: &ComponentType) -> Vec<Item> {
        let mut fields = Vec::new();
        // 添加 CPU ID 字段
        fields.push(Field {
            name: "cpu_id".to_string(),
            ty: Type::Named("isize".to_string()),
            docs: vec!["// 新增 CPU ID".to_string()],
            attrs: Vec::new(),
        });
        
        vec![Item::Struct(StructDef {
            name: comp.identifier.to_lowercase(),
            fields,
            properties: Vec::new(),
            generics: Vec::new(),
            derives: vec!["Debug".to_string()],
            docs: vec![format!("// AADL {:?} component", comp.category)],
            vis: Visibility::Public,
        })]
    }

    fn convert_implementation(&self, impl_: &ComponentImplementation) -> Vec<Item> {
        match impl_.category {
            ComponentCategory::Process => self.convert_process_implementation(impl_),
            ComponentCategory::Thread => self.convert_thread_implemenation(impl_),
            ComponentCategory::System => self.convert_system_implementation(impl_),
            ComponentCategory::Data => self.convert_data_implementation(impl_),
            _ => Vec::default(), // 默认实现
        }
    }

    fn convert_system_implementation(&self, impl_: &ComponentImplementation) -> Vec<Item> {
        let mut items = Vec::new();

        // 生成系统实现块
        items.push(Item::Impl(self.create_system_impl_block(impl_)));

        items
    }

    fn convert_data_implementation(&self, impl_: &ComponentImplementation) -> Vec<Item> {
        let mut items = Vec::new();

        // 检查子组件，判断是否为共享变量
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
        }

        items
    }

    fn convert_process_implementation(&self, impl_: &ComponentImplementation) -> Vec<Item> {
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
    fn get_process_fields(&self, impl_: &ComponentImplementation) -> Vec<Field> {
        let mut fields = Vec::new();

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
                    ComponentCategory::Thread => Type::Named(format!("{}Thread", type_name.to_lowercase())),
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

        // 2. 建立连接
        if let ConnectionClause::Items(connections) = &impl_.connections {
            for conn in connections {
                if let Connection::Port(port_conn) = conn {
                    stmts.extend(self.create_channel_connection(port_conn));
                }
            }
        }

        // 3. 返回结构体实例
        let fields = if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            subcomponents
                .iter()
                .map(|s| s.identifier.to_lowercase())
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::new()
        };

        // 构建完整的字段列表，包括 cpu_id
        let all_fields = if fields.is_empty() {
            "cpu_id".to_string()
        } else {
            format!("{}, cpu_id", fields)
        };

        stmts.push(Statement::Expr(Expr::Ident(format!(
            "return Self {{ {} }}  //显式return",
            all_fields
        ))));

        Block { stmts, expr: None }
    }

    fn create_process_start_body(&self, impl_: &ComponentImplementation) -> Block {
        let mut stmts = Vec::new();

        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            for sub in subcomponents {
                // 仅对线程子组件启动线程，数据子组件忽略
                if let ComponentCategory::Thread = sub.category {
                    let var_name = sub.identifier.to_lowercase();

                    // 构建线程闭包（使用move语义）
                    let closure = Expr::Closure(
                        Vec::new(), // 无参数
                        Box::new(Expr::MethodCall(
                            Box::new(Expr::Path(
                                vec!["self".to_string(), var_name.clone()],
                                PathType::Member,
                            )),
                            "run".to_string(),
                            Vec::new(),
                        )),
                    );

                    // 构建线程构建器表达式链
                    let builder_chain = vec![
                        BuilderMethod::Named(format!("\"{}\".to_string()", var_name)),
                        BuilderMethod::Spawn {
                            closure: Box::new(closure),
                            move_kw: true,
                        },
                    ];

                    stmts.push(Statement::Expr(Expr::MethodCall(
                        Box::new(Expr::BuilderChain(builder_chain)),
                        "unwrap".to_string(),
                        Vec::new(),
                    )));
                }
            }
        }

        Block { stmts, expr: None }
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
                stmts.push(Statement::Expr(Expr::MethodCall(
                    Box::new(Expr::Ident(port_name.clone())),
                    "send".to_string(),
                    vec![Expr::Ident("channel.0".to_string())],
                )));

                stmts.push(Statement::Expr(Expr::MethodCall(
                    Box::new(Expr::Ident(format!("{}.{}", dst_comp, dst_port))),
                    "receive".to_string(),
                    vec![Expr::Ident("channel.1".to_string())],
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
        let mut docs = vec![format!(
            "// AADL {:?}: {}",
            comp.category,
            comp.identifier.to_lowercase()
        )];

        docs
    }

    fn create_component_impl_docs(&self, impl_: &ComponentImplementation) -> Vec<String> {
        let mut docs = vec![format!(
            "// AADL {:?}: {}",
            impl_.category,
            impl_.name.type_identifier.to_lowercase()
        )];

        docs
    }

    fn convert_thread_implemenation(&self, impl_: &ComponentImplementation) -> Vec<Item> {
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
    fn create_thread_run_body(&self, impl_: &ComponentImplementation) -> Block {
        let mut stmts = Vec::new();
        
        //======================= 线程优先级设置 ========================
        // 如果线程有 priority 属性，则设置线程优先级
        if let Some(priority) = self.extract_property_value(impl_, "priority") {
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
                    // let ret = pthread_setschedparam(pthread_self(), SCHED_FIFO, &mut param);
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
                                Expr::Path(
                                    vec!["SCHED_FIFO".to_string()],
                                    PathType::Namespace,
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
        println!("!!!!!!!!!!!!!!!!!!!!!!!dispatch_protocol: {:?}", dispatch_protocol);
        
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
            _ => {
                // 默认使用周期性调度
                stmts.extend(self.create_periodic_execution_logic(impl_));
            }
        }

        Block { stmts, expr: None }
    }

    /// 创建周期性执行逻辑
    fn create_periodic_execution_logic(&self, impl_: &ComponentImplementation) -> Vec<Statement> {
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

        // 生成子程序调用处理代码
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

        stmts
    }

    /// 创建非周期性执行逻辑
    fn create_aperiodic_execution_logic(&self, impl_: &ComponentImplementation) -> Vec<Statement> {
        let mut stmts = Vec::new();
        
        // 生成子程序调用处理代码
        let port_handling_stmts = self.create_subprogram_call_logic(impl_);

        // 生成事件驱动的执行逻辑
        stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
            stmts: vec![
                // 执行子程序调用处理块
                Statement::Expr(Expr::Block(Block {
                    stmts: port_handling_stmts.clone(),
                    expr: None,
                })),
                // 短暂休眠，避免CPU占用过高
                Statement::Expr(Expr::MethodCall(
                    Box::new(Expr::Path(
                        vec!["std".to_string(), "thread".to_string(), "sleep".to_string()],
                        PathType::Namespace,
                    )),
                    "".to_string(),
                    vec![Expr::Call(
                        Box::new(Expr::Path(
                            vec!["Duration".to_string(), "from_millis".to_string()],
                            PathType::Namespace,
                        )),
                        vec![Expr::Literal(Literal::Int(10))], // 10ms休眠
                    )],
                )),
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
                .filter(|(_, _, _, is_send)| !is_send)
                .map(|(_, _, thread_port_name, _)| thread_port_name.clone())
                .collect()
        };

        // 检查是否有需要端口数据的子程序调用
        let subprogram_calls = self.extract_subprogram_calls(impl_);
        let has_receiving_subprograms = subprogram_calls.iter().any(|(_, _, _, is_send)| !is_send);

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
                                                Statement::Let(LetStmt {
                                                    ifmut: true,
                                                    name: "last_dispatch".to_string(),
                                                    ty: None,
                                                    init: Some(Expr::Call(
                                                        Box::new(Expr::Path(
                                                            vec!["Instant".to_string(), "now".to_string()],
                                                            PathType::Namespace,
                                                        )),
                                                        Vec::new(),
                                                    )),
                                                }),
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
            .map(|(_, spg_name, _, _)| spg_name.clone())
            .collect();
        
        // 添加调用序列注释
        if !mycalls_sequence.is_empty() {
            let call_sequence = mycalls_sequence.iter()
                .map(|(call_id, _)| format!("{}()", call_id))
                .collect::<Vec<_>>()
                .join(" -> ");
            
            port_handling_stmts.push(Statement::Expr(Expr::Ident(format!(
                "// --- 调用序列（等价 AADL 的 Wrapper）---\n            // {}",
                call_sequence
            ))));
        }

        // 根据Mycalls中的顺序处理所有子程序调用
        for (call_id, subprogram_name) in mycalls_sequence {
            let has_parameter_ports = subprograms_with_ports.contains(&subprogram_name);
            
            port_handling_stmts.push(Statement::Expr(Expr::Ident(format!("// {}", call_id))));
            
            if has_parameter_ports {
                // 有参数端口的子程序处理
                if let Some((_, _, thread_port_name, is_send)) = subprogram_calls.iter()
                    .find(|(_, spg_name, _, _)| spg_name == &subprogram_name) {
                    
                    if *is_send {
                        // 发送模式
                        let mut send_stmts = Vec::new();
                        
                        send_stmts.push(Statement::Let(LetStmt {
                            name: "val".to_string(),
                            ty: None,
                            init: Some(Expr::Literal(Literal::Int(0))),
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
    ) -> Vec<(String, String, String, bool)> {
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
                                                calls.push((
                                                    sou_parameter.to_lowercase(),    // 子程序端口名
                                                    subprogram_name.to_lowercase(),  // 子程序名
                                                    thread_port_name.to_lowercase(), // 线程端口名
                                                    true,
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
                                                calls.push((
                                                    des_parameter.to_lowercase(),
                                                    subprogram_name.to_lowercase(),
                                                    thread_port_name.to_lowercase(),
                                                    false,
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

    // 20250813新增辅助函数：提取没有参数端口的子程序调用
    fn extract_subprogram_calls_no_ports(&self, impl_: &ComponentImplementation) -> Vec<String> {
        let mut calls = Vec::new();

        if let CallSequenceClause::Items(calls_clause) = &impl_.calls {
            for call_clause in calls_clause {
                for subprocall in &call_clause.calls {
                    if let CalledSubprogram::Classifier(
                        UniqueComponentClassifierReference::Implementation(temp),
                    ) = &subprocall.called
                    {
                        let subprogram_name = temp.implementation_name.type_identifier.to_lowercase();
                        
                        // 检查是否有参数连接
                        let has_connections = if let ConnectionClause::Items(connections) = &impl_.connections {
                            connections.iter().any(|conn| {
                                if let Connection::Parameter(port_conn) = conn {
                                    match (&port_conn.source, &port_conn.destination) {
                                        (
                                            ParameterEndpoint::SubprogramCallParameter { call_identifier, .. },
                                            _,
                                        ) | (
                                            _,
                                            ParameterEndpoint::SubprogramCallParameter { call_identifier, .. },
                                        ) => call_identifier.to_lowercase() == subprocall.identifier.to_lowercase(),
                                        _ => false,
                                    }
                                } else {
                                    false
                                }
                            })
                        } else {
                            false
                        };

                        if !has_connections {
                            calls.push(subprogram_name);
                        }
                    }
                }
            }
        }

        calls
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
    fn create_system_new_body(&self, impl_: &ComponentImplementation) -> Block {
        let processor_bindings = self.extract_processor_bindings(impl_);
        let mut stmts = Vec::new();
        
        // 收集系统内的进程子组件，默认 cpu_id 为 -1
        let mut process_order: Vec<String> = Vec::new();
        let mut proc_to_cpu: HashMap<String, isize> = HashMap::new();
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            for sub in subcomponents {
                if sub.category == ComponentCategory::Process {
                    let proc_name = sub.identifier.clone();
                    if !proc_to_cpu.contains_key(&proc_name) {
                        process_order.push(proc_name.clone());
                        proc_to_cpu.insert(proc_name, -1);
                    }
                }
            }
        }

        // 在默认设置所有的进程cpu后，去查看cpu绑定的属性，覆盖绑定了处理器的进程 cpu_id
        let mut cpu_counter = 0; //从0开始设置cpu的编号
        let mut cpu_name_to_num = std::collections::HashMap::new();

        for (proc_name, cpu_name) in processor_bindings {
            // 如果是新CPU名，就分配一个编号
            let cpu_num = *cpu_name_to_num
                .entry(cpu_name.clone())
                .or_insert_with(|| {
                    let id = cpu_counter;
                    cpu_counter += 1;
                    id
                });

            if !proc_to_cpu.contains_key(&proc_name) {
                process_order.push(proc_name.clone());
            }
            proc_to_cpu.insert(proc_name, cpu_num);
        }
        
        // 生成 vec![(proc, cpu)]
        let mut processes_vec: Vec<String> = Vec::new();
        for name in process_order {
            let cpu = *proc_to_cpu.get(&name).unwrap_or(&-1);
            processes_vec.push(format!("(\"{}\".to_string(), {})", name, cpu));
        }
        
        let processes_str = format!("vec![{}]", processes_vec.join(", "));
        
        stmts.push(Statement::Expr(Expr::Ident(format!("return Self {{ processes: {} }}", processes_str))));
        
        Block { stmts, expr: None }
    }
    
    // 创建系统实例中run()方法,目前有些硬编码的方式
    fn create_system_run_body(&self, impl_: &ComponentImplementation) -> Block {
        let mut stmts = Vec::new();

        // 收集系统内所有进程子组件的名称和类型
        let mut process_info: Vec<(String, String)> = Vec::new(); // (identifier, type_name)
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            //println!("********************subcomponents: {:?}", subcomponents);
            for sub in subcomponents {
                //println!("********************sub: {:?}", sub);
                if sub.category == ComponentCategory::Process {
                    let identifier = sub.identifier.clone();
                    let type_name = if let SubcomponentClassifier::ClassifierReference(UniqueComponentClassifierReference::Implementation(impl_ref)) = &sub.classifier {
                        impl_ref.implementation_name.type_identifier.clone()
                    } else {
                        identifier.clone() // fallback
                    };
                    process_info.push((identifier, type_name));
                }
            }
        }

        // 生成 match 分支
        let mut arms: Vec<String> = Vec::new();
        for (identifier, type_name) in &process_info {
            let rust_type_name = format!("{}Process", type_name.to_lowercase());
            arms.push(format!(
                "\"{id}\" => {{\n                    let proc = {ty}::new(cpu_id);\n                    proc.start();\n                }}",
                id = identifier,
                ty = rust_type_name
            ));
        }
        // 默认分支
        arms.push("_ => { eprintln!(\"Unknown process: {}\", proc_name); }".to_string());

        // 拼接完整的 for + match 代码块
        let for_block = format!(
            "for (proc_name, cpu_id) in self.processes {{\n        match proc_name.as_str() {{\n            {}\n           }}\n        }}",
            arms.join(",\n            ")
        );

        stmts.push(Statement::Expr(Expr::Ident(for_block)));

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
    
    /// 从数据组件类型名称中查找具体的数据类型
    /// 例如：从 POS.Impl 类型中找到对应的数据类型
    fn find_data_type_from_type(&self, type_name: &str, package: &Package) -> Option<String> {
        // 在 Package 中查找组件类型
        if let Some(public_section) = &package.public_section {
            for decl in &public_section.declarations {
                if let AadlDeclaration::ComponentType(comp) = decl {
                    if comp.identifier == type_name {
                        // 找到匹配的组件类型，查找其中的数据类型属性
                        if let PropertyClause::Properties(props) = &comp.properties {
                            for prop in props {
                                if let Property::BasicProperty(bp) = prop {
                                    // 处理 Data_Model::Data_Representation 属性
                                    if bp.identifier.name.to_lowercase() == "data_model" {
                                        if let Some(property_set) = &bp.identifier.property_set {
                                            if property_set.to_lowercase() == "data_representation" {
                                                if let PropertyValue::Single(PropertyExpression::String(
                                                    StringTerm::Literal(str_val),
                                                )) = &bp.value
                                                {
                                                    // 使用 type_mappings 查找对应的类型
                                                    return self
                                                        .type_mappings
                                                        .get(&str_val.to_string())
                                                        .cloned()
                                                        .map(|t| {
                                                            if let Type::Named(name) = t {
                                                                name
                                                            } else {
                                                                str_val.to_string()
                                                            }
                                                        })
                                                        .or(Some(str_val.to_string()));
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
        }
        
        // 在私有部分也查找
        if let Some(private_section) = &package.private_section {
            for decl in &private_section.declarations {
                if let AadlDeclaration::ComponentType(comp) = decl {
                    if comp.identifier == type_name {
                        // 找到匹配的组件类型，查找其中的数据类型属性
                        if let PropertyClause::Properties(props) = &comp.properties {
                            for prop in props {
                                if let Property::BasicProperty(bp) = prop {
                                    // 处理 Data_Model::Data_Representation 属性
                                    if bp.identifier.name.to_lowercase() == "data_model" {
                                        if let Some(property_set) = &bp.identifier.property_set {
                                            if property_set.to_lowercase() == "data_representation" {
                                                if let PropertyValue::Single(PropertyExpression::String(
                                                    StringTerm::Literal(str_val),
                                                )) = &bp.value
                                                {
                                                    // 使用 type_mappings 查找对应的类型
                                                    return self
                                                        .type_mappings
                                                        .get(&str_val.to_string())
                                                        .cloned()
                                                        .map(|t| {
                                                            if let Type::Named(name) = t {
                                                                name
                                                            } else {
                                                                str_val.to_string()
                                                            }
                                                        })
                                                        .or(Some(str_val.to_string()));
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
}
