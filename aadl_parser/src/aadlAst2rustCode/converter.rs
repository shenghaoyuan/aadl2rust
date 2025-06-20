// src/aadl_to_rust/converter.rs
use crate::ast::aadl_ast_cj::*;
use super::{
    intermediate_ast::*,
};
use std::{collections::HashMap, default};

/// AADL到Rust中间表示的转换器
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
        type_mappings.insert("Integer".to_string(), Type::Named("i32".to_string()));
        type_mappings.insert("String".to_string(), Type::Named("String".to_string()));
        type_mappings.insert("Boolean".to_string(), Type::Named("bool".to_string()));

        Self {
            type_mappings,
            port_handlers: HashMap::new(),
        }
    }
}

impl AadlConverter {
    /// 主转换入口
    pub fn convert_package(&self, pkg: &Package) -> RustModule {
        let mut module = RustModule {
            name: pkg.name.0.join("_").to_lowercase(),
            docs: vec![format!("/// Auto-generated from AADL package: {}", pkg.name.0.join("::"))],
            ..Default::default()
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
            _ => Vec::default(), //TODO:进程、系统还需要处理
        }
    }

    fn convert_data_component(&self, comp: &ComponentType) -> Vec<Item> {
        let target_type = self.determine_data_type(comp);
        vec![Item::TypeAlias(TypeAlias {
            name: comp.identifier.clone(),
            target: target_type,
            vis: Visibility::Public,
            docs: vec![format!("/// AADL Data Type: {}", comp.identifier)],
        })]
    }

    fn determine_data_type(&self, comp: &ComponentType) -> Type {
        if let PropertyClause::Properties(props) = &comp.properties {
            for prop in props {
                if let Property::BasicProperty(bp) = prop {
                    if bp.identifier.name == "Data_Representation" {
                        if let PropertyValue::Single(PropertyExpression::String(StringTerm::Literal(str_val))) = &bp.value {
                            
                                return self.type_mappings.get(&str_val.to_string())
                                    .cloned()
                                    .unwrap_or(Type::Named("()".to_string()));
                            
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
        let struct_def = StructDef {
            name: format!("{}Thread", comp.identifier),
            fields: self.convert_features(&comp.features),  //特征列表
            properties: self.convert_properties(comp),      // 属性列表
            generics: Vec::new(),
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            docs: self.create_component_docs(comp),
            vis: Visibility::Public, //默认public
        };
        items.push(Item::Struct(struct_def));

        // 2. 实现块
        if let Some(impl_block) = self.create_thread_impl(comp) {
            items.push(Item::Impl(impl_block));
        }

        items
    }

    fn convert_features(&self, features: &FeatureClause) -> Vec<Field> {
        let mut fields = Vec::new();
        
        if let FeatureClause::Items(feature_items) = features {
            for feature in feature_items {
                if let Feature::Port(port) = feature {
                    fields.push(Field {
                        name: port.identifier.clone().to_lowercase(),
                        ty: self.convert_port_type(&port),
                        docs: vec![format!("/// Port: {} {:?}", port.identifier, port.direction)],
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
                classifier.as_ref()
                    .map(|c| self.classifier_to_type(c))
                    .unwrap_or(Type::Named("()".to_string()))
            }
            PortType::Event => Type::Named("()".to_string()), // 事件端口固定使用单元类型
        };

        // 组合成最终类型
        Type::Generic(channel_type.to_string(), vec![inner_type])
    }

    fn classifier_to_type(&self, classifier: &PortDataTypeReference) -> Type {
        match classifier {
            PortDataTypeReference::Classifier(
                    UniqueComponentClassifierReference::Type(ref type_ref)
                ) => {
                    // 优先查找我们所自定义类型映射规则
                    self.type_mappings.get(&type_ref.implementation_name.type_identifier)
                        .cloned()
                        .unwrap_or_else(|| {
                            Type::Named(type_ref.implementation_name.type_identifier.clone())
                        })
                    
                }
            _ => Type::Named("()".to_string()),
        }
    }

    /// 转换AADL属性为Property列表
    fn convert_properties(&self, comp: &ComponentType) -> Vec<StruProperty> {
        // 通用属性转换方法
        let mut result = Vec::new();
        
        if let PropertyClause::Properties(props) = &comp.properties {
            for prop in props {
                if let Some(converted) = self.convert_single_property(prop) {
                    result.push(converted);
                }
            }
        }
        
        result
        // let mut properties = Vec::new();

        // // 1. 转换周期属性
        // if let Some(period) = self.extract_period(comp) {
        //     properties.push(StruProperty {
        //         name: "period".to_string(),
        //         value: StruPropertyValue::Duration(period),
        //         docs: vec![format!("/// 执行周期: {}ms", period)], //TODO:暂时写死是毫秒
        //     });
        // }

        // // 3. TODO:可扩展其他属性转换...
        // properties
    }
    /// 转换单个属性
    fn convert_single_property(&self, prop: &Property) -> Option<StruProperty> {
        let Property::BasicProperty(bp) = prop else {
            return None; // 跳过非基础属性
        };

        let docs = vec![format!("/// AADL属性: {}", bp.identifier.name)];
        
        Some(StruProperty {
            name: bp.identifier.name.clone(),
            value: self.parse_property_value(&bp.value)?,
            docs,
        })
    }

    /// 解析AADL属性值到Rust类型
    fn parse_property_value(&self, value: &PropertyValue) -> Option<StruPropertyValue> {
        match value {
            PropertyValue::Single(expr) => self.parse_property_expression(expr),
            _ => None, // 忽略其他复杂属性
        }
    }

    /// 解析属性表达式为StruPropertyValue
    fn parse_property_expression(&self, expr: &PropertyExpression) -> Option<StruPropertyValue> {
        match expr {
            // 基础类型处理
            PropertyExpression::Boolean(boolean_term) => {
                self.parse_boolean_term(boolean_term)
            }
            PropertyExpression::Real(real_term) => {
                self.parse_real_term(real_term)
            }
            PropertyExpression::Integer(integer_term) => {
                self.parse_integer_term(integer_term)
            }
            PropertyExpression::String(string_term) => {
                self.parse_string_term(string_term)
            }
            
            // 范围类型处理
            PropertyExpression::IntegerRange(range_term) => {
                Some(StruPropertyValue::Range(
                    range_term.lower.value.parse().ok()?,
                    range_term.upper.value.parse().ok()?,
                    range_term.lower.unit.clone()
                ))
            }
            
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
    

    fn create_thread_impl(&self, comp: &ComponentType) -> Option<ImplBlock> {
        let period = self.extract_period(comp)?;

        Some(ImplBlock {
            target: Type::Named(format!("{}Thread", comp.identifier)),
            generics: Vec::new(),
            items: vec![ImplItem::Method(FunctionDef {
                name: "run".to_string(),
                params: vec![Param {
                    name: "self".to_string(),
                    ty: Type::Reference(
                        Box::new(Type::Named(format!("{}Thread", comp.identifier))),
                        true,
                    ),
                }],
                return_type: Type::Unit,
                body: self.create_thread_body(period),
                asyncness: true,
                vis: Visibility::Public,
                docs: vec![
                    "/// Thread execution entry point".to_string(),
                    format!("/// Period: {}ms", period),
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
                    if bp.identifier.name == "Period" {
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
                    name: "interval".to_string(),
                    ty: Some(Type::Path(vec!["tokio".to_string(), "time".to_string(), "Interval".to_string()])),
                    init: Some(Expr::Call(
                        Box::new(Expr::Path(vec!["tokio".to_string(), "time".to_string(), "interval".to_string()])),
                        vec![Expr::Call(
                            Box::new(Expr::Path(vec!["Duration".to_string(), "from_millis".to_string()])),
                            vec![Expr::Literal(Literal::Int(period_ms as i64))],
                        )],
                    )),
                }),
                Statement::Expr(Expr::Loop(
                    Box::new(Block {
                        stmts: vec![
                            Statement::Expr(Expr::MethodCall(
                                Box::new(Expr::Ident("interval".to_string())),
                                "tick".to_string(),
                                Vec::new(),
                            )),
                            Statement::Expr(Expr::Await(
                                Box::new(Expr::Ident("_".to_string())),
                            )),
                        ],
                        expr: None,
                    }),
                )),
            ],
            expr: None,
        }
    }

    fn convert_subprogram(&self, comp: &ComponentType) -> Vec<Item> {
        let mut items = Vec::new();

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
                            stmts: vec![Statement::Expr(Expr::Ident(format!("// Handle port: {}", port.identifier)))],
                            expr: None,
                        },
                        asyncness: matches!(port.port_type, PortType::Event | PortType::EventData { .. }),
                        vis: Visibility::Public,
                        docs: vec![
                            format!("/// Port handler for {}", port.identifier),
                            format!("/// Direction: {:?}", port.direction),
                        ],
                        attrs: Vec::new(),
                    }));
                }
            }
        }

        items
    }

    fn convert_generic_component(&self, comp: &ComponentType) -> Vec<Item> {
        vec![Item::Struct(StructDef {
            name: comp.identifier.clone(),
            fields: Vec::new(),
            properties: Vec::new(),
            generics: Vec::new(),
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            docs: vec![format!("/// AADL {:?} component", comp.category)],
            vis: Visibility::Public,
        })]
    }

    fn convert_implementation(&self, impl_: &ComponentImplementation) -> Vec<Item> {
        match impl_.category {
            ComponentCategory::Process => self.convert_process_implementation(impl_),
            _ => Vec::default(), // 默认实现
        }
    }

    fn convert_process_implementation(&self, impl_: &ComponentImplementation) -> Vec<Item> {
        let mut items = Vec::new();
        
        // 1. 生成进程结构体
        let struct_def = StructDef {
            name: format!{"{}Process",impl_.name.type_identifier.to_lowercase()},
            fields: self.get_process_fields(impl_),
            properties: Vec::new(),
            generics: Vec::new(),
            derives: vec!["Debug".to_string()],
            docs: vec![
                format!("/// Process implementation: {}", impl_.name.type_identifier),
                "/// Auto-generated from AADL".to_string(),
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
                        UniqueComponentClassifierReference::Implementation(unirf)) => {
                        // 直接使用子组件标识符 + "Thread"
                        format!("{}", unirf.implementation_name.type_identifier.to_lowercase())
                    },
                    _ => "UnsupportedComponent".to_string()
                };

                fields.push(Field {
                    name: sub.identifier.clone().to_lowercase(),
                    ty: Type::Named(format!("{}Thread", type_name)),
                    docs: vec![format!("/// Subcomponent: {}", sub.identifier)],
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
            params: Vec::new(),
            return_type: Type::Named("Self".to_string()),
            body: self.create_process_new_body(impl_),
            asyncness: false,
            vis: Visibility::Public,
            docs: vec!["/// Creates a new process instance".to_string()],
            attrs: Vec::new(),
        }));
        
        // 添加start方法
        items.push(ImplItem::Method(FunctionDef {
            name: "start".to_string(),
            params: vec![Param {
                name: "self".to_string(),
                ty: Type::Reference(Box::new(Type::Named("Self".to_string())), true),
            }],
            return_type: Type::Unit,
            body: self.create_process_start_body(impl_),
            asyncness: false,
            vis: Visibility::Public,
            docs: vec!["/// Starts all threads in the process".to_string()],
            attrs: Vec::new(),
        }));
        
        ImplBlock {
            target: Type::Named(format!{"{}Process",impl_.name.type_identifier.clone().to_lowercase()}),
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
                        UniqueComponentClassifierReference::Type(type_ref)
                    ) => type_ref.implementation_name.type_identifier.clone(),
                    SubcomponentClassifier::ClassifierReference(
                        UniqueComponentClassifierReference::Implementation(impl_ref)
                    ) => impl_ref.implementation_name.type_identifier.clone(),
                    SubcomponentClassifier::Prototype(_) => {
                        "UnsupportedPrototype".to_string()
                    }
                };

                let var_name = sub.identifier.clone().to_lowercase();
                stmts.push(Statement::Let(LetStmt {
                    name: format!("mut {}", var_name),
                    ty: Some(Type::Named(format!("{}Thread", type_name))),
                    init: Some(Expr::Call(
                        Box::new(Expr::Path(vec![
                            format!("{}Thread", type_name),
                            "new".to_string()
                        ])),
                        Vec::new(),
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
            subcomponents.iter()
                .map(|s| s.identifier.clone().to_lowercase())
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::new()
        };
        
        stmts.push(Statement::Expr(Expr::Ident(
            format!("Self {{ {} }}", fields)
        )));
        
        Block {
            stmts,
            expr: None,
        }
    }

    fn create_process_start_body(&self, impl_: &ComponentImplementation) -> Block {
        let mut stmts = Vec::new();
        
        if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
            for sub in subcomponents {
                let var_name = sub.identifier.clone().to_lowercase();
                stmts.push(Statement::Expr(Expr::MethodCall(
                    Box::new(Expr::Path(vec!["thread".to_string(), "Builder".to_string(), "new".to_string()])),
                    "spawn".to_string(),
                    vec![
                        Expr::Closure(
                            vec![var_name.clone()],
                            Box::new(Expr::MethodCall(
                                Box::new(Expr::Ident(var_name)),
                                "run".to_string(),
                                Vec::new(),
                            )),
                        )
                    ],
                )));
            }
        }
        
        Block {
            stmts,
            expr: None,
        }
    }

    fn create_channel_connection(&self, conn: &PortConnection) -> Vec<Statement> {
        let mut stmts = Vec::new();
        
        // 这里简化处理，实际应根据连接类型创建适当的channel
        stmts.push(Statement::Let(LetStmt {
            name: "channel".to_string(),
            ty: None, //这里的通道类型由编译器自动推导
            init: Some(Expr::Call(
                Box::new(Expr::Path(vec!["mpsc".to_string(), "channel".to_string()])),
                Vec::new(),
            )),
        }));
        
        // 处理源端和目标端
        match (&conn.source, &conn.destination) {
            (
                PortEndpoint::SubcomponentPort { subcomponent: src_comp, port: src_port },
                PortEndpoint::SubcomponentPort { subcomponent: dst_comp, port: dst_port }
            ) => {
                // 分配发送端
                stmts.push(Statement::Expr(Expr::MethodCall(
                    Box::new(Expr::Ident(format!("{}.{}", src_comp, src_port))),
                    "send".to_string(),  //这个关键字的固定的，例如cnx: port the_sender.p -> the_receiver.p;，前者发送，后者接收
                    vec![Expr::Ident("channel.0".to_string())],
                )));
                
                // 分配接收端
                stmts.push(Statement::Expr(Expr::MethodCall(
                    Box::new(Expr::Ident(format!("{}.{}", dst_comp, dst_port))),
                    "receive".to_string(),
                    vec![Expr::Ident("channel.1".to_string())],
                )));
            }
            (
                PortEndpoint::ComponentPort(port_name),
                PortEndpoint::SubcomponentPort { subcomponent: dst_comp, port: dst_port }
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
                stmts.push(Statement::Expr(Expr::Ident(
                    format!("// TODO: Unsupported connection type: {:?} -> {:?}", conn.source, conn.destination)
                )));
            }
        }
        
        stmts
    }

    fn create_component_docs(&self, comp: &ComponentType) -> Vec<String> {
        let mut docs = vec![format!(
            "/// AADL {:?}: {}",
            comp.category, comp.identifier
        )];

        docs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::aadl_ast_cj::*;

    fn create_test_package() -> Package {
        Package {
            name: PackageName(vec!["test".to_string()]),
            visibility_decls: Vec::new(),
            public_section: Some(PackageSection {
                is_public: true,
                declarations: vec![
                    AadlDeclaration::ComponentType(ComponentType {
                        category: ComponentCategory::Thread,
                        identifier: "sender".to_string(),
                        prototypes: PrototypeClause::None,
                        features: FeatureClause::Items(vec![
                            Feature::Port(PortSpec {
                                identifier: "out_port".to_string(),
                                direction: PortDirection::Out,
                                port_type: PortType::EventData {
                                    classifier: Some(PortDataTypeReference::Classifier(
                                        UniqueComponentClassifierReference::Type(
                                            UniqueImplementationReference {
                                                package_prefix: None,
                                                implementation_name: ImplementationName {
                                                    type_identifier: "Integer".to_string(),
                                                    implementation_identifier: "".to_string(),
                                                },
                                            },
                                        ),
                                    )),
                                },
                            }),
                        ]),
                        properties: PropertyClause::Properties(vec![
                            Property::BasicProperty(BasicPropertyAssociation {
                                identifier: PropertyIdentifier {
                                    property_set: None,
                                    name: "Period".to_string(),
                                },
                                operator: PropertyOperator::Assign,
                                is_constant: false,
                                value: PropertyValue::Single(PropertyExpression::Integer(
                                    SignedIntergerOrConstant::Real(SignedInteger {
                                        sign: None,
                                        value: 1000,
                                        unit: None,
                                    }),
                                )),
                            }),
                        ]),
                        annexes: Vec::new(),
                    }),
                ],
            }),
            private_section: None,
            properties: PropertyClause::ExplicitNone,
        }
    }

    #[test]
    fn test_convert_thread_component() {
        let converter = AadlConverter::default();
        let pkg = create_test_package();
        let module = converter.convert_package(&pkg);

        assert_eq!(module.name, "test");
        assert!(module.items.iter().any(|i| matches!(i, Item::Struct(_))));
        assert!(module.items.iter().any(|i| matches!(i, Item::Impl(_))));
    }
}