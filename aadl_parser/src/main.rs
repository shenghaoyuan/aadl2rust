/* 

pub mod aadlight_parser;

use aadlight_parser::AADLParser;
use pest::{iterators::Pair, Parser};
use std::fs;

// fn print_pair(pair: Pair<aadlight_parser::Rule>, indent: usize) {
//     println!(
//         "{}{:?}: {:?}",
//         "  ".repeat(indent),
//         pair.as_rule(),
//         pair.as_str()
//     );
//     for inner in pair.into_inner() {
//         print_pair(inner, indent + 1);
//     }
// }

fn print_pair(pair: Pair<aadlight_parser::Rule>, indent: usize) {
    // 跳过空白和注释节点
    match pair.as_rule() {
        aadlight_parser::Rule::WHITESPACE | aadlight_parser::Rule::COMMENT => return,
        _ => {
            // 获取位置信息
            let span = pair.as_span();
            let (start_line, _) = span.start_pos().line_col();
            let (end_line, _) = span.end_pos().line_col();
            
            // 格式化输出
            let content = pair.as_str().trim();
            let truncated_content = if content.len() > 30 {
                format!("{}...", &content[..30])
            } else {
                content.to_string()
            };
            
            println!(
                "{}{:<25} {:<30} (lines {}-{})",
                "  ".repeat(indent),
                format!("{:?}:", pair.as_rule()),
                truncated_content,
                start_line,
                end_line
            );

            // 递归处理子节点
            for inner in pair.into_inner() {
                print_pair(inner, indent + 1);
            }
        }
    }
}
fn main() {
    let path = "pingpong.aadl";
    let aadl_input = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("读取文件失败: {}", err);
            return;
        }
    };
    match AADLParser::parse(aadlight_parser::Rule::file, &aadl_input) {
        Ok(pairs) => {
            for pair in pairs {
                print_pair(pair, 0);
            }
        }
        Err(e) => {
            eprintln!("解析失败: {}", e);
        }
    }
}
*/
pub mod aadlight_parser;
mod ast;

use ast::aadl_ast_cj::*;
use aadlight_parser::AADLParser;
use pest::{iterators::Pair, Parser};
use std::fs;

// 辅助函数：从 Pair 中提取标识符
fn extract_identifier(pair: Pair<aadlight_parser::Rule>) -> String {
    pair.as_str().trim().to_string()
}

// 辅助函数：从 Pair 中提取包名
fn extract_package_name(pair: Pair<aadlight_parser::Rule>) -> PackageName {
    PackageName(
        pair.as_str()
            .split("::")
            .map(|s| s.trim().to_string())
            .collect(),
    )
}

// 主转换结构体
struct AADLTransformer;

impl AADLTransformer {
    fn transform_file(pairs: Vec<Pair<aadlight_parser::Rule>>) -> Vec<Package> {
        let mut packages = Vec::new();
        
        // for pair in pairs {
        //     println!("处理规则: {:?}, 内容: {}", pair.as_rule(), pair.as_str());
        //     if pair.as_rule() == aadlight_parser::Rule::package_declaration { //检查是否是package_declaration规则
        //         if let Some(pkg) = Self::transform_package(pair) {
        //             packages.push(pkg);
        //         }
        //     }
        // }
        for pair in pairs {
            //println!("顶层规则: {:?}, 内容: {}", pair.as_rule(), pair.as_str());
            println!("  内部规则: {:?}", pair.as_rule());

            // 进入 file 规则内部，提取出真正的 package_declaration
            if pair.as_rule() == aadlight_parser::Rule::file {
                for inner in pair.into_inner() {
                    //println!("  内部规则: {:?}, 内容: {}", inner.as_rule(), inner.as_str());
                    println!("  内部规则: {:?}", inner.as_rule());
                    if inner.as_rule() == aadlight_parser::Rule::package_declaration {
                        if let Some(pkg) = Self::transform_package(inner) {
                            packages.push(pkg);
                        }
                    }
                }
            }
        }


        packages
    }
    
    fn transform_package(pair: Pair<aadlight_parser::Rule>) -> Option<Package> {
        let mut inner_iter = pair.into_inner();
        
        // 第一个元素应该是"package"关键字
        let _ = inner_iter.next();
        
        let package_name = extract_package_name(inner_iter.next().unwrap());
        let mut visibility_decls = Vec::new();
        let mut public_section = None;
        let mut private_section = None;
        let mut properties = PropertyClause::ExplicitNone;
        
        while let Some(inner) = inner_iter.next() {
            //println!("  内部规则: {:?}, 内容: {}", inner.as_rule(), inner.as_str());
            match inner.as_rule() {
                aadlight_parser::Rule::visibility_declarations => {
                    visibility_decls.push(Self::transform_visibility_declaration(inner));
                }
                aadlight_parser::Rule::package_sections => {
                    let section = Self::transform_package_section(inner);
                    if section.is_public {
                        public_section = Some(section);
                    } else {
                        private_section = Some(section);
                    }
                }
                _ => {}
            }
        }
        
        Some(Package {
            name: package_name,
            visibility_decls,
            public_section,
            private_section,
            properties,
        })
    }
    
    fn transform_visibility_declaration(pair: Pair<aadlight_parser::Rule>) -> VisibilityDeclaration {
        // 首先收集所有内部项到向量中，这样我们可以多次遍历
        let items: Vec<_> = pair.into_inner().collect();
        println!("🧩 解析到 {} 个 item:", items.len());
        for (i, item) in items.iter().enumerate() {
            println!("  [{}] Rule: {:?}, Text: {}", i, item.as_rule(), item.as_str());
        }


        match items.first().unwrap().as_str() {
            "with" => {
                // 处理 with 声明
                let mut packages = Vec::new();
                let mut property_sets = Vec::new();
                
                // 跳过第一个"with"项
                for item in items.iter().skip(1) {
                    match item.as_rule() {
                        aadlight_parser::Rule::package_name => {
                            packages.push(extract_package_name(item.clone()));
                        }
                        aadlight_parser::Rule::property_set_name => {
                            property_sets.push(extract_identifier(item.clone()));
                        }
                        _ => {} // 忽略逗号等其他符号
                    }
                }
                
                VisibilityDeclaration::Import {
                    packages,
                    property_sets,
                }
            }
            _ => {
                let identifier = extract_identifier(items[0].clone());
                //println!("🔎 尝试处理 renames 语句: {:?}", items);

                let original = extract_package_name(items[1].clone());

                VisibilityDeclaration::Alias {
                    new_name: identifier.clone(),
                    original: QualifiedName {
                        package_prefix: None,
                        identifier: original.0.join("::"),
                    },
                    is_package: true, // 假设现在只处理 package rename
                }
            }
        }
    }
    
    fn transform_package_section(pair: Pair<aadlight_parser::Rule>) -> PackageSection {
        let mut is_public = false;
        let mut declarations = Vec::new();
        
        let mut inner_iter = pair.into_inner();
        
        // 检查第一个元素是否是 public/private 修饰符
        if let Some(first) = inner_iter.next() {
            match first.as_str() {
                "public" => {
                    is_public = true;
                }
                "private" => {
                    is_public = false;
                }
                _ => {
                    // 如果不是修饰符，则是一个声明，需要回退处理
                    declarations.push(Self::transform_declaration(first));
                }
            }
        }
        
        // 处理剩余的声明
        for inner in inner_iter {
            match inner.as_rule() {
                aadlight_parser::Rule::declaration => {
                    declarations.push(Self::transform_declaration(inner));
                }
                _ => {} // 忽略其他规则
            }
        }
        
        PackageSection {
            is_public,
            declarations,
        }
    }
    
    fn transform_declaration(pair: Pair<aadlight_parser::Rule>) -> AadlDeclaration {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            aadlight_parser::Rule::component_type => {
                AadlDeclaration::ComponentType(Self::transform_component_type(inner))
            }
            aadlight_parser::Rule::component_implementation => {
                AadlDeclaration::ComponentImplementation(Self::transform_component_implementation(inner))
            }
            aadlight_parser::Rule::annex_library => {
                AadlDeclaration::AnnexLibrary(AnnexLibrary {})
            }
            _ => panic!("Unsupported declaration type: {:?}", inner.as_rule()),
        }
    }
    
    fn transform_component_type(pair: Pair<aadlight_parser::Rule>) -> ComponentType {
        let mut inner_iter = pair.into_inner();
        
        let category = match inner_iter.next().unwrap().as_str() {
            "system" => ComponentCategory::System,
            "process" => ComponentCategory::Process,
            "thread" => ComponentCategory::Thread,
            "data" => ComponentCategory::Data,
            "subprogram" => ComponentCategory::Subprogram,
            "processor" => ComponentCategory::Processor,
            "memory" => ComponentCategory::Memory,
            "device" => ComponentCategory::Device,
            "bus" => ComponentCategory::Bus,
            s => panic!("Unknown component category: {}", s),
        };
        
        let identifier = extract_identifier(inner_iter.next().unwrap());
        let mut prototypes = PrototypeClause::None;
        let mut features = FeatureClause::None;
        let mut properties = PropertyClause::ExplicitNone;
        let mut annexes = Vec::new();
        
        while let Some(inner) = inner_iter.next() {
            match inner.as_rule() {
                aadlight_parser::Rule::prototypes => {
                    prototypes = Self::transform_prototypes_clause(inner);
                }
                aadlight_parser::Rule::features => {
                    features = Self::transform_features_clause(inner);
                }
                aadlight_parser::Rule::properties => {
                    properties = Self::transform_properties_clause(inner);
                }
                aadlight_parser::Rule::annexes => {
                    annexes = Self::transform_annexes_clause(inner);
                }
                _ => {}
            }
        }
        
        ComponentType {
            category,
            identifier,
            prototypes,
            features,
            properties,
            annexes,
        }
    }
    
    fn transform_prototypes_clause(pair: Pair<aadlight_parser::Rule>) -> PrototypeClause {
        if pair.as_str().contains("none") {
            return PrototypeClause::Empty;
        }
        
        let mut prototypes = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == aadlight_parser::Rule::prototype_declaration {
                prototypes.push(Self::transform_prototype_declaration(inner));
            }
        }
        
        if prototypes.is_empty() {
            PrototypeClause::None
        } else {
            PrototypeClause::Items(prototypes)
        }
    }
    
    fn transform_prototype_declaration(pair: Pair<aadlight_parser::Rule>) -> Prototype {
        let mut inner_iter = pair.into_inner();
        let _identifier = extract_identifier(inner_iter.next().unwrap());
        let _colon = inner_iter.next();
        let prototype_type = inner_iter.next().unwrap();
        
        match prototype_type.as_str() {
            "component" => {
                let category = match inner_iter.next().unwrap().as_str() {
                    "system" => ComponentCategory::System,
                    "process" => ComponentCategory::Process,
                    "thread" => ComponentCategory::Thread,
                    "data" => ComponentCategory::Data,
                    "subprogram" => ComponentCategory::Subprogram,
                    "processor" => ComponentCategory::Processor,
                    "memory" => ComponentCategory::Memory,
                    s => panic!("Unknown component prototype category: {}", s),
                };
                
                Prototype::Component(ComponentPrototype {
                    category,
                    classifier: None, // TODO: Handle classifier
                    is_array: false,  // TODO: Handle array spec
                })
            }
            "feature" => {
                Prototype::Feature(FeaturePrototype {
                    direction: None, // TODO: Handle direction
                    classifier: None, // TODO: Handle classifier
                })
            }
            "feature group" => {
                Prototype::FeatureGroup(FeatureGroupPrototype {
                    classifier: None, // TODO: Handle classifier
                })
            }
            _ => panic!("Unknown prototype type"),
        }
    }
    
    fn transform_features_clause(pair: Pair<aadlight_parser::Rule>) -> FeatureClause {
        if pair.as_str().contains("none") {
            return FeatureClause::Empty;
        }
        
        let mut features = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == aadlight_parser::Rule::feature_declaration {
                features.push(Self::transform_feature_declaration(inner));
            }
        }
        
        if features.is_empty() {
            FeatureClause::None
        } else {
            FeatureClause::Items(features)
        }
    }
    
    fn transform_feature_declaration(pair: Pair<aadlight_parser::Rule>) -> Feature {
        let mut inner_iter = pair.into_inner();

        let identifier = extract_identifier(inner_iter.next().unwrap()); // p
        let mut direction: Option<PortDirection> = None;
        let mut port_type: Option<&str> = None;
        let mut type_name: Option<String> = None;

        for inner in inner_iter {
            match inner.as_rule() {
                aadlight_parser::Rule::direction => {
                    direction = match inner.as_str() {
                        "in" => Some(PortDirection::In),
                        "out" => Some(PortDirection::Out),
                        "in out" => Some(PortDirection::InOut),
                        _ => None,
                    };
                }
                aadlight_parser::Rule::port_type => {
                    port_type = Some(inner.as_str());
                }
                aadlight_parser::Rule::identifier => {
                    type_name = Some(inner.as_str().to_string());
                }
                _ => {}
            }
        }

        let (resolved_port_type, classifier) = match port_type.expect("Missing port type") {
            "data port" => {
                let classifier = type_name.clone().map(|type_id| {
                    PortDataTypeReference::Classifier(
                        UniqueComponentClassifierReference::Type(UniqueImplementationReference {
                            package_prefix: None,
                            implementation_name: ImplementationName {
                                type_identifier: type_id,
                                implementation_identifier: String::new(),
                            },
                        })
                    )
                });
                (PortType::Data { classifier: classifier.clone() }, classifier)
            }
            "event data port" => {
                let classifier = type_name.clone().map(|type_id| {
                    PortDataTypeReference::Classifier(
                        UniqueComponentClassifierReference::Type(UniqueImplementationReference {
                            package_prefix: None,
                            implementation_name: ImplementationName {
                                type_identifier: type_id,
                                implementation_identifier: String::new(),
                            },
                        })
                    )
                });
                (PortType::EventData { classifier: classifier.clone() }, classifier)
            }
            "event port" => (PortType::Event, None),
            "parameter" => {
                // TODO: 实现 parameter 处理
                (PortType::Event, None)
            }
            other => panic!("Unknown port type: {}", other),
        };

        Feature::Port(PortSpec {
            identifier,
            direction: direction.unwrap_or_else(|| match resolved_port_type {
                PortType::Data { .. } | PortType::EventData { .. } => PortDirection::InOut,
                PortType::Event => PortDirection::In,
            }),
            port_type: resolved_port_type,
        })
    }
    fn transform_properties_clause(pair: Pair<aadlight_parser::Rule>) -> PropertyClause {
        if pair.as_str().contains("none") {
            return PropertyClause::ExplicitNone;
        }
        
        let mut properties = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == aadlight_parser::Rule::property_association {
                properties.push(Self::transform_property_association(inner));
            }
        }
        
        if properties.is_empty() {
            PropertyClause::ExplicitNone
        } else {
            PropertyClause::Properties(properties)
        }
    }
    
    fn transform_property_association(pair: Pair<aadlight_parser::Rule>) -> Property {
        println!("=== 调试 property ===");
        println!("pair = Rule::{:?}, text = {}", pair.as_rule(), pair.as_str());

        let mut inner_iter = pair.into_inner();
        let identifier = extract_identifier(inner_iter.next().unwrap());
        
        let mut property_set = None;
        if let Some(inner) = inner_iter.next() {
            if inner.as_rule() == aadlight_parser::Rule::property_set_name {
                property_set = Some(extract_identifier(inner));
            }
        }
        
        let operator = match inner_iter.next().unwrap().as_str() {
            "=>" => PropertyOperator::Assign,
            "+=>" => PropertyOperator::Append,
            _ => panic!("Unknown property operator"),
        };
        
        let value = Self::transform_property_value(inner_iter.next().unwrap());
        
        Property::BasicProperty(BasicPropertyAssociation {
            identifier: PropertyIdentifier {
                property_set,
                name: identifier,
            },
            operator,
            is_constant: false, // TODO: Handle constant
            value,
        })
    }
    
    fn transform_property_value(pair: Pair<aadlight_parser::Rule>) -> PropertyValue {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            aadlight_parser::Rule::range_value => {
                let mut parts = inner.into_inner();
                let lower = extract_identifier(parts.next().unwrap());
                let _unit1 = parts.next();
                let _dotdot = parts.next();
                let upper = extract_identifier(parts.next().unwrap());
                let _unit2 = parts.next();
                
                PropertyValue::List(vec![
                    PropertyListElement::Value(PropertyExpression::String(StringTerm::Literal(lower))),
                    PropertyListElement::Value(PropertyExpression::String(StringTerm::Literal(upper))),
                ])
            }
            aadlight_parser::Rule::literal_value => {
                let value = inner.as_str().trim().to_string();
                PropertyValue::Single(PropertyExpression::String(StringTerm::Literal(value)))
            }
            aadlight_parser::Rule::list_value => {
                let mut elements = Vec::new();
                for item in inner.into_inner() {
                    elements.push(PropertyListElement::Value(
                        PropertyExpression::String(StringTerm::Literal(extract_identifier(item)))),
                    );
                }
                PropertyValue::List(elements)
            }
            aadlight_parser::Rule::reference_value => {
                let qualified_id = inner.into_inner().next().unwrap();
                PropertyValue::Single(PropertyExpression::String(StringTerm::Literal(
                    extract_identifier(qualified_id),
                )))
            }
            _ => panic!("Unknown property value type"),
        }
    }
    
    fn transform_annexes_clause(pair: Pair<aadlight_parser::Rule>) -> Vec<AnnexSubclause> {
        if pair.as_str().contains("none") {
            return Vec::new();
        }
        
        // TODO: Properly handle annexes
        Vec::new()
    }
    
    fn transform_component_implementation(pair: Pair<aadlight_parser::Rule>) -> ComponentImplementation {
        let mut inner_iter = pair.into_inner();
        
        let category = match inner_iter.next().unwrap().as_str() {
            "system" => ComponentCategory::System,
            "process" => ComponentCategory::Process,
            "thread" => ComponentCategory::Thread,
            "processor" => ComponentCategory::Processor,
            "memory" => ComponentCategory::Memory,
            s => panic!("Unknown component implementation category: {}", s),
        };
        
        // Skip "implementation" keyword
        let _ = inner_iter.next();
        
        let name_str = extract_identifier(inner_iter.next().unwrap());
        let mut name_parts = name_str.split('.');
        let name = ImplementationName {
            type_identifier: name_parts.next().unwrap().to_string(),
            implementation_identifier: name_parts.next().unwrap_or("").to_string(),
        };
        
        let mut prototypes = PrototypeClause::None;
        let mut subcomponents = SubcomponentClause::None;
        let mut calls = CallSequenceClause::None;
        let mut connections = ConnectionClause::None;
        let mut properties = PropertyClause::ExplicitNone;
        let mut annexes = Vec::new();
        
        while let Some(inner) = inner_iter.next() {
            match inner.as_rule() {
                aadlight_parser::Rule::prototypes => {
                    prototypes = Self::transform_prototypes_clause(inner);
                }
                aadlight_parser::Rule::subcomponents => {
                    subcomponents = Self::transform_subcomponents_clause(inner);
                }
                aadlight_parser::Rule::calls => {
                    calls = Self::transform_calls_clause(inner);
                }
                aadlight_parser::Rule::connections => {
                    connections = Self::transform_connections_clause(inner);
                }
                aadlight_parser::Rule::properties => {
                    properties = Self::transform_properties_clause(inner);
                }
                aadlight_parser::Rule::annexes => {
                    annexes = Self::transform_annexes_clause(inner);
                }
                _ => {}
            }
        }
        
        ComponentImplementation {
            category,
            name,
            prototype_bindings: None,
            prototypes,
            subcomponents,
            calls,
            connections,
            properties,
            annexes,
        }
    }
    
    fn transform_subcomponents_clause(pair: Pair<aadlight_parser::Rule>) -> SubcomponentClause {
        if pair.as_str().contains("none") {
            return SubcomponentClause::Empty;
        }
        
        let mut subcomponents = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == aadlight_parser::Rule::subcomponent {
                subcomponents.push(Self::transform_subcomponent(inner));
            }
        }
        
        if subcomponents.is_empty() {
            SubcomponentClause::None
        } else {
            SubcomponentClause::Items(subcomponents)
        }
    }
    
    fn transform_subcomponent(pair: Pair<aadlight_parser::Rule>) -> Subcomponent {
        let mut inner_iter = pair.into_inner();
        let identifier = extract_identifier(inner_iter.next().unwrap());
        let _colon = inner_iter.next();
        
        let category = match inner_iter.next().unwrap().as_str() {
            "system" => ComponentCategory::System,
            "process" => ComponentCategory::Process,
            "thread" => ComponentCategory::Thread,
            "processor" => ComponentCategory::Processor,
            "memory" => ComponentCategory::Memory,
            s => panic!("Unknown subcomponent category: {}", s),
        };
        
        let classifier = SubcomponentClassifier::ClassifierReference(
            UniqueComponentClassifierReference::Implementation(UniqueImplementationReference {
                package_prefix: None,
                implementation_name: ImplementationName {
                    type_identifier: extract_identifier(inner_iter.next().unwrap()),
                    implementation_identifier: String::new(),
                },
            }),
        );
        
        Subcomponent {
            identifier,
            category,
            classifier,
            array_spec: None, // TODO: Handle array spec
            properties: Vec::new(), // TODO: Handle properties
        }
    }
    
    fn transform_calls_clause(pair: Pair<aadlight_parser::Rule>) -> CallSequenceClause {
        if pair.as_str().contains("none") {
            return CallSequenceClause::Empty;
        }
        
        let mut call_sequences = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == aadlight_parser::Rule::call_sequence {
                call_sequences.push(Self::transform_call_sequence(inner));
            }
        }
        
        if call_sequences.is_empty() {
            CallSequenceClause::None
        } else {
            CallSequenceClause::Items(call_sequences)
        }
    }
    
    fn transform_call_sequence(pair: Pair<aadlight_parser::Rule>) -> CallSequence {
        let mut inner_iter = pair.into_inner();
        let identifier = extract_identifier(inner_iter.next().unwrap());
        let _colon = inner_iter.next();
        let _open_brace = inner_iter.next();
        
        let mut calls = Vec::new();
        while let Some(inner) = inner_iter.next() {
            if inner.as_rule() == aadlight_parser::Rule::subprogram_call {
                calls.push(Self::transform_subprogram_call(inner));
            }
        }
        
        CallSequence {
            identifier,
            calls,
            properties: Vec::new(), // TODO: Handle properties
            in_modes: None, // TODO: Handle modes
        }
    }
    
    fn transform_subprogram_call(pair: Pair<aadlight_parser::Rule>) -> SubprogramCall {
        let mut inner_iter = pair.into_inner();
        let identifier = extract_identifier(inner_iter.next().unwrap());
        let _colon = inner_iter.next();
        let _subprogram = inner_iter.next();
        
        let called = CalledSubprogram::Classifier(
            UniqueComponentClassifierReference::Implementation(UniqueImplementationReference {
                package_prefix: None,
                implementation_name: ImplementationName {
                    type_identifier: extract_identifier(inner_iter.next().unwrap()),
                    implementation_identifier: String::new(),
                },
            }),
        );
        
        SubprogramCall {
            identifier,
            called,
            properties: Vec::new(), // TODO: Handle properties
        }
    }
    
    fn transform_connections_clause(pair: Pair<aadlight_parser::Rule>) -> ConnectionClause {
        if pair.as_str().contains("none") {
            return ConnectionClause::Empty;
        }
        
        let mut connections = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == aadlight_parser::Rule::connection {
                connections.push(Self::transform_connection(inner));
            }
        }
        
        if connections.is_empty() {
            ConnectionClause::None
        } else {
            ConnectionClause::Items(connections)
        }
    }
    
    fn transform_connection(pair: Pair<aadlight_parser::Rule>) -> Connection {
        println!("=== 调试 connection ===");
        println!("pair = Rule::{:?}, text = {}", pair.as_rule(), pair.as_str());

        for (i, inner) in pair.clone().into_inner().enumerate() {
            println!("  inner[{}]: Rule::{:?}, text = {}", i, inner.as_rule(), inner.as_str());
        }


        let mut inner_iter = pair.into_inner();
        let identifier = extract_identifier(inner_iter.next().unwrap());
        //let _colon = inner_iter.next();
        
        let connection_type = inner_iter.next().unwrap();
        let connection_body = inner_iter.next().unwrap(); // ✅ port_connection or parameter_connection

        match connection_type.as_str() {
            "port" => {
            let mut port_iter = connection_body.into_inner();

            let source = Self::transform_port_reference(port_iter.next().unwrap());
            let direction = match port_iter.next().unwrap().as_str() {
                "->" => ConnectionSymbol::Direct,
                "<->" => ConnectionSymbol::Didirect,
                _ => panic!("Unknown connection direction"),
            };
            let destination = Self::transform_port_reference(port_iter.next().unwrap());

            Connection::Port(PortConnection {
                source,
                destination,
                connection_direction: direction,
            })
        }
            "parameter" => {
                // TODO: Handle parameter connections
                Connection::Parameter(ParameterConnection {
                    source: ParameterEndpoint::ComponentParameter {
                        parameter: "".to_string(),
                        data_subcomponent: None,
                    },
                    destination: ParameterEndpoint::ComponentParameter {
                        parameter: "".to_string(),
                        data_subcomponent: None,
                    },
                    connection_direction: ConnectionSymbol::Direct,
                })
            }
            _ => panic!("Unknown connection type"),
        }
    }
    
    fn transform_port_reference(pair: Pair<aadlight_parser::Rule>) -> PortEndpoint {
        let reference = pair.as_str().trim();
        if reference.contains('.') {
            let mut parts = reference.split('.');
            PortEndpoint::SubcomponentPort {
                subcomponent: parts.next().unwrap().to_string(),
                port: parts.next().unwrap().to_string(),
            }
        } else {
            PortEndpoint::ComponentPort(reference.to_string())
        }
    }
}

fn print_pair(pair: Pair<aadlight_parser::Rule>, indent: usize) {
    match pair.as_rule() {
        aadlight_parser::Rule::WHITESPACE | aadlight_parser::Rule::COMMENT => return,
        _ => {
            let span = pair.as_span();
            let (start_line, _) = span.start_pos().line_col();
            let (end_line, _) = span.end_pos().line_col();
            
            let content = pair.as_str().trim();
            let truncated_content = if content.len() > 30 {
                format!("{}...", &content[..30])
            } else {
                content.to_string()
            };
            
            println!(
                "{}{:<25} {:<30} (lines {}-{})",
                "  ".repeat(indent),
                format!("{:?}:", pair.as_rule()),
                truncated_content,
                start_line,
                end_line
            );

            for inner in pair.into_inner() {
                print_pair(inner, indent + 1);
            }
        }
    }
}
fn main() {
    let path = "pingpong.aadl";
    let aadl_input = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("读取文件失败: {}", err);
            return;
        }
    };
    
    match AADLParser::parse(aadlight_parser::Rule::file, &aadl_input) {
        Ok(pairs) => {
            println!("=== 解析成功，共 {} 个pair ===", pairs.clone().count()); // 调试输出            
            // 转换到AST
            let ast = AADLTransformer::transform_file(pairs.clone().collect());
            println!("=== 转换得到 {} 个package ===", ast.len()); // 调试输出

            // 打印AST
            println!("\n================================== AST ==================================");
            for package in ast {
                println!("Package: {}", package.name.to_string());
                if let Some(public_section) = package.public_section {
                    for decl in public_section.declarations {
                        match decl {
                            AadlDeclaration::ComponentType(comp) => {
                                println!("  Component Type: {} ({:?})", comp.identifier, comp.category);
                                if let FeatureClause::Items(features) = comp.features {
                                    for feature in features {
                                        if let Feature::Port(port) = feature {
                                            println!("    Port: {} {:?} {:?}", port.identifier, port.direction, port.port_type);
                                        }
                                    }
                                }
                                if let PropertyClause::Properties(props) = comp.properties {
                                    for prop in props {
                                        if let Property::BasicProperty(bp) = prop {
                                            println!("    Property: {} => {:?}", bp.identifier.name, bp.value);
                                        }
                                    }
                                }
                            }
                            AadlDeclaration::ComponentImplementation(impl_) => {
                                println!("  Component Implementation: {} ({:?})", impl_.name.to_string(), impl_.category);
                                if let SubcomponentClause::Items(subcomps) = impl_.subcomponents {
                                    for subcomp in subcomps {
                                        println!("    Subcomponent: {} {:?}", subcomp.identifier, subcomp.category);
                                    }
                                }
                                if let ConnectionClause::Items(conns) = impl_.connections {
                                    for conn in conns {
                                        if let Connection::Port(port_conn) = conn {
                                            println!("    Connection: {:?} -> {:?}", port_conn.source, port_conn.destination);
                                        }
                                    }
                                }
                                if let PropertyClause::Properties(props) = impl_.properties {
                                    for prop in props {
                                        if let Property::BasicProperty(bp) = prop {
                                            println!("    Property: {} => {:?}", bp.identifier.name, bp.value);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            // 打印原始解析树
            println!("\n================================== Parse Tree ==================================");
            for pair in pairs {
                print_pair(pair, 0);
            }
        }
        Err(e) => {
            eprintln!("解析失败: {}", e);
        }
    }
}
    