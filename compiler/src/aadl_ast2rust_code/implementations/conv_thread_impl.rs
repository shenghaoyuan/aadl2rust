use crate::aadl_ast2rust_code::intermediate_ast::*;
use crate::aadl_ast2rust_code::converter::AadlConverter;
use crate::aadl_ast2rust_code::converter_annex::AnnexConverter;

use crate::ast::aadl_ast_cj::*;
use std::collections::HashMap;


pub fn convert_thread_implemenation(temp_converter: &mut AadlConverter, impl_: &ComponentImplementation) -> Vec<Item> {
    let mut items = Vec::new();

    // 1. 结构体定义
    let mut fields = Vec::new(); // 对于线程实现，没有特征，这里从属性生成字段
    let struct_name = format!("{}Thread", impl_.name.type_identifier.to_lowercase());
    let mut field_values = HashMap::new();

    // 将实现上的属性整合为字段，并存储属性值
    if let PropertyClause::Properties(props) = &impl_.properties {
        for prop in props {
            if let Property::BasicProperty(bp) = prop {
                if let Some(val) = temp_converter.parse_property_value(&bp.value) {
                    let field_name = bp.identifier.name.to_lowercase();
                    let ty_name = temp_converter.type_for_property(&val);
                    
                    // 存储属性值到 thread_field_values
                    field_values.insert(field_name.clone(), val.clone());
                    fields.push(Field {
                        name: field_name,
                        ty: Type::Named(ty_name),
                        docs: vec![format!("// AADL属性(impl): {}", bp.identifier.name)],
                        attrs: Vec::new(),
                    });
                }
            }
        }
    }
    
    // 将实现级别的属性值追加到 thread_field_values
    if !field_values.is_empty() {
        // 获取现有的字段值映射，如果不存在则创建新的
        let existing_values = temp_converter.thread_field_values.entry(struct_name.clone()).or_insert_with(HashMap::new);
        // 追加新的字段值，如果字段已存在则覆盖（实现级别的属性优先级更高）
        for (key, value) in field_values {
            existing_values.insert(key, value);
        }
    }
    //println!("!!!!!!!!!!!!thread_field_values: {:?}", self.thread_field_values);
    
    let struct_def = StructDef {
        name: format!("{}Thread", impl_.name.type_identifier.to_lowercase()),
        fields,
        properties: Vec::new(), // 属性字段已整合进 fields
        generics: Vec::new(),
        derives: vec!["Debug".to_string()],
        docs: temp_converter.create_component_impl_docs(impl_),
        vis: Visibility::Public, //默认public
    };
    items.push(Item::Struct(struct_def));

    // 2. 实现块（包含new和run方法）
    let mut impl_items = Vec::new();
    
    // 生成 new() 方法
    let mut flag_need_shared_variable_param = false;
    let new_method = create_thread_new_method(temp_converter, impl_, &mut flag_need_shared_variable_param);
    impl_items.push(ImplItem::Method(new_method));
    
    // 添加 run 方法
    impl_items.push(ImplItem::Method(FunctionDef {
        name: "run".to_string(),
        params: vec![Param {
            name: "".to_string(),
            ty: Type::Reference(Box::new(Type::Named("self".to_string())), false, true),
        }],
        return_type: Type::Unit,
        body: create_thread_run_body(temp_converter, impl_),
        asyncness: false,
        vis: Visibility::None,
        docs: vec![
            "// Thread execution entry point".to_string(),
            format!(
                "// Period: {:?} ms",
                extract_property_value(temp_converter, impl_, "period")
            ),
        ],
        attrs: Vec::new(),
    }));

    let impl_block = ImplBlock {
        target: Type::Named(format!(
            "{}Thread",
            impl_.name.type_identifier.to_lowercase()
        )),
        generics: Vec::new(),
        items: impl_items,
        trait_impl: Some(Type::Named("Thread".to_string())),
    };
    items.push(Item::Impl(impl_block));

    // 新增impl块，生成不包含在trait中的方法，带共享变量参数的new()方法。
    if flag_need_shared_variable_param {
        let items_no_trait = vec![ImplItem::Method(create_thread_new_method(temp_converter, impl_, &mut flag_need_shared_variable_param))];
        let impl_block_no_trait = ImplBlock {
            target: Type::Named(format!("{}Thread", impl_.name.type_identifier.to_lowercase())),
            generics: Vec::new(),
            items: items_no_trait,
            trait_impl: None,
        };
        items.push(Item::Impl(impl_block_no_trait));
    }
    

    items
}

/// 创建线程的 new() 方法
/// 使用存储的属性值初始化线程结构体字段
fn create_thread_new_method(temp_converter: &mut AadlConverter, impl_: &ComponentImplementation, flag_need_shared_variable_param: &mut bool) -> FunctionDef {
    let struct_name = format!("{}Thread", impl_.name.type_identifier.to_lowercase());
    let key = struct_name.clone();
    
    // 获取存储的属性值
    let field_values = temp_converter.thread_field_values.get(&key).cloned().unwrap_or_default();
    let field_types = temp_converter.thread_field_types.get(&key).cloned().unwrap_or_default();
    
    // 生成结构体字面量初始化字符串
    let mut field_initializations = Vec::new();
    let mut params = vec![Param {
        name: "cpu_id".to_string(),
        ty: Type::Named("isize".to_string()),
    }];
    
    // 为每个字段生成初始化表达式和注释
    for (field_name, prop_value) in &field_values {
        let mut init_value = property_value_to_initializer(prop_value);
        let comment = format!("");

        // 对以"Shared"结尾的字段类型添加参数（已删去），并修改初始化值
        if let Some(field_type) = field_types.get(field_name) {
            match field_type {
                Type::Named(type_name) => {
                    if type_name.ends_with("Shared") {
                        if !*flag_need_shared_variable_param {
                            *flag_need_shared_variable_param = true;

                             // 生成 Arc::new(Mutex::new(TypeName::new())) 格式的初始化值
                            let base_type_name = type_name.trim_end_matches("Shared");
                            init_value = format!("Arc::new(Mutex::new({}::new()))", base_type_name);
                        } else{
                            params.push(Param {
                                name: field_name.clone(),
                                ty: field_type.clone(),
                            });
                        }
                    }
                }
                _ => {
                    // 其他类型暂时不用作为new()的参数
                }
            }
        }
        // 字段赋值
        field_initializations.push(format!("            {}: {}, {}", field_name, init_value, comment));
    }
    
    // 添加 CPU ID 字段初始化
    field_initializations.push("            cpu_id: cpu_id, // CPU ID".to_string());
    
    // 创建结构体字面量返回语句
    let struct_literal = format!("return Self {{\n{}\n        }}", field_initializations.join("\n"));
    
    // 创建方法体
    let body = Block {
        stmts: vec![Statement::Expr(Expr::Ident(struct_literal))],
        expr: None,
    };
    
    FunctionDef {
        name: "new".to_string(),
        params,
        return_type: Type::Named("Self".to_string()),
        body,
        asyncness: false,
        vis: Visibility::None,
        docs: vec!["// 创建组件并初始化AADL属性".to_string()],
        attrs: Vec::new(),
    }
}


/// 将属性值转换为初始化表达式字符串
fn property_value_to_initializer(val: &StruPropertyValue) -> String {
    match val {
        StruPropertyValue::Boolean(b) => b.to_string(),
        StruPropertyValue::Integer(i) => i.to_string(),
        StruPropertyValue::Float(f) => {
            let s = f.to_string();
            if f.fract() == 0.0 && !s.contains('.') { format!("{s}.0") } else { s }
        }
        StruPropertyValue::String(s) => format!("\"{}\".to_string()", s),
        StruPropertyValue::Duration(v, _unit) => v.to_string(),
        StruPropertyValue::Range(min, max, _unit) => format!("({}, {})", min, max),
        StruPropertyValue::None => "None".to_string(),
        StruPropertyValue::Custom(s) => s.to_string(),
    }
}

/// 创建线程的 run() 方法体
/// 该方法生成线程的执行逻辑，包括：
/// 1. 线程优先级和CPU亲和性设置
/// 2. 根据调度协议生成不同的执行逻辑
/// 3. 子程序调用处理（参数端口、共享变量、普通调用）
fn create_thread_run_body(temp_converter: &mut AadlConverter, impl_: &ComponentImplementation) -> Block {
    let mut stmts = Vec::new();
    
    //======================= 线程优先级设置 ========================
    // 检查是否有优先级属性
    let priority = extract_property_value(temp_converter, impl_, "priority");
    let period = extract_property_value(temp_converter, impl_, "period");
    
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
    } else if period.is_some() {
        // 如果没有优先级但有周期，则根据周期计算优先级(RMS)
        // TODO:不知道这里为什么没有使用period
        stmts.push(Statement::Expr(Expr::Unsafe(Box::new(Block {
            stmts: vec![
                // let prio = period_to_priority(self.period as f64); 不知道这句为什么要注释
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
    let dispatch_protocol = extract_dispatch_protocol(temp_converter, impl_);
    
    // ==================== 步骤 2: 根据调度协议生成不同的执行逻辑 ====================
    match dispatch_protocol.as_deref() {
        Some("Periodic") => {
            // 周期性调度：生成周期性执行循环
            stmts.extend(create_periodic_execution_logic(temp_converter, impl_));
        }
        Some("Aperiodic") => {
            // 非周期性调度：生成事件驱动执行逻辑
            stmts.extend(create_aperiodic_execution_logic(temp_converter, impl_));
        }
        Some("Sporadic") => {
            // 偶发调度：生成偶发执行逻辑
            stmts.extend(create_sporadic_execution_logic(temp_converter, impl_));
        }
        Some("Timed") => {
            // 定时调度：生成定时执行逻辑
            stmts.extend(create_timed_execution_logic(temp_converter, impl_));
        }
        _ => {
            // 默认使用周期性调度
            stmts.extend(create_periodic_execution_logic(temp_converter, impl_));
        }
    }

    Block { stmts, expr: None }
}

/// 创建周期性执行逻辑
fn create_periodic_execution_logic(temp_converter: &AadlConverter, impl_: &ComponentImplementation) -> Vec<Statement> {
    let mut stmts = Vec::new();
    
    // 从AADL属性中提取周期值，默认为2000ms
    let period = extract_property_value(temp_converter, impl_, "period").unwrap_or(2000);
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

    //添加let mut next_release = Instant::now() + period;
    stmts.push(Statement::Let(LetStmt {
        ifmut: true,
        name: "next_release".to_string(),
        ty: None,
        init: Some(Expr::BinaryOp(
            Box::new(Expr::Ident("Instant::now()".to_string())),
            "+".to_string(),
            Box::new(Expr::Ident("period".to_string())),
        )),
    }));
    
    
    // 处理BA
    let mut annex_converter = AnnexConverter::default();
    // 检查是否有Behavior Annex
    let mut if_has_ba = false;
    
    if annex_converter.find_behavior_annex(impl_).is_some(){
        if_has_ba = true;
        stmts.extend(annex_converter.generate_ba_variables_states(impl_, annex_converter.find_behavior_annex(impl_).unwrap()));
    } 

    let mut ba_stmts = Vec::new();
    if if_has_ba {
        if let Some(transitions) = annex_converter.find_behavior_annex(impl_).unwrap().transitions.clone() {
            ba_stmts.extend(annex_converter.generate_state_machine_loop(&transitions));
        }
    };

    // 子程序调用处理代码
    let subprogram_handling_stmts = create_subprogram_call_logic(temp_converter, impl_);


    // 构造 loop 内部的语句列表,调度控制+子程序调用+BA执行
    let mut loop_stmts: Vec<Statement> = Vec::new();

    // 1. let now = Instant::now();
    loop_stmts.push(Statement::Let(LetStmt {
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
    }));

    // 2. if now < next_release { std::thread::sleep(next_release - now); }
    loop_stmts.push(Statement::Expr(Expr::If {
        condition: Box::new(Expr::BinaryOp(
            Box::new(Expr::Ident("now".to_string())),
            "<".to_string(),
            Box::new(Expr::Ident("next_release".to_string())),
        )),
        then_branch: Block {
            stmts: vec![Statement::Expr(Expr::MethodCall(
                Box::new(Expr::Path(
                    vec![
                        "std".to_string(),
                        "thread".to_string(),
                        "sleep".to_string(),
                    ],
                    PathType::Namespace,
                )),
                "".to_string(),
                vec![Expr::Ident("next_release - now".to_string())],
            ))],
            expr: None,
        },
        else_branch: None,
    }));

    // 3. 端口处理块
    if !subprogram_handling_stmts.is_empty() {
        loop_stmts.push(Statement::Expr(Expr::Block(Block {
            stmts: subprogram_handling_stmts,
            expr: None,
        })));
    }

    // 4. 如果存在 Behavior Annex，则插入 BA 执行块
    if !ba_stmts.is_empty() {
        loop_stmts.push(Statement::Expr(Expr::Block(Block {
            stmts: ba_stmts,
            expr: None,
        })));
    }

    // 5. next_release += period;
    loop_stmts.push(Statement::Expr(Expr::BinaryOp(
        Box::new(Expr::Ident("next_release".to_string())),
        "+=".to_string(),
        Box::new(Expr::Ident("period".to_string())),
    )));

    // 6. 构造 loop 表达式并压入外层 stmts
    stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
        stmts: loop_stmts,
        expr: None,
    }))));


    stmts
}

/// 创建非周期性执行逻辑
fn create_aperiodic_execution_logic(temp_converter: &AadlConverter, impl_: &ComponentImplementation) -> Vec<Statement> {
    let mut stmts = Vec::new();

    // 获取事件端口信息（事件端口或事件数据端口）
    let event_ports = extract_event_ports(temp_converter, impl_);

    // 提取事件端口的优先级信息
    let port_urgency = extract_event_port_urgency(impl_);
    //println!("port_urgency: {:?}", port_urgency);
    
    // 如果没有找到事件端口，则从参数连接中获取接收端口作为备选
    let receive_ports = if !event_ports.is_empty() {
        event_ports
    } else {
        Vec::new()
        // let subprogram_calls = extract_subprogram_calls(temp_converter, impl_);
        // subprogram_calls.iter()
        //     .filter(|(_, _, _, is_send, _)| !is_send)
        //     .map(|(_, _, thread_port_name, _, _)| thread_port_name.clone())
        //     .collect()
    };

    // 检查是否有需要端口数据的子程序调用
    let subprogram_calls = extract_subprogram_calls(temp_converter, impl_);
    let has_receiving_subprograms = subprogram_calls.iter().any(|(_, _, _, is_send, _)| !is_send);

    // 在循环外定义 events 变量
    stmts.push(Statement::Let(LetStmt {
        ifmut: true,
        name: "events".to_string(),
        ty: None,
        init: Some(Expr::Call(
            Box::new(Expr::Path(vec!["Vec".to_string(), "new".to_string()], PathType::Namespace)),
            Vec::new(),
        )),
    }));

    // 生成非周期性执行逻辑 - 按优先级处理事件
    stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
        stmts: {
            let mut loop_stmts = Vec::new();
            
            // 添加生成事件收集逻辑
            loop_stmts.extend(create_event_collection_logic(&port_urgency, &receive_ports));
            
            // 如果事件队列中有事件，则挑选出优先级最高的进行处理
            loop_stmts.push(Statement::Expr(Expr::IfLet {
                pattern: "Some((idx, (val, _urgency, _ts)))".to_string(),
                value: Box::new(Expr::MethodCall(
                    Box::new(Expr::MethodCall(
                        Box::new(Expr::MethodCall(
                            Box::new(Expr::Ident("events".to_string())),
                            "iter".to_string(),
                            Vec::new(),
                        )),
                        "enumerate".to_string(),
                        Vec::new(),
                    )),
                    "max_by".to_string(),
                    vec![Expr::Ident("|a, b| match a.1.1.cmp(&b.1.1) {\n                        std::cmp::Ordering::Equal => b.1.2.cmp(&a.1.2),\n                        other => other,\n                    }".to_string())],
                )),
                then_branch: Block {
                    stmts: vec![
                        // 移除已处理事件
                        Statement::Let(LetStmt {
                            ifmut: false,
                            name: "(val, _, _)".to_string(),
                            ty: None,
                            init: Some(Expr::MethodCall(
                                Box::new(Expr::Ident("events".to_string())),
                                "remove".to_string(),
                                vec![Expr::Ident("idx".to_string())],
                            )),
                        }),
                        // 执行子程序调用处理
                        Statement::Expr(Expr::Block(Block {
                            stmts: create_subprogram_call_logic_with_data(temp_converter, impl_, has_receiving_subprograms),
                            expr: None,
                        })),
                    ],
                    expr: None,
                },
                else_branch: Some(Block {
                    stmts: vec![
                        // 如果没有事件，短暂休眠避免忙等待
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
                                vec![Expr::Literal(Literal::Int(1))], // 1ms
                            )],
                        )),
                    ],
                    expr: None,
                }),
            }));
            
            loop_stmts
        },
        expr: None,
    }))));

    stmts
}

/// 创建偶发执行逻辑
fn create_sporadic_execution_logic(temp_converter: &AadlConverter, impl_: &ComponentImplementation) -> Vec<Statement> {
    let mut stmts = Vec::new();
    
    // 从AADL属性中提取最小间隔时间，默认为1000ms
    let min_interval = extract_property_value(temp_converter, impl_, "period").unwrap_or(1000);
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
    let event_ports = extract_event_ports(temp_converter, impl_);

    // 提取事件端口的优先级信息
    let port_urgency = extract_event_port_urgency(impl_);
    //println!("port_urgency: {:?}", port_urgency);
    
    // 如果没有找到事件端口，则从参数连接中获取接收端口作为备选
    let receive_ports = if !event_ports.is_empty() {
        event_ports
    } else {
        Vec::new()
        // let subprogram_calls = extract_subprogram_calls(temp_converter, impl_);
        // subprogram_calls.iter()
        //     .filter(|(_, _, _, is_send, _)| !is_send)
        //     .map(|(_, _, thread_port_name, _, _)| thread_port_name.clone())
        //     .collect()
    };

    // 检查是否有需要端口数据的子程序调用
    let subprogram_calls = extract_subprogram_calls(temp_converter, impl_);
    let has_receiving_subprograms = subprogram_calls.iter().any(|(_, _, _, is_send, _)| !is_send);

    // 在循环外定义 events 变量
    stmts.push(Statement::Let(LetStmt {
        ifmut: true,
        name: "events".to_string(),
        ty: None,
        init: Some(Expr::Call(
            Box::new(Expr::Path(vec!["Vec".to_string(), "new".to_string()], PathType::Namespace)),
            Vec::new(),
        )),
    }));

    // 生成偶发执行逻辑 - 按优先级处理事件
    stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
        stmts: {
            let mut loop_stmts = Vec::new();
            
            // 添加生成事件收集逻辑
            loop_stmts.extend(create_event_collection_logic(&port_urgency, &receive_ports));
            
            // 如果事件队列中有事件，则挑选出优先级最高的进行处理
            loop_stmts.push(Statement::Expr(Expr::IfLet {
                pattern: "Some((idx, (val, _urgency, _ts)))".to_string(),
                value: Box::new(Expr::MethodCall(
                    Box::new(Expr::MethodCall(
                        Box::new(Expr::MethodCall(
                            Box::new(Expr::Ident("events".to_string())),
                            "iter".to_string(),
                            Vec::new(),
                        )),
                        "enumerate".to_string(),
                        Vec::new(),
                    )),
                    "max_by".to_string(),
                    vec![Expr::Ident("|a, b| match a.1.1.cmp(&b.1.1) {\n                        std::cmp::Ordering::Equal => b.1.2.cmp(&a.1.2),\n                        other => other,\n                    }".to_string())],
                )),
                then_branch: Block {
                    stmts: vec![
                        // 移除已处理事件
                        Statement::Let(LetStmt {
                            ifmut: false,
                            name: "(val, _, _)".to_string(),
                            ty: None,
                            init: Some(Expr::MethodCall(
                                Box::new(Expr::Ident("events".to_string())),
                                "remove".to_string(),
                                vec![Expr::Ident("idx".to_string())],
                            )),
                        }),
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
                            stmts: create_subprogram_call_logic_with_data(temp_converter, impl_, has_receiving_subprograms),
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
                else_branch: Some(Block {
                    stmts: vec![
                        // 如果没有事件，短暂休眠避免忙等待
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
                                vec![Expr::Literal(Literal::Int(1))], // 1ms
                            )],
                        )),
                    ],
                    expr: None,
                }),
            }));
            
            loop_stmts
        },
        expr: None,
    }))));

    stmts
}

/// 创建定时执行逻辑
fn create_timed_execution_logic(temp_converter: &AadlConverter, impl_: &ComponentImplementation) -> Vec<Statement> {
    let mut stmts = Vec::new();
    
    // 从AADL属性中提取最小间隔时间，默认为1000ms
    let period = extract_property_value(temp_converter, impl_, "period").unwrap_or(1000);
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

    // 记录开始时间
    stmts.push(Statement::Let(LetStmt {
        ifmut: true,
        name: "start_time".to_string(),
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
    let event_ports = extract_event_ports(temp_converter, impl_);

    // 提取事件端口的优先级信息
    let port_urgency = extract_event_port_urgency(impl_);
    //println!("port_urgency: {:?}", port_urgency);
    
    // 如果没有找到事件端口，则从参数连接中获取接收端口作为备选
    let receive_ports = if !event_ports.is_empty() {
        event_ports
    } else {
        Vec::new()
        // let subprogram_calls = extract_subprogram_calls(temp_converter,impl_);
        // subprogram_calls.iter()
        //     .filter(|(_, _, _, is_send, _)| !is_send)
        //     .map(|(_, _, thread_port_name, _, _)| thread_port_name.clone())
        //     .collect()
    };

    // 检查是否有需要端口数据的子程序调用
    let subprogram_calls = extract_subprogram_calls(temp_converter,impl_);
    let has_receiving_subprograms = subprogram_calls.iter().any(|(_, _, _, is_send, _)| !is_send);

    // 在循环外定义 events 变量
    stmts.push(Statement::Let(LetStmt {
        ifmut: true,
        name: "events".to_string(),
        ty: None,
        init: Some(Expr::Call(
            Box::new(Expr::Path(vec!["Vec".to_string(), "new".to_string()], PathType::Namespace)),
            Vec::new(),
        )),
    }));

    // 生成定时执行逻辑 - 按优先级处理事件，支持超时
    stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
        stmts: {
            let mut loop_stmts = Vec::new();
            
            // 添加生成事件收集逻辑
            loop_stmts.extend(create_event_collection_logic(&port_urgency, &receive_ports));
            
            // 如果事件队列中有事件，则挑选出优先级最高的进行处理
            loop_stmts.push(Statement::Expr(Expr::IfLet {
                pattern: "Some((idx, (val, _urgency, _ts)))".to_string(),
                value: Box::new(Expr::MethodCall(
                    Box::new(Expr::MethodCall(
                        Box::new(Expr::MethodCall(
                            Box::new(Expr::Ident("events".to_string())),
                            "iter".to_string(),
                            Vec::new(),
                        )),
                        "enumerate".to_string(),
                        Vec::new(),
                    )),
                    "max_by".to_string(),
                    vec![Expr::Ident("|a, b| match a.1.1.cmp(&b.1.1) {\n                        std::cmp::Ordering::Equal => b.1.2.cmp(&a.1.2),\n                        other => other,\n                    }".to_string())],
                )),
                then_branch: Block {
                    stmts: vec![
                        // 移除已处理事件
                        Statement::Let(LetStmt {
                            ifmut: false,
                            name: "(val, _, _)".to_string(),
                            ty: None,
                            init: Some(Expr::MethodCall(
                                Box::new(Expr::Ident("events".to_string())),
                                "remove".to_string(),
                                vec![Expr::Ident("idx".to_string())],
                            )),
                        }),
                        // --- Compute Entrypoint (正常触发) ---
                        Statement::Expr(Expr::Block(Block {
                            stmts: create_subprogram_call_logic_with_data(temp_converter, impl_, has_receiving_subprograms),
                            expr: None,
                        })),
                    ],
                    expr: None,
                },
                else_branch: Some(Block {
                    stmts: vec![
                        // 检查是否超时
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
                        Statement::Let(LetStmt {
                            ifmut: false,
                            name: "elapsed".to_string(),
                            ty: None,
                            init: Some(Expr::MethodCall(
                                Box::new(Expr::Ident("now".to_string())),
                                "duration_since".to_string(),
                                vec![Expr::Ident("start_time".to_string())],
                            )),
                        }),
                        Statement::Expr(Expr::If {
                            condition: Box::new(Expr::BinaryOp(
                                Box::new(Expr::Ident("elapsed".to_string())),
                                ">".to_string(),
                                Box::new(Expr::Ident("period".to_string())),
                            )),
                            then_branch: Block {
                                stmts: vec![
                                    // 输出超时报错信息
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
                            else_branch: None,
                        }),
                    ],
                    expr: None,
                }),
            }));
            
            loop_stmts
        },
        expr: None,
    }))));

    stmts
}

/// 创建子程序调用处理逻辑（提取公共部分）
fn create_subprogram_call_logic(temp_converter: &AadlConverter, impl_: &ComponentImplementation) -> Vec<Statement> {
    create_subprogram_call_logic_with_data(temp_converter, impl_, false)
}

/// 创建子程序调用处理逻辑（带数据参数版本）
fn create_subprogram_call_logic_with_data(temp_converter: &AadlConverter, impl_: &ComponentImplementation, has_receiving_subprograms: bool) -> Vec<Statement> {
    let mut port_handling_stmts = Vec::new();

    // 提取有参数端口的子程序调用信息
    let subprogram_calls = extract_subprogram_calls(temp_converter, impl_);
    
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
                    mycalls_sequence.push((subprocall.identifier.to_lowercase(), subprogram_name));
                }
            }
        }
    }
    
    // 提取共享变量访问信息
    let data_access_calls = extract_data_access_calls(impl_);
    
    // 创建子程序调用映射
    let mut shared_var_subprograms = std::collections::HashMap::new();
    for (subprogram_name, _, shared_var_field) in &data_access_calls {
        shared_var_subprograms.insert(subprogram_name.clone(), shared_var_field.clone());
    }
    
    // 创建子程序集合
    let subprograms_with_ports: std::collections::HashSet<String> = subprogram_calls.iter()
        .map(|(_, spg_name, _, _, _)| spg_name.clone())
        .collect();
    // println!("subprograms_with_ports: {:?}", subprograms_with_ports);
    // 添加调用序列注释
    if !mycalls_sequence.is_empty() {
        let call_sequence = mycalls_sequence.iter()
            .map(|(call_id, _)| format!("{}()", call_id))
            .collect::<Vec<_>>()
            .join(" -> ");
        
        port_handling_stmts.push(Statement::Expr(Expr::Ident(format!(
            "// --- 调用序列(等价 AADL 的 Wrapper)---\n                           // {}",
            call_sequence
        ))));
    }

    // 根据Mycalls中的顺序处理所有子程序调用
    for (call_id, subprogram_name) in mycalls_sequence {
        let has_parameter_ports = subprograms_with_ports.contains(&call_id);
        
        port_handling_stmts.push(Statement::Expr(Expr::Ident(format!("// {}", call_id))));
        
        if has_parameter_ports {
            // 有参数端口的子程序处理
            if let Some((_, _, thread_port_name, is_send, port_type)) = subprogram_calls.iter()
                .find(|(_, spg_identifier, _, _, _)| spg_identifier == &call_id) {

                if *is_send {
                    // 发送模式
                    let mut send_stmts = Vec::new();
                    
                    // 根据端口类型生成合适的默认值
                    let default_value = temp_converter.generate_default_value_for_type(port_type);
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
                                    pattern: "Err(crossbeam_channel::TryRecvError::Empty)".to_string(),
                                    guard: None,
                                    body: Block { stmts: vec![], expr: None },
                                },
                                MatchArm {
                                    pattern: "Err(crossbeam_channel::TryRecvError::Disconnected)".to_string(),
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
                                        vec!["guard".to_string(), subprogram_name.clone()],
                                        PathType::Member,
                                    )),
                                    vec![],
                                )),
                                // 从read_pos::call(&mut guard.field) 改为了-> guard.read_pos();
                                // Statement::Expr(Expr::Call(
                                //     Box::new(Expr::Path(
                                //         vec![subprogram_name.clone(), "call".to_string()],
                                //         PathType::Namespace,
                                //     )),
                                //     vec![Expr::Reference(
                                //         Box::new(Expr::Ident("guard".to_string())),
                                //         true,
                                //         true,
                                //     )],
                                // )),
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
fn extract_property_value(temp_converter: &AadlConverter, impl_: &ComponentImplementation, name: &str) -> Option<u64> {
    let target_name = name.to_lowercase();
    for prop in temp_converter.convert_properties(ComponentRef::Impl(impl_)) {
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
fn extract_dispatch_protocol(temp_converter: &AadlConverter, impl_: &ComponentImplementation) -> Option<String> {
    let target_name = "dispatch_protocol";
    for prop in temp_converter.convert_properties(ComponentRef::Impl(impl_)) {
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

// 提取事件端口和事件数据端口
fn extract_event_ports(temp_converter: &AadlConverter, impl_: &ComponentImplementation) -> Vec<String> {
    let mut event_ports = Vec::new();
    
    // 从组件类型中获取端口定义
    if let Some(comp_type) = temp_converter.get_component_type(impl_) {
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

// 辅助函数：提取事件端口的优先级信息
fn extract_event_port_urgency(impl_: &ComponentImplementation) -> Vec<(String, u32)> {
    let mut port_priorities = Vec::new();
    
    // 获取组件的属性
    let properties = match &impl_.properties {
        PropertyClause::Properties(props) => props,
        _ => return port_priorities,
    };
    
    // 查找 urgency 属性
    for prop in properties {
        if let Property::BasicProperty(bp) = prop {
            if bp.identifier.name.to_lowercase() == "urgency" {
                // 解析属性值
                if let PropertyValue::Single(PropertyExpression::Apply(apply_term)) = &bp.value {
                    // 解析优先级数值
                    if let Ok(priority) = apply_term.number.parse::<u32>() {
                        port_priorities.push((apply_term.applies_to.clone(), priority));
                    }
                }
            }
        }
    }
    
    // 按优先级降序排序（优先级高的在前）
    port_priorities.sort_by(|a, b| b.1.cmp(&a.1));
    port_priorities
}

fn extract_subprogram_calls(
    temp_converter: &AadlConverter,
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
                                            data_subcomponent:_,
                                        } = &port_conn.destination
                                        {
                                            let thread_port_name = parameter.to_lowercase();
                                            let port_type = get_subprogram_port_type(temp_converter, &subprogram_name, &sou_parameter);
                                            calls.push((
                                                sou_parameter.to_lowercase(),    // 子程序端口名
                                                //subprogram_name.to_lowercase(),  // 子程序名
                                                subprogram_identifier.to_lowercase(),  // 子程序标识符
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
                                            data_subcomponent:_,
                                        } = &port_conn.source
                                        {
                                            let thread_port_name = parameter.to_lowercase();
                                            let port_type = get_subprogram_port_type(temp_converter, &subprogram_name, &des_parameter);
                                            calls.push((
                                                des_parameter.to_lowercase(),
                                                //subprogram_name.to_lowercase(),
                                                subprogram_identifier.to_lowercase(),
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

// 根据子程序名和端口名获取端口类型
fn get_subprogram_port_type(temp_converter: &AadlConverter, subprogram_name: &str, port_name: &str) -> Type {
    // 遍历所有组件类型，查找子程序类型
    for comp_type in temp_converter.component_types.values() {
        if comp_type.identifier.to_lowercase() == subprogram_name.to_lowercase() {
            // 找到子程序类型，查找其中的端口
            if let FeatureClause::Items(features) = &comp_type.features {
                for feature in features {
                    if let Feature::Port(port) = feature {
                        if port.identifier.to_lowercase() == port_name.to_lowercase() {
                            // 找到匹配的端口，返回其类型
                            return temp_converter.convert_paramport_type(port);
                        }
                    }
                }
            }
        }
    }
    // 如果找不到，返回默认类型
    Type::Named("i32".to_string())
}



    /// 创建事件收集代码
    fn create_event_collection_logic(port_urgency: &[(String, u32)], receive_ports: &[String]) -> Vec<Statement> {
        let mut stmts = Vec::new();
        
        // 只有当事件队列为空时才尝试接收新消息
        stmts.push(Statement::Expr(Expr::If {
            condition: Box::new(Expr::MethodCall(
                Box::new(Expr::Ident("events".to_string())),
                "is_empty".to_string(),
                Vec::new(),
            )),
            then_branch: Block {
                stmts: {
                    let mut collect_stmts = Vec::new();
                    
                    // 为每个有优先级的端口生成事件收集代码
                    for (port_name, urgency) in port_urgency {
                        let port_field_name = port_name.to_lowercase();
                        
                        // 生成 if let Some(rx) = &self.port_name 的代码
                        collect_stmts.push(Statement::Expr(Expr::IfLet {
                            pattern: "Some(rx)".to_string(),
                            value: Box::new(Expr::Reference(
                                Box::new(Expr::Path(
                                    vec!["self".to_string(), port_field_name.clone()],
                                    PathType::Member,
                                )),
                                true,
                                false,
                            )),
                            then_branch: Block {
                                stmts: vec![
                                    // 生成 if let Ok(val) = rx.try_recv() 的代码
                                    Statement::Expr(Expr::IfLet {
                                        pattern: "Ok(val)".to_string(),
                                        value: Box::new(Expr::MethodCall(
                                            Box::new(Expr::Ident("rx".to_string())),
                                            "try_recv".to_string(),
                                            Vec::new(),
                                        )),
                                        then_branch: Block {
                                            stmts: vec![
                                                // events.push((val, urgency, ts))
                                                Statement::Expr(Expr::MethodCall(
                                                    Box::new(Expr::Ident("events".to_string())),
                                                    "push".to_string(),
                                                    vec![Expr::Call(
                                                        Box::new(Expr::Ident("".to_string())), // 空标识符表示元组构造
                                                        vec![
                                                            Expr::Ident("val".to_string()),
                                                            Expr::Literal(Literal::Int(*urgency as i64)),
                                                            Expr::Ident("Instant::now()".to_string()),
                                                        ],
                                                    )],
                                                    
                                                )),
                                            ],
                                            expr: None,
                                        },
                                        else_branch: None,
                                    }),
                                ],
                                expr: None,
                            },
                            else_branch: None,
                        }));
                    }
                    
                    // 如果没有优先级信息，使用原来的逻辑处理接收端口
                    if port_urgency.is_empty() && !receive_ports.is_empty() {
                        let port_field_name = receive_ports[0].to_lowercase();
                        collect_stmts.push(Statement::Expr(Expr::IfLet {
                            pattern: "Some(rx)".to_string(),
                            value: Box::new(Expr::Reference(
                                Box::new(Expr::Path(
                                    vec!["self".to_string(), port_field_name],
                                    PathType::Member,
                                )),
                                true,
                                false,
                            )),
                            then_branch: Block {
                                stmts: vec![
                                    Statement::Expr(Expr::IfLet {
                                        pattern: "Ok(val)".to_string(),
                                        value: Box::new(Expr::MethodCall(
                                            Box::new(Expr::Ident("rx".to_string())),
                                            "try_recv".to_string(),
                                            Vec::new(),
                                        )),
                                        then_branch: Block {
                                            stmts: vec![
                                                Statement::Expr(Expr::MethodCall(
                                                    Box::new(Expr::Ident("events".to_string())),
                                                    "push".to_string(),
                                                    vec![Expr::Call(
                                                        Box::new(Expr::Ident("".to_string())), // 空标识符表示元组构造
                                                        vec![
                                                            Expr::Ident("val".to_string()),
                                                            Expr::Literal(Literal::Int(0)),
                                                            Expr::Ident("Instant::now()".to_string()),
                                                        ],
                                                    )],
                                                    
                                                )),
                                            ],
                                            expr: None,
                                        },
                                        else_branch: None,
                                    }),
                                ],
                                expr: None,
                            },
                            else_branch: None,
                        }));
                    }
                    
                    collect_stmts
                },
                expr: None,
            },
            else_branch: None,
        }));
        
        stmts
    }

    /// 提取data access连接，识别哪些子程序使用共享变量
    /// 返回：(子程序名, 共享变量名, 共享变量全小写名)
    fn extract_data_access_calls(impl_: &ComponentImplementation) -> Vec<(String, String, String)> {
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
                                // 从数据名称中提取共享变量全小写名，TODO:这里shared_var_field不是字段名
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
