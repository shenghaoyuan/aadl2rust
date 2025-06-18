// src/aadl_to_rust/converter.rs
use crate::ast::aadl_ast_cj::*;
use super::{
    intermediate_ast::*,
};
use std::collections::HashMap;

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
                self.convert_implementation(impl_, module);
            }
            _ => {} // 忽略其他声明类型
        }
    }

    fn convert_component(&self, comp: &ComponentType) -> Vec<Item> {
        match comp.category {
            ComponentCategory::Data => self.convert_data_component(comp),
            ComponentCategory::Thread => self.convert_thread_component(comp),
            ComponentCategory::Subprogram => self.convert_subprogram(comp),
            _ => self.convert_generic_component(comp),
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
            fields: self.convert_features(&comp.features),
            generics: Vec::new(),
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            docs: self.create_component_docs(comp),
            vis: Visibility::Public,
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
                        name: port.identifier.clone(),
                        ty: self.convert_port_type(&port.port_type),
                        docs: vec![format!("/// Port: {} {:?}", port.identifier, port.direction)],
                        attrs: Vec::new(),
                    });
                }
            }
        }

        fields
    }

    fn convert_port_type(&self, port_type: &PortType) -> Type {
        match port_type {
            PortType::Data { classifier } => {
                let inner = classifier.as_ref()
                    .map(|c| self.classifier_to_type(c))
                    .unwrap_or(Type::Named("()".to_string()));
                Type::Generic("mpsc::Sender".to_string(), vec![inner])
            }
            PortType::Event => Type::Path(vec!["mpsc::Sender".to_string(), "()".to_string()]),
            PortType::EventData { classifier } => {
                let inner = classifier.as_ref()
                    .map(|c| self.classifier_to_type(c))
                    .unwrap_or(Type::Named("()".to_string()));
                Type::Generic("mpsc::Sender".to_string(), vec![inner])
            }
        }
    }

    fn classifier_to_type(&self, classifier: &PortDataTypeReference) -> Type {
        match classifier {
            PortDataTypeReference::Classifier(UniqueComponentClassifierReference::Type(r)) => {
                Type::Named(r.implementation_name.type_identifier.clone())
            }
            _ => Type::Named("()".to_string()),
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
                            ty: self.convert_port_type(&port.port_type),
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
            generics: Vec::new(),
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            docs: vec![format!("/// AADL {:?} component", comp.category)],
            vis: Visibility::Public,
        })]
    }

    fn convert_implementation(&self, impl_: &ComponentImplementation, module: &mut RustModule) {
        if let ConnectionClause::Items(connections) = &impl_.connections {
            let init_func = self.create_initialization_function(impl_.name.clone(), connections);
            module.items.push(Item::Function(init_func));
        }
    }

    fn create_initialization_function(&self, impl_name: ImplementationName, connections: &[Connection]) -> FunctionDef {
        let mut stmts = Vec::new();

        // 创建线程实例
        stmts.push(Statement::Let(LetStmt {
            name: "sender".to_string(),
            ty: None,
            init: Some(Expr::Call(
                Box::new(Expr::Path(vec!["SenderThread".to_string(), "new".to_string()])),
                Vec::new(),
            )),
        }));

        stmts.push(Statement::Let(LetStmt {
            name: "receiver".to_string(),
            ty: None,
            init: Some(Expr::Call(
                Box::new(Expr::Path(vec!["ReceiverThread".to_string(), "new".to_string()])),
                Vec::new(),
            )),
        }));

        // 建立连接
        for conn in connections {
            if let Connection::Port(PortConnection { source, destination, .. }) = conn {
                stmts.push(Statement::Expr(Expr::Ident(
                    format!("// Connect {:?} to {:?}", source, destination),
                )));
            }
        }

        // 启动线程
        stmts.push(Statement::Expr(Expr::MethodCall(
            Box::new(Expr::Path(vec!["thread".to_string(), "spawn".to_string()])),
            "unwrap".to_string(),
            vec![Expr::Closure(
                vec!["sender".to_string()],
                Box::new(Expr::MethodCall(
                    Box::new(Expr::Ident("sender".to_string())),
                    "run".to_string(),
                    Vec::new(),
                )),
            )],
        )));

        FunctionDef {
            name: format!("init_{}", impl_name.type_identifier),
            params: Vec::new(),
            return_type: Type::Unit,
            body: Block { stmts, expr: None },
            asyncness: false,
            vis: Visibility::Public,
            docs: vec![format!("/// Initialize {}", impl_name.to_string())],
            attrs: Vec::new(),
        }
    }

    fn create_component_docs(&self, comp: &ComponentType) -> Vec<String> {
        let mut docs = vec![format!(
            "/// AADL {:?}: {}",
            comp.category, comp.identifier
        )];

        if let Some(period) = self.extract_period(comp) {
            docs.push(format!("/// Period: {}ms", period));
        }

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