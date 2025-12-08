use crate::aadlAst2rustCode::converter::AadlConverter;
use crate::aadlAst2rustCode::intermediate_ast::*;

use crate::ast::aadl_ast_cj::*;

pub fn convert_system_implementation(
    temp_converter: &mut AadlConverter,
    impl_: &ComponentImplementation,
) -> Vec<Item> {
    let mut items = Vec::new();

    // 1. 生成系统结构体
    let fields = get_system_fields(impl_); // 获取系统的子组件

    let struct_def = StructDef {
        name: format!("{}System", impl_.name.type_identifier.to_lowercase()),
        fields,                 // 系统的子组件
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
    items.push(Item::Impl(create_system_impl_block(temp_converter, impl_)));

    items
}

fn get_system_fields(impl_: &ComponentImplementation) -> Vec<Field> {
    let mut fields = Vec::new();

    if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
        for sub in subcomponents {
            if matches!(
                sub.category,
                ComponentCategory::Process | ComponentCategory::Device
            ) {
                let type_name = match &sub.classifier {
                    SubcomponentClassifier::ClassifierReference(
                        UniqueComponentClassifierReference::Implementation(unirf),
                    ) => unirf.implementation_name.type_identifier.clone(),
                    _ => "UnsupportedComponent".to_string(),
                };

                let type_suffix = match sub.category {
                    ComponentCategory::Process => "Process",
                    ComponentCategory::Device => "Device",
                    _ => unreachable!("Filtered above"),
                };

                let field_ty = Type::Named(format!("{}{}", type_name.to_lowercase(), type_suffix));
                let doc = match sub.category {
                    ComponentCategory::Process => {
                        format!(
                            "// 子组件进程（{} : process {}）",
                            sub.identifier, type_name
                        )
                    }
                    ComponentCategory::Device => {
                        format!("// 子组件设备（{} : device {}）", sub.identifier, type_name)
                    }
                    _ => unreachable!("Filtered above"),
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
    }

    fields
}

fn create_system_impl_block(
    temp_converter: &mut AadlConverter,
    impl_: &ComponentImplementation,
) -> ImplBlock {
    let mut items = Vec::new();

    // 添加new方法
    items.push(ImplItem::Method(FunctionDef {
        name: "new".to_string(),
        params: vec![],
        return_type: Type::Named("Self".to_string()),
        body: create_system_new_body(temp_converter, impl_),
        asyncness: false,
        vis: Visibility::None,
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
        body: create_system_run_body(impl_),
        asyncness: false,
        vis: Visibility::None,
        docs: vec!["// Runs the system, starts all processes".to_string()],
        attrs: Vec::new(),
    }));

    ImplBlock {
        target: Type::Named(format!(
            "{}System",
            impl_.name.type_identifier.to_lowercase()
        )),
        generics: Vec::new(),
        items,
        trait_impl: Some(Type::Named("System".to_string())),
    }
}

// 提取系统实现中的处理器绑定信息
fn extract_processor_bindings(impl_: &ComponentImplementation) -> Vec<(String, String)> {
    let mut bindings = Vec::new();

    if let PropertyClause::Properties(properties) = &impl_.properties {
        for property in properties {
            if let Property::BasicProperty(basic_prop) = property {
                if basic_prop.identifier.name.to_lowercase() == "actual_processor_binding" {
                    if let PropertyValue::Single(PropertyExpression::Reference(ref_term)) =
                        &basic_prop.value
                    {
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
fn create_system_new_body(
    temp_converter: &mut AadlConverter,
    impl_: &ComponentImplementation,
) -> Block {
    let mut stmts = Vec::new();

    // 1. 提取处理器绑定信息并创建CPU映射
    let processor_bindings = extract_processor_bindings(impl_);

    // 为每个唯一的CPU名称分配一个ID（如果还没有分配的话）
    for (_, cpu_name) in &processor_bindings {
        if !temp_converter.cpu_name_to_id_mapping.contains_key(cpu_name) {
            let next_id = temp_converter.cpu_name_to_id_mapping.len();
            temp_converter
                .cpu_name_to_id_mapping
                .insert(cpu_name.clone(), next_id);
        }
    }

    // 如果没有处理器绑定，默认使用CPU 0
    if temp_converter.cpu_name_to_id_mapping.is_empty() {
        temp_converter
            .cpu_name_to_id_mapping
            .insert("default".to_string(), 0);
    }

    // 2. 创建子组件实例 - 处理进程和设备子组件
    if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
        for sub in subcomponents {
            let var_name = sub.identifier.to_lowercase();
            let type_name = match &sub.classifier {
                SubcomponentClassifier::ClassifierReference(
                    UniqueComponentClassifierReference::Implementation(unirf),
                ) => unirf.implementation_name.type_identifier.clone(),
                _ => "UnsupportedComponent".to_string(),
            };

            match sub.category {
                ComponentCategory::Process => {
                    // 查找该进程的CPU绑定
                    let cpu_id = processor_bindings
                        .iter()
                        .find(|(process_name, _)| process_name == &sub.identifier)
                        .and_then(|(_, cpu_name)| {
                            temp_converter.cpu_name_to_id_mapping.get(cpu_name).copied()
                        })
                        .unwrap_or(0); // 默认使用CPU 0

                    let creation_stmt = format!(
                        "let mut {}: {}Process = {}Process::new({})",
                        var_name,
                        type_name.to_lowercase(),
                        type_name.to_lowercase(),
                        cpu_id
                    );
                    stmts.push(Statement::Expr(Expr::Ident(creation_stmt)));
                }
                ComponentCategory::Device => {
                    let creation_stmt = format!(
                        "let mut {}: {}Device = {}Device::new()",
                        var_name,
                        type_name.to_lowercase(),
                        type_name.to_lowercase()
                    );
                    stmts.push(Statement::Expr(Expr::Ident(creation_stmt)));
                }
                _ => {}
            }
        }
    }

    // 2. 构建连接（如果有的话）
    // 函数内存储已处理过的广播连接，避免二次处理。
    let mut processed_broadcast_connections = Vec::new();

    if let ConnectionClause::Items(connections) = &impl_.connections {
        for conn in connections {
            match conn {
                Connection::Port(port_conn) => {
                    //查看连接是否是已处理过的广播，如果是，则不需要再处理，跳过。
                    if let PortEndpoint::SubcomponentPort { subcomponent, port } = &port_conn.source
                    {
                        if temp_converter
                            .process_broadcast_send
                            .contains(&(subcomponent.clone(), port.clone()))
                        {
                            if processed_broadcast_connections
                                .contains(&(subcomponent.clone(), port.clone()))
                            {
                                continue;
                            } else {
                                processed_broadcast_connections
                                    .push((subcomponent.clone(), port.clone()));
                            }
                        }
                    }
                    // 处理端口连接，使用与进程相同的逻辑
                    stmts.extend(
                        temp_converter.create_channel_connection(
                            port_conn,
                            impl_.name.type_identifier.clone(),
                        ),
                    );
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
            if matches!(
                sub.category,
                ComponentCategory::Process | ComponentCategory::Device
            ) {
                field_names.push(sub.identifier.to_lowercase());
            }
        }
    }

    let fields_str = field_names.join(", ");
    stmts.push(Statement::Expr(Expr::Ident(format!(
        "return Self {{ {} }}  //显式return",
        fields_str
    ))));

    Block { stmts, expr: None }
}

// 创建系统实例中run()方法
fn create_system_run_body(impl_: &ComponentImplementation) -> Block {
    let mut stmts = Vec::new();

    if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
        for sub in subcomponents {
            let var_name = sub.identifier.to_lowercase();
            match sub.category {
                ComponentCategory::Process => {
                    let start_stmt = format!("self.{}.start()", var_name);
                    stmts.push(Statement::Expr(Expr::Ident(start_stmt)));
                }
                ComponentCategory::Device => {
                    // 构建线程闭包（使用move语义捕获self）
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
                            move_kw: true, // 使用move关键字捕获self
                        },
                    ];

                    stmts.push(Statement::Expr(Expr::MethodCall(
                        Box::new(Expr::BuilderChain(builder_chain)),
                        "unwrap".to_string(),
                        Vec::new(),
                    )));
                }
                _ => {}
            }
        }
    }

    Block { stmts, expr: None }
}
