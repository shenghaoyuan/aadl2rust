mod ast;
use ast::aadl_ast_cj::*;
fn main() {
    let pkg = build_pingpong_package();

    // 1. 输出包名
    println!("=== Package Info ===");
    println!("Name: {}", pkg.name.to_string());//自定义的to_string（）,

    // 2. 输出导入的属性集
    for decl in &pkg.visibility_decls {
        match decl {
            VisibilityDeclaration::Import { packages, property_sets } => {
                println!("Imported packages: {:?}", packages);
                println!("Imported property sets: {:?}", property_sets);
            }
            _ => {}
        }
    }

    // 3. 输出公共声明数量
    if let Some(public_section) = &pkg.public_section {
        println!("Public section contains {} declarations:", public_section.declarations.len());
        
        // 4. 遍历所有公共声明
        for decl in &public_section.declarations {
            match decl {
                AadlDeclaration::ComponentType(c) => {
                    println!("- Component Type: {}", c.identifier);
                }
                AadlDeclaration::ComponentImplementation(impl_) => {
                    println!("- Implementation: {}.{}", 
                        impl_.name.type_identifier, 
                        impl_.name.implementation_identifier);
                }
                // 其他类型的声明...
                _ => {}
            }
        }
    }

    // 5. 输出私有声明（如果有）
    if let Some(private_section) = &pkg.private_section {
        println!("Private section contains {} declarations", private_section.declarations.len());
    }
}
fn build_pingpong_package() -> Package {
    Package {
        name: PackageName(vec!["ojr".to_string(),"pingpong".to_string(),"queued".to_string()]),
        visibility_decls: vec![VisibilityDeclaration::Import {
            packages: vec![],
            property_sets: vec!["Data_Model".to_string()],
        }],
        public_section: Some(PackageSection {
            is_public: true,
            declarations: vec![
                // System root
                AadlDeclaration::ComponentType(ComponentType {
                    category: ComponentCategory::System,
                    identifier: "root".to_string(),
                    prototypes: PrototypeClause::None,
                    features: FeatureClause::None,
                    properties: PropertyClause::ExplicitNone,
                    annexes: vec![],
                }),
                
                // System implementation root.impl
                AadlDeclaration::ComponentImplementation(ComponentImplementation {
                    category: ComponentCategory::System,
                    name: ImplementationName {
                        type_identifier: "root".to_string(),
                        implementation_identifier: "impl".to_string(),
                    },
                    prototype_bindings: None,
                    prototypes: PrototypeClause::None,
                    subcomponents: SubcomponentClause::Items(vec![
                        Subcomponent {
                            identifier: "the_cpu".to_string(),
                            category: ComponentCategory::Processor,
                            classifier: SubcomponentClassifier::ClassifierReference(
                                UniqueComponentClassifierReference::Implementation(
                                    UniqueImplementationReference {
                                        package_prefix: None,
                                        implementation_name: ImplementationName {
                                            type_identifier: "cpu".to_string(),
                                            implementation_identifier: "impl".to_string(),
                                        },
                                    }
                                )
                            ),
                            array_spec: None,
                            properties: vec![],
                        },
                        Subcomponent {
                            identifier: "the_proc".to_string(),
                            category: ComponentCategory::Process,
                            classifier: SubcomponentClassifier::ClassifierReference(
                                UniqueComponentClassifierReference::Implementation(
                                    UniqueImplementationReference {
                                        package_prefix: None,
                                        implementation_name: ImplementationName {
                                            type_identifier: "proc".to_string(),
                                            implementation_identifier: "impl".to_string(),
                                        },
                                    }
                                )
                            ),
                            array_spec: None,
                            properties: vec![],
                        },
                        Subcomponent {
                            identifier: "the_mem".to_string(),
                            category: ComponentCategory::Memory,
                            classifier: SubcomponentClassifier::ClassifierReference(
                                UniqueComponentClassifierReference::Implementation(
                                    UniqueImplementationReference {
                                        package_prefix: None,
                                        implementation_name: ImplementationName {
                                            type_identifier: "mem".to_string(),
                                            implementation_identifier: "impl".to_string(),
                                        },
                                    }
                                )
                            ),
                            array_spec: None,
                            properties: vec![],
                        },
                    ]),
                    calls: CallSequenceClause::None,
                    connections: ConnectionClause::None,
                    properties: PropertyClause::Properties(vec![
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "actual_memory_binding".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("(reference (the_mem))".to_string()))),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "actual_processor_binding".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("(reference (the_cpu))".to_string()))),
                        }),
                    ]),
                    annexes: vec![],
                }),
                
                // Processor cpu
                AadlDeclaration::ComponentType(ComponentType {
                    category: ComponentCategory::Processor,
                    identifier: "cpu".to_string(),
                    prototypes: PrototypeClause::None,
                    features: FeatureClause::None,
                    properties: PropertyClause::ExplicitNone,
                    annexes: vec![],
                }),
                
                // Processor implementation cpu.impl
                AadlDeclaration::ComponentImplementation(ComponentImplementation {
                    category: ComponentCategory::Processor,
                    name: ImplementationName {
                        type_identifier: "cpu".to_string(),
                        implementation_identifier: "impl".to_string(),
                    },
                    prototype_bindings: None,
                    prototypes: PrototypeClause::None,
                    subcomponents: SubcomponentClause::None,
                    calls: CallSequenceClause::None,
                    connections: ConnectionClause::None,
                    properties: PropertyClause::ExplicitNone,
                    annexes: vec![],
                }),
                
                // Process proc
                AadlDeclaration::ComponentType(ComponentType {
                    category: ComponentCategory::Process,
                    identifier: "proc".to_string(),
                    prototypes: PrototypeClause::None,
                    features: FeatureClause::None,
                    properties: PropertyClause::ExplicitNone,
                    annexes: vec![],
                }),
                
                // Process implementation proc.impl
                AadlDeclaration::ComponentImplementation(ComponentImplementation {
                    category: ComponentCategory::Process,
                    name: ImplementationName {
                        type_identifier: "proc".to_string(),
                        implementation_identifier: "impl".to_string(),
                    },
                    prototype_bindings: None,
                    prototypes: PrototypeClause::None,
                    subcomponents: SubcomponentClause::Items(vec![
                        Subcomponent {
                            identifier: "the_sender".to_string(),
                            category: ComponentCategory::Thread,
                            classifier: SubcomponentClassifier::ClassifierReference(
                                UniqueComponentClassifierReference::Implementation(
                                    UniqueImplementationReference {
                                        package_prefix: None,
                                        implementation_name: ImplementationName {
                                            type_identifier: "sender".to_string(),
                                            implementation_identifier: "impl".to_string(),
                                        },
                                    }
                                )
                            ),
                            array_spec: None,
                            properties: vec![],
                        },
                        Subcomponent {
                            identifier: "the_receiver".to_string(),
                            category: ComponentCategory::Thread,
                            classifier: SubcomponentClassifier::ClassifierReference(
                                UniqueComponentClassifierReference::Implementation(
                                    UniqueImplementationReference {
                                        package_prefix: None,
                                        implementation_name: ImplementationName {
                                            type_identifier: "receiver".to_string(),
                                            implementation_identifier: "impl".to_string(),
                                        },
                                    }
                                )
                            ),
                            array_spec: None,
                            properties: vec![],
                        },
                    ]),
                    calls: CallSequenceClause::None,
                    connections: ConnectionClause::Items(vec![
                        Connection::Port(PortConnection {
                            source: PortEndpoint::SubcomponentPort {
                                subcomponent: "the_sender".to_string(),
                                port: "p".to_string(),
                            },
                            destination: PortEndpoint::SubcomponentPort {
                                subcomponent: "the_receiver".to_string(),
                                port: "p".to_string(),
                            },
                            connection_direction: ConnectionSymbol::Direct,
                        }),
                    ]),
                    properties: PropertyClause::ExplicitNone,
                    annexes: vec![],
                }),
                
                // Memory mem
                AadlDeclaration::ComponentType(ComponentType {
                    category: ComponentCategory::Memory,
                    identifier: "mem".to_string(),
                    prototypes: PrototypeClause::None,
                    features: FeatureClause::None,
                    properties: PropertyClause::ExplicitNone,
                    annexes: vec![],
                }),
                
                // Memory implementation mem.impl
                AadlDeclaration::ComponentImplementation(ComponentImplementation {
                    category: ComponentCategory::Memory,
                    name: ImplementationName {
                        type_identifier: "mem".to_string(),
                        implementation_identifier: "impl".to_string(),
                    },
                    prototype_bindings: None,
                    prototypes: PrototypeClause::None,
                    subcomponents: SubcomponentClause::None,
                    calls: CallSequenceClause::None,
                    connections: ConnectionClause::None,
                    properties: PropertyClause::Properties(vec![
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Memory_Size".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            //value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("200 KByte".to_string()))),
                            value: PropertyValue::Single(
                                PropertyExpression::Integer(
                                    SignedIntergerOrConstant::Real(
                                        SignedInteger{
                                        sign: Some(Sign::Plus),
                                        value: 200,
                                        unit: Some("KByte".to_string()),
                                        }
                                    )
                                )
                            )
                        }),
                    ]),
                    annexes: vec![],
                }),
                
                // Thread sender
                AadlDeclaration::ComponentType(ComponentType {
                    category: ComponentCategory::Thread,
                    identifier: "sender".to_string(),
                    prototypes: PrototypeClause::None,
                    features: FeatureClause::Items(vec![
                        Feature::Port(PortSpec {
                            identifier: "p".to_string(),
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
                                        }
                                    )
                                )),
                            },
                        }),
                    ]),
                    properties: PropertyClause::Properties(vec![
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Dispatch_Protocol".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("Periodic".to_string()))),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Compute_Execution_Time".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("0 ms .. 1 ms".to_string()))),
                        }),
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
                                    value: 2000,
                                    unit: Some("Ms".to_string()),
                                })
                            )),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Priority".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::Integer(
                                SignedIntergerOrConstant::Real(SignedInteger {
                                    sign: None,
                                    value: 5,
                                    unit: None,  // 优先级通常是无单位的纯数字
                                })
                            )),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Source_Data_Size".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::Integer(
                                SignedIntergerOrConstant::Real(SignedInteger {
                                    sign: None,
                                    value: 40000,
                                    unit: Some("bytes".to_string()),
                                })
                            )),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Source_Stack_Size".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::Integer(
                                SignedIntergerOrConstant::Real(SignedInteger {
                                    sign: None,
                                    value: 40000,
                                    unit: Some("bytes".to_string()),
                                })
                            )),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Source_Code_Size".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::Integer(
                                SignedIntergerOrConstant::Real(SignedInteger {
                                    sign: None,
                                    value: 40,
                                    unit: Some("bytes".to_string()),
                                })
                            )),
                        }),
                    ]),
                    annexes: vec![],
                }),
                
                // Thread implementation sender.impl
                AadlDeclaration::ComponentImplementation(ComponentImplementation {
                    category: ComponentCategory::Thread,
                    name: ImplementationName {
                        type_identifier: "sender".to_string(),
                        implementation_identifier: "impl".to_string(),
                    },
                    prototype_bindings: None,
                    prototypes: PrototypeClause::None,
                    subcomponents: SubcomponentClause::None,
                    calls: CallSequenceClause::Items(vec![
                        CallSequence {
                            identifier: "call".to_string(),
                            calls: vec![
                                SubprogramCall {
                                    identifier: "c".to_string(),
                                    called: CalledSubprogram::Classifier(
                                        UniqueComponentClassifierReference::Type(
                                            UniqueImplementationReference {
                                                package_prefix: None,
                                                implementation_name: ImplementationName {
                                                    type_identifier: "sender_spg".to_string(),
                                                    implementation_identifier: "".to_string(),
                                                },
                                            }
                                        )
                                    ),
                                    properties: vec![],
                                },
                            ],
                            properties: vec![],
                            in_modes: None,
                        },
                    ]),
                    connections: ConnectionClause::Items(vec![
                        Connection::Parameter(ParameterConnection {
                            source: ParameterEndpoint::SubprogramCallParameter {
                                call_identifier: "c".to_string(),
                                parameter: "result".to_string(),
                            },
                            destination: ParameterEndpoint::ComponentParameter {
                                parameter: "p".to_string(),
                                data_subcomponent: None,
                            },
                            connection_direction: ConnectionSymbol::Direct,
                        }),
                    ]),
                    properties: PropertyClause::Properties(vec![
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Compute_Entrypoint_Call_Sequence".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("reference (call)".to_string()))),
                        }),
                    ]),
                    annexes: vec![],
                }),
                
                // Subprogram sender_spg
                AadlDeclaration::ComponentType(ComponentType {
                    category: ComponentCategory::Subprogram,
                    identifier: "sender_spg".to_string(),
                    prototypes: PrototypeClause::None,
                    features: FeatureClause::Items(vec![
                        Feature::Port(PortSpec {
                            identifier: "result".to_string(),
                            direction: PortDirection::Out,
                            port_type: PortType::Data {
                                classifier: Some(PortDataTypeReference::Classifier(
                                    UniqueComponentClassifierReference::Type(
                                        UniqueImplementationReference {
                                            package_prefix: None,
                                            implementation_name: ImplementationName {
                                                type_identifier: "Integer".to_string(),
                                                implementation_identifier: "".to_string(),
                                            },
                                        }
                                    )
                                )),
                            },
                        }),
                    ]),
                    properties: PropertyClause::Properties(vec![
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "source_name".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("\"PingPong.Send\"".to_string()))),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "source_language".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("(c)".to_string()))),
                        }),
                    ]),
                    annexes: vec![],
                }),
                
                // Thread receiver
                AadlDeclaration::ComponentType(ComponentType {
                    category: ComponentCategory::Thread,
                    identifier: "receiver".to_string(),
                    prototypes: PrototypeClause::None,
                    features: FeatureClause::Items(vec![
                        Feature::Port(PortSpec {
                            identifier: "p".to_string(),
                            direction: PortDirection::In,
                            port_type: PortType::EventData {
                                classifier: Some(PortDataTypeReference::Classifier(
                                    UniqueComponentClassifierReference::Type(
                                        UniqueImplementationReference {
                                            package_prefix: None,
                                            implementation_name: ImplementationName {
                                                type_identifier: "Integer".to_string(),
                                                implementation_identifier: "".to_string(),
                                            },
                                        }
                                    )
                                )),
                            },
                        }),
                    ]),
                    properties: PropertyClause::Properties(vec![
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Dispatch_Protocol".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("Periodic".to_string()))),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Compute_Execution_Time".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("0 ms .. 1 ms".to_string()))),
                        }),
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
                                    unit: Some("Ms".to_string()),
                                })
                            )),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Priority".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::Integer(
                                SignedIntergerOrConstant::Real(SignedInteger {
                                    sign: None,
                                    value: 10,
                                    unit: None,  // 优先级通常是无单位的纯数字
                                })
                            )),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Source_Data_Size".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::Integer(
                                SignedIntergerOrConstant::Real(SignedInteger {
                                    sign: None,
                                    value: 40000,
                                    unit: Some("bytes".to_string()),
                                })
                            )),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Source_Stack_Size".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::Integer(
                                SignedIntergerOrConstant::Real(SignedInteger {
                                    sign: None,
                                    value: 40000,
                                    unit: Some("bytes".to_string()),
                                })
                            )),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Source_Code_Size".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::Integer(
                                SignedIntergerOrConstant::Real(SignedInteger {
                                    sign: None,
                                    value: 40,
                                    unit: Some("bytes".to_string()),
                                })
                            )),
                        }),
                    ]),
                    annexes: vec![],
                }),
                
                // Thread implementation receiver.impl
                AadlDeclaration::ComponentImplementation(ComponentImplementation {
                    category: ComponentCategory::Thread,
                    name: ImplementationName {
                        type_identifier: "receiver".to_string(),
                        implementation_identifier: "impl".to_string(),
                    },
                    prototype_bindings: None,
                    prototypes: PrototypeClause::None,
                    subcomponents: SubcomponentClause::None,
                    calls: CallSequenceClause::Items(vec![
                        CallSequence {
                            identifier: "call".to_string(),
                            calls: vec![
                                SubprogramCall {
                                    identifier: "c".to_string(),
                                    called: CalledSubprogram::Classifier(
                                        UniqueComponentClassifierReference::Type(
                                            UniqueImplementationReference {
                                                package_prefix: None,
                                                implementation_name: ImplementationName {
                                                    type_identifier: "receiver_spg".to_string(),
                                                    implementation_identifier: "".to_string(),
                                                },
                                            }
                                        )
                                    ),
                                    properties: vec![],
                                },
                            ],
                            properties: vec![],
                            in_modes: None,
                        },
                    ]),
                    connections: ConnectionClause::Items(vec![
                        Connection::Parameter(ParameterConnection {
                            source: ParameterEndpoint::ComponentParameter {
                                parameter: "p".to_string(),
                                data_subcomponent: None,
                            },
                            destination: ParameterEndpoint::SubprogramCallParameter {
                                call_identifier: "c".to_string(),
                                parameter: "input".to_string(),
                            },
                            connection_direction: ConnectionSymbol::Direct,
                        }),
                    ]),
                    properties: PropertyClause::Properties(vec![
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Compute_Entrypoint_Call_Sequence".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("reference (call)".to_string()))),
                        }),
                    ]),
                    annexes: vec![],
                }),
                
                // Subprogram receiver_spg
                AadlDeclaration::ComponentType(ComponentType {
                    category: ComponentCategory::Subprogram,
                    identifier: "receiver_spg".to_string(),
                    prototypes: PrototypeClause::None,
                    features: FeatureClause::Items(vec![
                        Feature::Port(PortSpec {
                            identifier: "input".to_string(),
                            direction: PortDirection::In,
                            port_type: PortType::Data {
                                classifier: Some(PortDataTypeReference::Classifier(
                                    UniqueComponentClassifierReference::Type(
                                        UniqueImplementationReference {
                                            package_prefix: None,
                                            implementation_name: ImplementationName {
                                                type_identifier: "Integer".to_string(),
                                                implementation_identifier: "".to_string(),
                                            },
                                        }
                                    )
                                )),
                            },
                        }),
                    ]),
                    properties: PropertyClause::Properties(vec![
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "source_name".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("\"PingPong.Receive\"".to_string()))),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "source_language".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("(c)".to_string()))),
                        }),
                    ]),
                    annexes: vec![],
                }),
                
                // Data Integer
                AadlDeclaration::ComponentType(ComponentType {
                    category: ComponentCategory::Data,
                    identifier: "Integer".to_string(),
                    prototypes: PrototypeClause::None,
                    features: FeatureClause::None,
                    properties: PropertyClause::Properties(vec![
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: None,
                                name: "Source_Name".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("\"usercode.PingPongType\"".to_string()))),
                        }),
                        Property::BasicProperty(BasicPropertyAssociation {
                            identifier: PropertyIdentifier {
                                property_set: Some("Data_Model".to_string()),
                                name: "Initial_Value".to_string(),
                            },
                            operator: PropertyOperator::Assign,
                            is_constant: false,
                            value: PropertyValue::Single(PropertyExpression::String(StringTerm::Literal("\"new PingPongType()\"".to_string()))),
                        }),
                    ]),
                    annexes: vec![],
                }),
            ],
        }),
        private_section: None,
        properties: PropertyClause::ExplicitNone,
    }
}

