// src/aadl_to_rust/converter.rs
// aadlAST2rustAST
use super::intermediate_ast::*;
use crate::ast::aadl_ast_cj::*;
use std::{collections::HashMap, default};

// AADL到Rust中间表示的转换器
pub struct AadlConverter {
    type_mappings: HashMap<String, Type>,
    port_handlers: HashMap<String, PortHandlerConfig>,
}

#[derive(Debug)]
struct PortHandlerConfig {
    // 端口处理配置
}

impl Default for AadlConverter {
    fn default() -> Self {
        let mut type_mappings = HashMap::new();
        type_mappings.insert("Integer".to_string(), Type::Named("u32".to_string()));
        type_mappings.insert("String".to_string(), Type::Named("String".to_string()));
        type_mappings.insert("Boolean".to_string(), Type::Named("bool".to_string()));

        Self {
            type_mappings,
            port_handlers: HashMap::new(),
        }
    }
}

impl AadlConverter {
    // 主转换入口
    pub fn convert_package(&self, pkg: &Package) -> RustModule {
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
                self.convert_declaration(decl, &mut module);
            }
        }

        // 处理私有声明
        if let Some(private_section) = &pkg.private_section {
            for decl in &private_section.declarations {
                self.convert_declaration(decl, &mut module);
            }
        }

        module
    }

    fn convert_declaration(&self, decl: &AadlDeclaration, module: &mut RustModule) {
        match decl {
            AadlDeclaration::ComponentType(comp) => {
                module.items.extend(self.convert_component(comp));
            }
            AadlDeclaration::ComponentImplementation(impl_) => {
                module.items.extend(self.convert_implementation(impl_));
            }
            _ => {} // TODO:忽略其他声明类型
        }
    }

    fn convert_component(&self, comp: &ComponentType) -> Vec<Item> {
        match comp.category {
            ComponentCategory::Data => self.convert_data_component(comp),
            ComponentCategory::Thread => self.convert_thread_component(comp),
            ComponentCategory::Subprogram => self.convert_subprogram(comp),
            ComponentCategory::System => self.convert_system_component(comp),
            _ => Vec::default(), //TODO:进程还需要处理
        }
    }

    fn convert_data_component(&self, comp: &ComponentType) -> Vec<Item> {
        let target_type = self.determine_data_type(comp);
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
                if let Feature::Port(port) = feature {
                    fields.push(Field {
                        name: port.identifier.to_lowercase(),
                        ty: self.convert_port_type(&port),
                        docs: vec![format!("// Port: {} {:?}", port.identifier, port.direction)],
                        attrs: Vec::new(),
                    });
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
                    .as_ref() //.as_ref() 的作用是把 Option<T> 变成 Option<&T>。它不会取得其中值的所有权，而只是“借用”里面的值。
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

    fn convert_subprogram(&self, comp: &ComponentType) -> Vec<Item> {
        let mut items = Vec::new();

        // 检查是否是C语言绑定的子程序
        if let Some(c_func_name) = self.extract_c_function_name(comp) {
            return self.generate_c_function_wrapper(comp, &c_func_name);
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

    fn generate_c_function_wrapper(&self, comp: &ComponentType, c_func_name: &str) -> Vec<Item> {
        //获取C程序源文件文件名
        let source_files = self.extract_source_files(comp);

        let mut items = Vec::new();
        let mut functions = Vec::new();
        let mut types_to_import = std::collections::HashSet::new();

        // 处理每个参数特征
        if let FeatureClause::Items(features) = &comp.features {
            for feature in features {
                if let Feature::Port(port) = feature {
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
            _ => Vec::default(), // 默认实现
        }
    }

    fn convert_system_implementation(&self, impl_: &ComponentImplementation) -> Vec<Item> {
        let mut items = Vec::new();

        // 生成系统实现块
        items.push(Item::Impl(self.create_system_impl_block(impl_)));

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

                fields.push(Field {
                    name: sub.identifier.to_lowercase(),
                    ty: Type::Named(format!("{}Thread", type_name.to_lowercase())),
                    docs: vec![format!("// Subcomponent: {}", sub.identifier)],
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

        // 1. 创建子组件实例
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
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
                stmts.push(Statement::Let(LetStmt {
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
                    // BuilderMethod::StackSize(Box::new(Expr::Path(vec![
                    //     "self".to_string(),
                    //     var_name.clone(),
                    //     "stack_size".to_string()
                    // ],PathType::Member))),
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

    fn create_thread_run_body(&self, impl_: &ComponentImplementation) -> Block {
        let mut stmts = Vec::new();

        // 0. 线程优先级设置（如果存在priority属性）
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

        // 0.5. CPU亲和性设置（如果cpu_id > 0）
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

        // 1. 周期设置
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

        // 2. 处理子程序调用（使用 IfLet 结构）
        let subprogram_calls = self.extract_subprogram_calls(impl_);//这个函数是针对子程序有对外的连接关系，其在提取这种关系
        let mut port_handling_stmts = Vec::new();

        for (param_port_name, subprogram_name, thread_port_name, is_send) in subprogram_calls {
            // 构建 then 分支的语句
            if is_send {
                // ====================== 发送模式 ======================
                let mut then_stmts = Vec::new();

                // let mut val = 0;
                then_stmts.push(Statement::Let(LetStmt {
                    name: "val".to_string(),
                    ty: None,
                    init: Some(Expr::Literal(Literal::Int(0))),
                    ifmut: true, // 标记为可变
                }));

                // do_ping_spg::send(val);
                then_stmts.push(Statement::Expr(Expr::Call(
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

                // sender.send(val).unwrap();
                then_stmts.push(Statement::Expr(Expr::MethodCall(
                    Box::new(Expr::MethodCall(
                        Box::new(Expr::Ident("sender".to_string())),
                        "send".to_string(),
                        vec![Expr::Ident("val".to_string())],
                    )),
                    "unwrap".to_string(),
                    Vec::new(),
                )));

                // 构建 IfLet 表达式
                port_handling_stmts.push(Statement::Expr(Expr::IfLet {
                    pattern: "Some(sender)".to_string(),
                    value: Box::new(Expr::Reference(
                        Box::new(Expr::Path(
                            vec!["self".to_string(), thread_port_name],
                            PathType::Member,
                        )),
                        true,
                        false, // 不可变引用 & ,no mut
                    )),
                    then_branch: Block {
                        stmts: then_stmts,
                        expr: None,
                    },
                    else_branch: None,
                }));
            } else {
                // ====================== 接收模式 ======================
                let mut then_stmts = Vec::new();

                // let val = receiver.recv().unwrap();
                then_stmts.push(Statement::Let(LetStmt {
                    name: "val".to_string(),
                    ty: None,
                    init: Some(Expr::MethodCall(
                        Box::new(Expr::MethodCall(
                            Box::new(Expr::Ident("receiver".to_string())),
                            "recv".to_string(),
                            Vec::new(),
                        )),
                        "unwrap".to_string(),
                        Vec::new(),
                    )),
                    ifmut: false, // 不可变
                }));

                // receiver_spg::receive(val);
                then_stmts.push(Statement::Expr(Expr::Call(
                    Box::new(Expr::Path(
                        vec![subprogram_name.clone(), "receive".to_string()],
                        PathType::Namespace,
                    )),
                    vec![Expr::Ident("val".to_string())],
                )));

                // 构建 IfLet 表达式
                port_handling_stmts.push(Statement::Expr(Expr::IfLet {
                    pattern: "Some(receiver)".to_string(),
                    value: Box::new(Expr::Reference(
                        Box::new(Expr::Path(
                            vec!["self".to_string(), thread_port_name],
                            PathType::Member,
                        )),
                        true,
                        false, // 不可变引用 &
                    )),
                    then_branch: Block {
                        stmts: then_stmts,
                        expr: None,
                    },
                    else_branch: None,
                }));
            }
        }

        //20250813新增，处理无端口的子程序调用
        // 新增：处理无端口的子程序调用
        let subprogram_calls_no_ports = self.extract_subprogram_calls_no_ports(impl_);
        for subprogram_name in subprogram_calls_no_ports {
            port_handling_stmts.push(Statement::Expr(Expr::Call(
                Box::new(Expr::Path(
                    vec![subprogram_name.clone(), "execute".to_string()],
                    PathType::Namespace,
                )),
                Vec::new(),
            )));
        }

        // 3. 主循环
        stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
            stmts: vec![
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
                // 子程序调用处理块
                Statement::Expr(Expr::Block(Block {
                    stmts: port_handling_stmts,
                    expr: None,
                })),
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

        Block { stmts, expr: None }
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
                                    //这里针对“发送”连接，判断的是“源端口”的信息
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
                                    //这里针对“接收”连接，判断的是“目的端口”的信息
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
    // fn extract_subprogram_calls(&self, impl_: &ComponentImplementation) -> Vec<(String, String, String)> {
    //     let mut calls = Vec::new();

    //     if let CallSequenceClause::Items(calls_clause) = &impl_.calls {
    //         for call_clause in calls_clause {
    //             for subprocall in &call_clause.calls {
    //                 if let CalledSubprogram::Classifier(UniqueComponentClassifierReference::Type(type_ref)) = &subprocall.called {
    //                     let subprogram_name = type_ref.implementation_name.type_identifier.to_lowercase();
    //                     let call_identifier = subprocall.identifier.to_lowercase();

    //                     if let ConnectionClause::Items(connections) = &impl_.connections {
    //                         for conn in connections {
    //                             if let Connection::Parameter(param_conn) = conn {
    //                                 // 匹配参数连接的源和目标
    //                                 if let ParameterEndpoint::SubprogramCallParameter {
    //                                     call_identifier: src_call_id,
    //                                     parameter: src_param
    //                                 } = &param_conn.source {
    //                                     if *src_call_id == call_identifier {
    //                                         if let ParameterEndpoint::ComponentParameter {
    //                                             parameter: _,
    //                                             data_subcomponent: Some(dst_comp)
    //                                         } = &param_conn.destination {
    //                                             calls.push((
    //                                                 src_param.to_lowercase(),  // 参数名
    //                                                 subprogram_name.clone(),   // 子程序名
    //                                                 dst_comp.to_lowercase()    // 线程字段名
    //                                             ));
    //                                         }
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     calls
    // }
    
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

        // 收集系统内所有进程子组件的名称
        let mut process_names: Vec<String> = Vec::new();
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            for sub in subcomponents {
                if sub.category == ComponentCategory::Process {
                    process_names.push(sub.identifier.clone());
                }
            }
        }

        // 生成 match 分支
        let mut arms: Vec<String> = Vec::new();
        for proc_name in &process_names {
            let type_name = format!("{}Process", proc_name.to_lowercase());
            arms.push(format!(
                "\"{pn}\" => {{\n                    let proc = {ty}::new(cpu_id);\n                    proc.start();\n                }}",
                pn = proc_name,
                ty = type_name
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
}
