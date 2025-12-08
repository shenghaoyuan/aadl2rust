use crate::aadlAst2rustCode::intermediate_ast::*;

use crate::aadlAst2rustCode::converter::AadlConverter;
use crate::ast::aadl_ast_cj::*;

pub fn convert_device_component(temp_converter: &AadlConverter, comp: &ComponentType) -> Vec<Item> {
    let mut items = Vec::new();

    // 1. 结构体定义
    let mut fields = temp_converter.convert_type_features(&comp.features, comp.identifier.clone()); //特征列表（端口字段）

    // 添加周期字段
    fields.push(Field {
        name: "period_ms".to_string(),
        ty: Type::Named("u64".to_string()),
        docs: vec![format!(
            "// 周期：{}ms",
            extract_period(comp).unwrap_or(2000)
        )],
        attrs: Vec::new(),
    });

    let struct_name = format!("{}Device", comp.identifier.to_lowercase());
    let struct_def = StructDef {
        name: struct_name.clone(),
        fields,
        properties: Vec::new(),
        generics: Vec::new(),
        derives: vec!["Debug".to_string()],
        docs: vec![format!("// AADL Device: {}", comp.identifier)],
        vis: Visibility::Public,
    };
    items.push(Item::Struct(struct_def));

    // 2. 实现块（包含new和run方法）
    let mut impl_items = Vec::new();

    // 生成 new() 方法
    let period_ms = extract_period(comp).unwrap_or(2000);
    let new_method = create_device_new_method(comp, period_ms);
    impl_items.push(ImplItem::Method(new_method));

    // 生成 run() 方法
    let run_method = create_device_run_method(temp_converter, comp);
    impl_items.push(ImplItem::Method(run_method));

    let impl_block = ImplBlock {
        target: Type::Named(struct_name),
        generics: Vec::new(),
        items: impl_items,
        trait_impl: Some(Type::Named("Device".to_string())),
    };
    items.push(Item::Impl(impl_block));

    items
}

/// 创建 device 的 new() 方法
fn create_device_new_method(comp: &ComponentType, period_ms: u64) -> FunctionDef {
    let mut field_initializations = Vec::new();

    // 初始化所有端口字段为 None
    if let FeatureClause::Items(features) = &comp.features {
        for feature in features {
            if let Feature::Port(port) = feature {
                let port_name = port.identifier.to_lowercase();
                field_initializations.push(format!("            {}: None,", port_name));
            }
        }
    }

    // 初始化周期字段
    field_initializations.push(format!("            period_ms: {},", period_ms));

    // 创建结构体字面量返回语句
    let struct_literal = format!(
        "return Self {{\n{}\n        }}",
        field_initializations.join("\n")
    );

    // 创建方法体
    let body = Block {
        stmts: vec![Statement::Expr(Expr::Ident(struct_literal))],
        expr: None,
    };

    FunctionDef {
        name: "new".to_string(),
        params: Vec::new(),
        return_type: Type::Named("Self".to_string()),
        body,
        asyncness: false,
        vis: Visibility::None,
        docs: vec!["// Creates a new device instance".to_string()],
        attrs: Vec::new(),
    }
}

/// 创建 device 的 run() 方法
fn create_device_run_method(temp_converter: &AadlConverter, comp: &ComponentType) -> FunctionDef {
    let mut stmts = Vec::new();

    // 获取所有输入端口
    let mut input_ports = Vec::new();
    if let FeatureClause::Items(features) = &comp.features {
        for feature in features {
            if let Feature::Port(port) = feature {
                if port.direction == PortDirection::In {
                    input_ports.push(port.clone());
                }
            }
        }
    }

    // 获取所有输出端口
    let mut output_ports = Vec::new();
    if let FeatureClause::Items(features) = &comp.features {
        for feature in features {
            if let Feature::Port(port) = feature {
                if port.direction == PortDirection::Out {
                    output_ports.push(port.clone());
                }
            }
        }
    }

    // 创建周期 Duration
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
            vec![Expr::Path(
                vec!["self".to_string(), "period_ms".to_string()],
                PathType::Member,
            )],
        )),
    }));

    // 创建随机数生成器
    if !output_ports.is_empty() {
        stmts.push(Statement::Let(LetStmt {
            ifmut: true,
            name: "rng".to_string(),
            ty: None,
            init: Some(Expr::Call(
                Box::new(Expr::Path(
                    vec!["rand".to_string(), "thread_rng".to_string()],
                    PathType::Namespace,
                )),
                Vec::new(),
            )),
        }));
    }

    // 创建主循环
    let mut loop_stmts = Vec::new();

    // 记录开始时间
    loop_stmts.push(Statement::Let(LetStmt {
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
    }));
    //println!("type_mappings: {:?}", self.type_mappings);

    if !input_ports.is_empty() {
        loop_stmts.push(Statement::Comment(
            "// --- 从输入端口接收数据 ---".to_string(),
        ));
    }

    // 为每个输入端口尝试接收数据
    for port in &input_ports {
        let port_name = port.identifier.to_lowercase();
        let received_var = format!("{}_in_val", port_name.clone());
        let log_port_name = port_name.clone();
        let field_port_name = port_name.clone();

        let inner_then_block = Block {
            stmts: vec![
                Statement::Expr(Expr::Call(
                    Box::new(Expr::Path(
                        vec!["println!".to_string()],
                        PathType::Namespace,
                    )),
                    vec![
                        Expr::Literal(Literal::Str(format!(
                            "[{}] Received {}: {{:?}}",
                            comp.identifier.to_lowercase(),
                            log_port_name
                        ))),
                        Expr::Ident(received_var.clone()),
                    ],
                )),
                Statement::Comment("// TODO: 在此处加入执行逻辑".to_string()),
            ],
            expr: None,
        };

        let receive_block = Block {
            stmts: vec![Statement::Expr(Expr::IfLet {
                pattern: format!("Ok({})", received_var),
                value: Box::new(Expr::MethodCall(
                    Box::new(Expr::Ident("rx".to_string())),
                    "try_recv".to_string(),
                    Vec::new(),
                )),
                then_branch: inner_then_block,
                else_branch: None,
            })],
            expr: None,
        };

        loop_stmts.push(Statement::Expr(Expr::IfLet {
            pattern: "Some(rx)".to_string(),
            value: Box::new(Expr::Reference(
                Box::new(Expr::Path(
                    vec!["self".to_string(), field_port_name],
                    PathType::Member,
                )),
                true,
                false,
            )),
            then_branch: receive_block,
            else_branch: None,
        }));
    }

    // 为每个输出端口生成数据并发送
    for port in &output_ports {
        let port_name = port.identifier.to_lowercase();

        // 确定端口的数据类型
        let data_type = match &port.port_type {
            PortType::Data { classifier } | PortType::EventData { classifier } => classifier
                .as_ref()
                .map(|c| temp_converter.classifier_to_type(c))
                .unwrap_or(Type::Named("error_type".to_string())),
            PortType::Event => Type::Named("()".to_string()),
        };
        println!("data_type: {:?}", data_type);

        // 生成随机数据值（根据类型）
        let random_value = match &data_type {
            Type::Named(type_name) => {
                match type_name.as_str() {
                    "i32" | "i64" | "i16" | "i8" => {
                        // 生成 0-200 的随机整数
                        Expr::MethodCall(
                            Box::new(Expr::Ident("rng".to_string())),
                            "gen_range".to_string(),
                            vec![
                                Expr::Literal(Literal::Int(0)),
                                Expr::Literal(Literal::Int(201)),
                            ],
                        )
                    }
                    "u32" | "u64" | "u16" | "u8" => Expr::MethodCall(
                        Box::new(Expr::Ident("rng".to_string())),
                        "gen_range".to_string(),
                        vec![
                            Expr::Literal(Literal::Int(0)),
                            Expr::Literal(Literal::Int(201)),
                        ],
                    ),
                    "f32" | "f64" => Expr::MethodCall(
                        Box::new(Expr::Ident("rng".to_string())),
                        "gen_range".to_string(),
                        vec![
                            Expr::Literal(Literal::Float(0.0)),
                            Expr::Literal(Literal::Float(200.0)),
                        ],
                    ),
                    "bool" => {
                        Expr::MethodCall(
                            Box::new(Expr::Ident("rng".to_string())),
                            "gen_bool".to_string(),
                            vec![Expr::Literal(Literal::Float(0.9))], // 90%概率为true
                        )
                    }
                    "error_type" => Expr::Ident("please customize".to_string()),
                    _ => Expr::Ident("please customize".to_string()), // 默认值
                }
            }
            _ => Expr::Literal(Literal::Int(0)),
        };

        // 创建变量存储生成的值
        let value_var = format!("{}_val", port_name);
        loop_stmts.push(Statement::Let(LetStmt {
            ifmut: false,
            name: value_var.clone(),
            ty: None,
            init: Some(random_value),
        }));

        // 发送数据（如果端口存在）
        let send_block = Block {
            stmts: vec![
                Statement::Let(LetStmt {
                    ifmut: false,
                    name: "_".to_string(),
                    ty: None,
                    init: Some(Expr::MethodCall(
                        Box::new(Expr::Ident("tx".to_string())),
                        "send".to_string(),
                        vec![Expr::Ident(value_var.clone())],
                    )),
                }),
                Statement::Expr(Expr::Call(
                    Box::new(Expr::Path(
                        vec!["println!".to_string()],
                        PathType::Namespace,
                    )),
                    vec![
                        Expr::Literal(Literal::Str(format!(
                            "[{}] send {} = {{:?}}",
                            comp.identifier.to_lowercase(),
                            port_name
                        ))),
                        Expr::Ident(value_var),
                    ],
                )),
            ],
            expr: None,
        };

        loop_stmts.push(Statement::Expr(Expr::IfLet {
            pattern: "Some(tx)".to_string(),
            value: Box::new(Expr::Reference(
                Box::new(Expr::Path(
                    vec!["self".to_string(), port_name.clone()],
                    PathType::Member,
                )),
                true,
                false,
            )),
            then_branch: send_block,
            else_branch: None,
        }));
    }

    // 计算已用时间并睡眠剩余时间
    loop_stmts.push(Statement::Let(LetStmt {
        ifmut: false,
        name: "elapsed".to_string(),
        ty: None,
        init: Some(Expr::MethodCall(
            Box::new(Expr::Ident("start".to_string())),
            "elapsed".to_string(),
            Vec::new(),
        )),
    }));

    // 睡眠剩余时间
    loop_stmts.push(Statement::Expr(Expr::If {
        condition: Box::new(Expr::BinaryOp(
            Box::new(Expr::Ident("elapsed".to_string())),
            "<".to_string(),
            Box::new(Expr::Ident("period".to_string())),
        )),
        then_branch: Block {
            stmts: vec![Statement::Expr(Expr::MethodCall(
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
            ))],
            expr: None,
        },
        else_branch: None,
    }));

    // 将循环语句添加到主语句列表
    stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
        stmts: loop_stmts,
        expr: None,
    }))));

    FunctionDef {
        name: "run".to_string(),
        params: vec![Param {
            name: "self".to_string(),
            ty: Type::Named("Self".to_string()),
        }],
        return_type: Type::Unit,
        body: Block { stmts, expr: None },
        asyncness: false,
        vis: Visibility::None,
        docs: vec![
            "// Device execution entry point - periodically generates and sends data".to_string(),
        ],
        attrs: Vec::new(),
    }
}

fn extract_period(comp: &ComponentType) -> Option<u64> {
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
