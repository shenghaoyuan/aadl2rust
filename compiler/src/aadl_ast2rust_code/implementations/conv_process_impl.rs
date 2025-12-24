use crate::aadl_ast2rust_code::converter::AadlConverter;
use crate::aadl_ast2rust_code::intermediate_ast::*;

use crate::ast::aadl_ast_cj::*;

pub fn convert_process_implementation(
    temp_converter: &mut AadlConverter,
    impl_: &ComponentImplementation,
) -> Vec<Item> {
    let mut items = Vec::new();

    // 1. 生成进程结构体
    let mut fields = get_process_fields(temp_converter, impl_); //这里是为了取得进程的子组件；生成内部端口也放在这里。
    
    // 添加 CPU ID 字段
    fields.push(Field {
        name: "cpu_id".to_string(),
        ty: Type::Named("isize".to_string()),
        docs: vec!["// 新增 CPU ID".to_string()],
        attrs: Vec::new(),
    });

    let struct_def = StructDef {
        name: format! {"{}Process",impl_.name.type_identifier.to_lowercase()},
        fields,                 //这里是为了取得进程的子组件
        properties: Vec::new(), //TODO
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
    items.push(Item::Impl(create_process_impl_block(temp_converter, impl_)));

    items
}

//新增转发端口（内部端口，用于转发数据到子组件）；处理子组件（thread+data）
fn get_process_fields(
    temp_converter: &mut AadlConverter,
    impl_: &ComponentImplementation,
) -> Vec<Field> {
    let mut fields = Vec::new();

    // 1. 添加进程的端口字段（对外端口 + 内部端口）
    if let Some(comp_type) = temp_converter.get_component_type(impl_) {
        if let FeatureClause::Items(features) = &comp_type.features {
            for feature in features {
                match feature {
                    Feature::Port(port) => {
                        // 添加对外端口
                        fields.push(Field {
                            name: port.identifier.to_lowercase(),
                            ty: temp_converter.convert_port_type(&port, "".to_string()),
                            docs: vec![format!(
                                "// Port: {} {:?}",
                                port.identifier, port.direction
                            )],
                            attrs: Vec::new(),
                        });

                        // 添加对应的内部端口
                        let internal_port_name = match port.direction {
                            PortDirection::In => format!("{}Send", port.identifier.to_lowercase()),
                            PortDirection::Out => format!("{}Rece", port.identifier.to_lowercase()),
                            PortDirection::InOut => {
                                format!("{}Send", port.identifier.to_lowercase())
                            } // InOut 暂时按 In 处理
                        };

                        let internal_port_type = match port.direction {
                            PortDirection::In => {
                                // 对外是接收端口，内部需要发送端口
                                match temp_converter.convert_port_type(&port, "".to_string()) {
                                    Type::Generic(option_name, inner_types)
                                        if option_name == "Option" =>
                                    {
                                        if let Type::Generic(channel_name, channel_args) =
                                            &inner_types[0]
                                        {

                                            let mut send_type = "Sender".to_string();
                                            // 从 Option<Receiver<T>> 转换为 Option<BcSender<T>>
                                            if temp_converter.thread_broadcast_receive.contains_key(&(port.identifier.clone(),impl_.name.type_identifier.clone())){
                                                send_type = "BcSender".to_string();
                                            }

                                            if channel_name == "Receiver" {
                                                Type::Generic(
                                                    "Option".to_string(),
                                                    vec![Type::Generic(
                                                        send_type.clone(),
                                                        channel_args.clone(),
                                                    )],
                                                )
                                            } else {
                                                Type::Generic(
                                                    "Option".to_string(),
                                                    vec![Type::Generic(
                                                        channel_name.clone(),
                                                        channel_args.clone(),
                                                    )],
                                                )
                                            }
                                        } else {
                                            Type::Generic(
                                                "Option".to_string(),
                                                vec![Type::Generic(
                                                    "Sender".to_string(),
                                                    vec![inner_types[0].clone()],
                                                )],
                                            )
                                        }
                                    }
                                    _ => {
                                        // 如果不是 Option 类型，创建 Option<BcSender<T>>
                                        Type::Generic(
                                            "Option".to_string(),
                                            vec![Type::Generic(
                                                "Sender".to_string(),
                                                vec![temp_converter
                                                    .convert_port_type(&port, "".to_string())],
                                            )],
                                        )
                                    }
                                }
                            }
                            PortDirection::Out => {
                                // 对外是发送端口，内部需要接收端口
                                match temp_converter.convert_port_type(&port, "".to_string()) {
                                    Type::Generic(option_name, inner_types)
                                        if option_name == "Option" =>
                                    {
                                        if let Type::Generic(channel_name, channel_args) =
                                            &inner_types[0]
                                        {
                                            if channel_name == "Sender" {
                                                // 从 Option<Sender<T>> 转换为 Option<Receiver<T>>
                                                Type::Generic(
                                                    "Option".to_string(),
                                                    vec![Type::Generic(
                                                        "Receiver".to_string(),
                                                        channel_args.clone(),
                                                    )],
                                                )
                                            } else {
                                                Type::Generic(
                                                    "Option".to_string(),
                                                    vec![Type::Generic(
                                                        channel_name.clone(),
                                                        channel_args.clone(),
                                                    )],
                                                )
                                            }
                                        } else {
                                            Type::Generic(
                                                "Option".to_string(),
                                                vec![Type::Generic(
                                                    "Receiver".to_string(),
                                                    vec![inner_types[0].clone()],
                                                )],
                                            )
                                        }
                                    }
                                    _ => {
                                        // 如果不是 Option 类型，创建 Option<Receiver<T>>
                                        Type::Generic(
                                            "Option".to_string(),
                                            vec![Type::Generic(
                                                "Receiver".to_string(),
                                                vec![temp_converter
                                                    .convert_port_type(&port, "".to_string())],
                                            )],
                                        )
                                    }
                                }
                            }
                            PortDirection::InOut => {
                                // InOut 暂时按 In 处理
                                match temp_converter.convert_port_type(&port, "".to_string()) {
                                    Type::Generic(option_name, inner_types)
                                        if option_name == "Option" =>
                                    {
                                        if let Type::Generic(channel_name, channel_args) =
                                            &inner_types[0]
                                        {
                                            if channel_name == "Receiver" {
                                                // 从 Option<Receiver<T>> 转换为 Option<BcSender<T>>
                                                Type::Generic(
                                                    "Option".to_string(),
                                                    vec![Type::Generic(
                                                        "BcSender".to_string(),
                                                        channel_args.clone(),
                                                    )],
                                                )
                                            } else {
                                                Type::Generic(
                                                    "Option".to_string(),
                                                    vec![Type::Generic(
                                                        channel_name.clone(),
                                                        channel_args.clone(),
                                                    )],
                                                )
                                            }
                                        } else {
                                            Type::Generic(
                                                "Option".to_string(),
                                                vec![Type::Generic(
                                                    "BcSender".to_string(),
                                                    vec![inner_types[0].clone()],
                                                )],
                                            )
                                        }
                                    }
                                    _ => {
                                        // 如果不是 Option 类型，创建 Option<BcSender<T>>
                                        Type::Generic(
                                            "Option".to_string(),
                                            vec![Type::Generic(
                                                "BcSender".to_string(),
                                                vec![temp_converter
                                                    .convert_port_type(&port, "".to_string())],
                                            )],
                                        )
                                    }
                                }
                            }
                        };

                        fields.push(Field {
                            name: internal_port_name,
                            ty: internal_port_type,
                            docs: vec![format!(
                                "// 内部端口: {} {:?}",
                                port.identifier, port.direction
                            )],
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
                ComponentCategory::Thread => {
                    format!("// 子组件线程({} : thread {})", sub.identifier, type_name)
                }
                ComponentCategory::Data => {
                    // 直接使用原始类型名
                    format!("// 共享数据({} : data {})", sub.identifier, type_name)
                }
                _ => format!("// Subcomponent: {}", sub.identifier),
            };

            fields.push(Field {
                name: sub.identifier.to_lowercase(),
                ty: field_ty,
                docs: vec![doc],
                // attrs: vec![Attribute {
                //     name: "allow".to_string(),
                //     args: vec![AttributeArg::Ident("dead_code".to_string())],
                // }],
                attrs: Vec::new(),
            });
        }
    }

    fields
}

fn create_process_impl_block(
    temp_converter: &mut AadlConverter,
    impl_: &ComponentImplementation,
) -> ImplBlock {
    let mut items = Vec::new();

    // 添加new方法
    items.push(ImplItem::Method(FunctionDef {
        name: "new".to_string(),
        params: vec![Param {
            name: "cpu_id".to_string(),
            ty: Type::Named("isize".to_string()),
        }],
        return_type: Type::Named("Self".to_string()),
        body: create_process_new_body(temp_converter, impl_),
        asyncness: false,
        vis: Visibility::None,
        docs: vec!["// Creates a new process instance".to_string()],
        attrs: Vec::new(),
    }));

    // 添加start方法
    items.push(ImplItem::Method(FunctionDef {
        name: "run".to_string(),
        params: vec![Param {
            name: "self".to_string(),
            ty: Type::Named("Self".to_string()),
        }],
        return_type: Type::Unit,
        body: create_process_start_body(temp_converter, impl_),
        asyncness: false,
        vis: Visibility::None,
        docs: vec!["// Starts all threads in the process".to_string()],
        attrs: Vec::new(),
    }));

    ImplBlock {
        target: Type::Named(format! {"{}Process",impl_.name.type_identifier.to_lowercase()}),
        generics: Vec::new(),
        items,
        trait_impl: Some(Type::Named("Process".to_string())),
    }
}

fn create_process_new_body(
    temp_converter: &mut AadlConverter,
    impl_: &ComponentImplementation,
) -> Block {
    let mut stmts = Vec::new();
    // 为每个线程收集需要注入到 new() 的共享变量参数（例如 data access 映射）
    let mut thread_extra_args: std::collections::HashMap<String, Vec<Expr>> =
        std::collections::HashMap::new();

    if let ConnectionClause::Items(connections) = &impl_.connections {
        for conn in connections {
            if let Connection::Access(access_conn) = conn {
                // 仅处理 data access 映射：ComponentAccess(data) -> SubcomponentAccess{ subcomponent: thread, .. }
                match (&access_conn.source, &access_conn.destination) {
                    (
                        AccessEndpoint::ComponentAccess(data_name),
                        AccessEndpoint::SubcomponentAccess {
                            subcomponent: thread_name,
                            ..
                        },
                    ) => {
                        let thread_key = thread_name.to_lowercase();
                        let data_var = data_name.to_lowercase();
                        let entry = thread_extra_args.entry(thread_key).or_default();
                        // 传递克隆：pos_data.clone()
                        // entry.push(Expr::MethodCall(
                        //     Box::new(Expr::Ident(data_var)),
                        //     "clone".to_string(),
                        //     Vec::new(),
                        // ));

                        //修改：显式传递,Arc::clone(&pos_data),即Arc::clone(&data_var)
                        entry.push(Expr::MethodCall(
                            Box::new(Expr::Path(
                                vec!["Arc".to_string(), "clone".to_string()],
                                PathType::Namespace,
                            )),
                            "".to_string(),
                            vec![Expr::Reference(Box::new(Expr::Ident(data_var)), true, false)],
                        ));
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
                        Box::new(Expr::Path(
                            vec!["Arc".to_string(), "new".to_string()],
                            PathType::Namespace,
                        )),
                        vec![Expr::Call(
                            Box::new(Expr::Path(
                                vec!["Mutex".to_string(), "new".to_string()],
                                PathType::Namespace,
                            )),
                            vec![Expr::Literal(Literal::Int(0))],
                        )],
                    );
                    data_inits.push(Statement::Let(LetStmt {
                        ifmut: false,
                        name: format!("{}", var_name),
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
                        ifmut: true,
                        name: format!("{}", var_name),
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
    if let Some(comp_type) = temp_converter.get_component_type(impl_) {
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
    // 函数内存储已处理过的广播连接，避免二次处理。
    let mut processed_broadcast_connections = Vec::new();

    if let ConnectionClause::Items(connections) = &impl_.connections {
        for conn in connections {
            if let Connection::Port(port_conn) = conn {
                if let PortEndpoint::ComponentPort(proc_port) = &port_conn.source {
                    if temp_converter
                        .thread_broadcast_receive
                        .contains_key(&(proc_port.clone(), impl_.name.type_identifier.clone()))
                    {
                        if processed_broadcast_connections
                            .contains(&(proc_port.clone(), impl_.name.type_identifier.clone()))
                        {
                            continue;
                        } else {
                            processed_broadcast_connections
                                .push((proc_port.clone(), impl_.name.type_identifier.clone()));
                        }
                    }
                }
                stmts.extend(
                    temp_converter
                        .create_channel_connection(port_conn, impl_.name.type_identifier.clone()),
                );
            }
        }
    }

    // 3. 返回结构体实例
    let mut field_inits = Vec::new();

    // 添加端口字段初始化（对外端口初始化为None，内部端口使用变量）
    if let Some(comp_type) = temp_converter.get_component_type(impl_) {
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

fn create_process_start_body(
    temp_converter: &mut AadlConverter,
    impl_: &ComponentImplementation,
) -> Block {
    let mut stmts = Vec::new();

    // 1. 解构self，获取所有需要的字段
    let mut destructure_fields = Vec::new();
    let mut thread_fields = Vec::new();
    let mut port_fields = Vec::new();

    // 1.1 添加端口字段（来自features）
    if let Some(comp_type) = temp_converter.get_component_type(impl_) {
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
    // destructure_fields.push("cpu_id".to_string());

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
                move_kw: true,
            },
        ];

        stmts.push(Statement::Expr(Expr::MethodCall(
            Box::new(Expr::BuilderChain(builder_chain)),
            "unwrap".to_string(),
            Vec::new(),
        )));
    }

    // 3. 启动数据转发循环（使用解构后的变量）
    let mut forwarding_tasks = create_data_forwarding_tasks(impl_);
    forwarding_tasks.sort();
    forwarding_tasks.dedup();

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
        let forwarding_loop = create_single_forwarding_thread(&rx_var_name, &dst_field);
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
fn create_data_forwarding_tasks(impl_: &ComponentImplementation) -> Vec<(String, String)> {
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
                            subcomponent: _dst_comp,
                            port: _dst_port,
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
                            subcomponent: _src_comp,
                            port: _src_port,
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
fn create_single_forwarding_thread(rx_var_name: &str, dst_field: &str) -> Vec<Statement> {
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
                stmts: vec![Statement::Expr(Expr::IfLet {
                    pattern: "Some(tx)".to_string(),
                    value: Box::new(Expr::Reference(
                        Box::new(Expr::Ident(dst_field.to_string())),
                        true,
                        false,
                    )),
                    then_branch: Block {
                        stmts: vec![Statement::Let(LetStmt {
                            ifmut: false,
                            name: "_".to_string(),
                            ty: None,
                            init: Some(Expr::MethodCall(
                                Box::new(Expr::Ident("tx".to_string())),
                                "send".to_string(),
                                vec![Expr::Ident("msg".to_string())],
                            )),
                        })],
                        expr: None,
                    },
                    else_branch: None,
                })],
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
                    vec![
                        "std".to_string(),
                        "time".to_string(),
                        "Duration".to_string(),
                        "from_millis".to_string(),
                    ],
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
