use crate::aadl_ast2rust_code::intermediate_ast::*;
use crate::aadl_ast2rust_code::tool;
use crate::ast::aadl_ast_cj::*;
use std::collections::HashMap;
/**
    收集器：收集AADL模型中的组件类型信息、process之间的多连接关系、thread之间的多连接关系
    collect_component_types: 收集所有组件类型信息
    collect_process_connections: 收集system内process之间的多连接关系
    collect_thread_connections: 收集process内和thread之间的多连接关系
*/

// 收集所有组件类型信息
pub fn collect_component_types(
    component_types: &mut HashMap<String, ComponentType>,
    pkg: &Package,
) {
    // 处理公共声明中的组件类型
    if let Some(public_section) = &pkg.public_section {
        for decl in &public_section.declarations {
            if let AadlDeclaration::ComponentType(comp) = decl {
                component_types.insert(comp.identifier.clone(), comp.clone());
            }
        }
    }

    // 处理私有声明中的组件类型
    if let Some(private_section) = &pkg.private_section {
        for decl in &private_section.declarations {
            if let AadlDeclaration::ComponentType(comp) = decl {
                component_types.insert(comp.identifier.clone(), comp.clone());
            }
        }
    }
}

//收集system内process之间的多连接关系
pub fn collect_process_connections(
    process_broadcast_send: &mut Vec<(String, String)>,
    process_broadcast_receive: &mut HashMap<(String, String), Vec<(String, String)>>,
    system_subcomponent_identify_to_type: &mut HashMap<String, String>,
    pkg: &Package,
) {
    //创建一个映射，存储已经出现的，key为子组件名称，value为端口名称。用来判断是否是重复出现的端口
    // let mut subcomponent_ports = HashMap::new();

    if let Some(public_section) = &pkg.public_section {
        for decl in &public_section.declarations {
            if let AadlDeclaration::ComponentImplementation(impl_) = decl {
                if impl_.category == ComponentCategory::System {
                    if let ConnectionClause::Items(connections) = &impl_.connections {
                        for conn in connections {
                            if let Connection::Port(port_conn) = conn {
                                if let PortEndpoint::SubcomponentPort { subcomponent, port } =
                                    &port_conn.source
                                {
                                    process_broadcast_send
                                        .push((subcomponent.clone(), port.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    //筛选重复出现的（发送）端口。保留。
    let temp_dedup_with_min_two_unique = tool::dedup_with_min_two_unique(process_broadcast_send);
    //现在process_connections_alias中存储的是重复出现的（发送）端口。
    //还需要根据它，再次遍历连接关系connections，找到所有与它们连接的接收端口，加入process_connections_alias
    for ele in &temp_dedup_with_min_two_unique {
        if let Some(public_section) = &pkg.public_section {
            for decl in &public_section.declarations {
                if let AadlDeclaration::ComponentImplementation(impl_) = decl {
                    if impl_.category == ComponentCategory::System {
                        if let ConnectionClause::Items(connections) = &impl_.connections {
                            for conn in connections {
                                if let Connection::Port(port_conn) = conn {
                                    if let PortEndpoint::SubcomponentPort { subcomponent, port } =
                                        &port_conn.source
                                    {
                                        if subcomponent.eq(&ele.0) && port.eq(&ele.1) {
                                            if let PortEndpoint::SubcomponentPort {
                                                subcomponent: d_subcomponent,
                                                port: d_port,
                                            } = &port_conn.destination
                                            {
                                                //self.process_broadcast_send.push((d_subcomponent.clone(), d_port.clone()));
                                                process_broadcast_receive
                                                    .entry(ele.clone())
                                                    .or_insert(Vec::new())
                                                    .push((d_subcomponent.clone(), d_port.clone()));
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
    //遍历process_broadcast_receive的键值对中所有键和值中的组件，查看该系统中的subcomponent部分，把有广播发送和接收的端口，找到它的真实类型，并记录下来存储到system_subcomponent_identify_to_type。
    for (sendcomp, vercport) in process_broadcast_receive {
        for (comp, _) in vercport {
            if let Some(public_section) = &pkg.public_section {
                for decl in &public_section.declarations {
                    if let AadlDeclaration::ComponentImplementation(impl_) = decl {
                        if impl_.category == ComponentCategory::System {
                            if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
                                for sub in subcomponents {
                                    if sub.identifier.eq(&comp.clone()) {
                                        if let SubcomponentClassifier::ClassifierReference(
                                            classifier,
                                        ) = &sub.classifier
                                        {
                                            if let UniqueComponentClassifierReference::Implementation(unirf) = classifier {
                                                system_subcomponent_identify_to_type.insert(sub.identifier.clone(), unirf.implementation_name.type_identifier.clone());
                                            }
                                        }
                                    } else if sub.identifier.eq(&sendcomp.0.clone()) {
                                        if let SubcomponentClassifier::ClassifierReference(
                                            classifier,
                                        ) = &sub.classifier
                                        {
                                            if let UniqueComponentClassifierReference::Implementation(unirf) = classifier {
                                                system_subcomponent_identify_to_type.insert(sub.identifier.clone(), unirf.implementation_name.type_identifier.clone());
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
}

//收集process内和thread之间的多连接关系
pub fn collect_thread_connections(
    thread_broadcast_receive: &mut HashMap<(String, String), Vec<(String, String)>>,
    process_subcomponent_identify_to_type: &mut HashMap<String, String>,
    pkg: &Package,
) {
    //List列表存储process内具有多连接关系的端口，每条数据是端口名
    let mut process_forwarder_broadcast_send: Vec<String> = Vec::new();

    if let Some(public_section) = &pkg.public_section {
        for decl in &public_section.declarations {
            if let AadlDeclaration::ComponentImplementation(impl_) = decl {
                if impl_.category == ComponentCategory::Process {
                    if let ConnectionClause::Items(connections) = &impl_.connections {
                        for conn in connections {
                            if let Connection::Port(port_conn) = conn {
                                if let PortEndpoint::ComponentPort(proc_port) = &port_conn.source {
                                    process_forwarder_broadcast_send.push(proc_port.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    //筛选重复出现的端口。保留。
    let temp_process_port =
        tool::dedup_with_min_two_unique_single_string(&mut process_forwarder_broadcast_send);
    for port in &temp_process_port {
        if let Some(public_section) = &pkg.public_section {
            for decl in &public_section.declarations {
                if let AadlDeclaration::ComponentImplementation(impl_) = decl {
                    if impl_.category == ComponentCategory::Process {
                        if let ConnectionClause::Items(connections) = &impl_.connections {
                            for conn in connections {
                                if let Connection::Port(port_conn) = conn {
                                    if let PortEndpoint::ComponentPort(proc_port) =
                                        &port_conn.source
                                    {
                                        if proc_port.eq(port) {
                                            if let PortEndpoint::SubcomponentPort {
                                                subcomponent,
                                                port,
                                            } = &port_conn.destination
                                            {
                                                thread_broadcast_receive
                                                    .entry((
                                                        port.clone(),
                                                        impl_.name.type_identifier.clone(),
                                                    ))
                                                    .or_insert(Vec::new())
                                                    .push((subcomponent.clone(), port.clone()));
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
    //遍历thread_broadcast_receive的键值对当中所有的值（不考虑键），把其中的组件名，在process中对应的子组件名，找到它的真实类型，并记录下来存储到process_subcomponent_identify_to_type。
    for (_, vercport) in thread_broadcast_receive {
        for (comp, _) in vercport {
            if let Some(public_section) = &pkg.public_section {
                for decl in &public_section.declarations {
                    if let AadlDeclaration::ComponentImplementation(impl_) = decl {
                        if impl_.category == ComponentCategory::Process {
                            if let SubcomponentClause::Items(subcomponents) = &impl_.subcomponents {
                                for sub in subcomponents {
                                    
                                    if sub.identifier.eq(&comp.clone()) {
                                        if let SubcomponentClassifier::ClassifierReference(
                                            classifier,
                                        ) = &sub.classifier
                                        {
                                            if let UniqueComponentClassifierReference::Implementation(unirf) = classifier {
                                                process_subcomponent_identify_to_type.insert(sub.identifier.clone(), unirf.implementation_name.type_identifier.clone());
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
}

// 生成CPU调度策略映射的静态代码
pub fn convert_cpu_schedule_mapping(
    module: &mut RustModule,
    cpu_scheduling_protocols: &HashMap<String, String>,
    cpu_name_to_id_mapping: &HashMap<String, isize>,
) {
    // 如果没有CPU映射信息，则不生成代码
    if cpu_name_to_id_mapping.is_empty() {
        return;
    }

    // 生成map.insert语句
    let mut map_insertions = Vec::new();

    for (cpu_name, cpu_id) in cpu_name_to_id_mapping {
        // 获取该CPU的调度协议
        let scheduling_protocol = cpu_scheduling_protocols
            .get(cpu_name)
            .map(|s| s.as_str())
            .unwrap_or("FIFO"); // 默认使用FIFO

        // 将调度协议转换为对应的常量
        let sched_constant = match scheduling_protocol.to_uppercase().as_str() {
            "POSIX_1003_HIGHEST_PRIORITY_FIRST_PROTOCOL" | "HPF" => "SCHED_FIFO",
            "ROUND_ROBIN_PROTOCOL" | "RR" => "SCHED_RR",
            "EDF" | "EARLIEST_DEADLINE_FIRST_PROTOCOL" => "SCHED_DEADLINE",
            "RATE_MONOTONIC_PROTOCOL" | "RMS" | "RM" => "SCHED_FIFO",
            "DEADLINE_MONOTONIC_PROTOCOL" | "DM" | "DMS" => "SCHED_FIFO",
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
        ty: Some(Type::Generic(
            "HashMap".to_string(),
            vec![
                Type::Named("isize".to_string()),
                Type::Named("i32".to_string()),
            ],
        )),
        init: Some(Expr::Call(
            Box::new(Expr::Path(
                vec!["HashMap".to_string(), "new".to_string()],
                PathType::Namespace,
            )),
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
        ty: Type::Generic(
            "HashMap".to_string(),
            vec![
                Type::Named("isize".to_string()),
                Type::Named("i32".to_string()),
            ],
        ),
        init: Block {
            stmts: init_stmts,
            expr: None,
        },
        vis: Visibility::Public,
        docs: vec!["// CPU ID到调度策略的映射".to_string()],
    };

    // 将 LazyStatic 添加到模块中
    module.items.push(Item::LazyStatic(lazy_static_def));
}

/// 添加 period_to_priority 函数到模块中
/// 该函数根据周期计算优先级：prio(P)=max(1,min(99,99−⌊k⋅log10(P)⌋))
/// 只有在检测到 RMS 或 DMS 调度协议时才生成此函数
pub fn add_period_to_priority_function(
    module: &mut RustModule,
    cpu_scheduling_protocols: &HashMap<String, String>,
) {
    // 检查是否有 RMS 或 DMS 调度协议
    let has_rms_or_dms = cpu_scheduling_protocols.values().any(|protocol| {
        let protocol_upper = protocol.to_uppercase();
        protocol_upper.contains("RATE_MONOTONIC")
            || protocol_upper.contains("RMS")
            || protocol_upper.contains("RM")
            || protocol_upper.contains("DEADLINE_MONOTONIC")
            || protocol_upper.contains("DMS")
            || protocol_upper.contains("DM")
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
    body_stmts.push(Statement::Expr(Expr::Ident(
        "return raw.max(1.0).min(99.0) as i32".to_string(),
    )));

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
            "// 用于 RMS (Rate Monotonic Scheduling) 和 DMS (Deadline Monotonic Scheduling)"
                .to_string(),
        ],
        attrs: Vec::new(),
    };

    // 将函数添加到模块中
    module.items.push(Item::Function(function_def));
}
