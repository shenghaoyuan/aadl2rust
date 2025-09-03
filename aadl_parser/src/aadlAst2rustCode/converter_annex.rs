// src/aadlAst2rustCode/converter_annex.rs
// Behavior Annex 代码生成器

use super::intermediate_ast::*;
use crate::ast::aadl_ast_cj::*;
use std::collections::HashMap;

/// Behavior Annex 代码生成器
pub struct AnnexConverter;

impl Default for AnnexConverter {
    fn default() -> Self {
        Self
    }
}

impl AnnexConverter {
    /// 为线程实现生成Behavior Annex代码
    pub fn generate_annex_code(&self, impl_: &ComponentImplementation) -> Option<Vec<Statement>> {
        // 查找Behavior Annex
        if let Some(behavior_annex) = self.find_behavior_annex(impl_) {
            // 生成状态机代码
            self.generate_state_machine_code(impl_, behavior_annex)
        } else {
            None
        }
    }

    /// 查找Behavior Annex
    fn find_behavior_annex<'a>(&self, impl_: &'a ComponentImplementation) -> Option<&'a BehaviorAnnexContent> {
        for annex in &impl_.annexes {
            if let AnnexContent::BehaviorAnnex(content) = &annex.content {
                return Some(content);
            }
        }
        None
    }

    /// 生成状态机代码
    fn generate_state_machine_code(&self, impl_: &ComponentImplementation, behavior_annex: &BehaviorAnnexContent) -> Option<Vec<Statement>> {
        let mut stmts = Vec::new();

        // 1. 定义局部变量
        if let Some(state_vars) = &behavior_annex.state_variables {
            stmts.extend(self.generate_state_variables(state_vars));
        }

        // 2. 生成状态枚举
        if let Some(states) = &behavior_annex.states {
            stmts.extend(self.generate_state_enum(states));
        }

        // 3. 设置初始状态
        if let Some(states) = &behavior_annex.states {
            stmts.extend(self.generate_initial_state(states));
        }

        // 4. 生成状态机循环
        if let Some(transitions) = &behavior_annex.transitions {
            stmts.extend(self.generate_state_machine_loop(impl_, transitions));
        }

        Some(stmts)
    }

    /// 生成状态变量声明
    fn generate_state_variables(&self, state_vars: &[StateVariable]) -> Vec<Statement> {
        let mut stmts = Vec::new();

        for var in state_vars {
            let var_name = var.identifier.clone();
            let rust_type = self.convert_aadl_type_to_rust(&var.data_type);

            // 生成变量声明
            let init_value = if let Some(init) = &var.initial_value {
                // 如果有初始值，使用它
                self.parse_initial_value(init, &rust_type)
            } else {
                // 否则使用默认值
                self.generate_default_value_for_type(&rust_type)
            };

            stmts.push(Statement::Let(LetStmt {
                ifmut: true,
                name: var_name,
                ty: Some(rust_type),
                init: Some(init_value),
            }));
        }

        stmts
    }

    /// 生成状态枚举
    fn generate_state_enum(&self, states: &[State]) -> Vec<Statement> {
        let mut variants = Vec::new();
        
        for state in states {
            for state_id in &state.identifiers {
                variants.push(Variant {
                    name: state_id.clone(),
                    data: None, // 状态枚举通常没有数据
                    docs: vec![format!("// State: {}", state_id)],
                });
            }
        }

        // 生成枚举定义
        let enum_def = EnumDef {
            name: "State".to_string(), // 使用通用名称，后续会重命名
            variants,
            generics: Vec::new(),
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            docs: vec!["// Behavior Annex state machine states".to_string()],
            vis: Visibility::Private, // 在函数内部定义
        };

        vec![Statement::Item(Box::new(Item::Enum(enum_def)))]
    }

    /// 生成初始状态设置
    fn generate_initial_state(&self, states: &[State]) -> Vec<Statement> {
        let mut stmts = Vec::new();

        // 查找初始状态
        let initial_state = states.iter()
            .find(|state| state.modifiers.contains(&StateModifier::Initial))
            .and_then(|state| state.identifiers.first());

        if let Some(init_state) = initial_state {
            stmts.push(Statement::Let(LetStmt {
                ifmut: true,
                name: "state".to_string(),
                ty: Some(Type::Named("State".to_string())),
                init: Some(Expr::Path(
                    vec!["State".to_string(), init_state.clone()],
                    PathType::Namespace,
                )),
            }));
        } else {
            // 如果没有找到初始状态，使用第一个状态,这种情况不会出现
            if let Some(first_state) = states.first().and_then(|s| s.identifiers.first()) {
                stmts.push(Statement::Let(LetStmt {
                    ifmut: true,
                    name: "state".to_string(),
                    ty: Some(Type::Named("State".to_string())),
                    init: Some(Expr::Path(
                        vec!["State".to_string(), first_state.clone()],
                        PathType::Member,
                    )),
                }));
            }
        }

        stmts
    }

    /// 生成状态机循环
    fn generate_state_machine_loop(&self, impl_: &ComponentImplementation, transitions: &[Transition]) -> Vec<Statement> {
        let mut stmts = Vec::new();

        // 生成端口数据接收代码
        let port_receive_stmts = self.generate_port_receive_code(impl_);

        // 生成状态转换逻辑
        let state_transition_stmts = self.generate_state_transition_logic(transitions);

        // 构建完整的循环
        let loop_body = vec![
            // 记录循环开始时间
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
            // 尝试接收端口数据
            Statement::Expr(Expr::Block(Block {
                stmts: port_receive_stmts,
                expr: None,
            })),
            // 状态机宏步执行
            Statement::Expr(Expr::Block(Block {
                stmts: state_transition_stmts,
                expr: None,
            })),
            // 计算执行时间并睡眠
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
        ];

        stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
            stmts: loop_body,
            expr: None,
        }))));

        stmts
    }

    /// 生成端口数据接收代码
    fn generate_port_receive_code(&self, impl_: &ComponentImplementation) -> Vec<Statement> {
        let mut stmts = Vec::new();

        // 获取组件类型以了解端口
        if let Some(comp_type) = self.get_component_type(impl_) {
            if let FeatureClause::Items(features) = &comp_type.features {
                for feature in features {
                    if let Feature::Port(port) = feature {
                        // 只处理输入端口
                        if port.direction == PortDirection::In {
                            let port_name = port.identifier.to_lowercase();
                            
                            // 生成端口数据接收代码
                            let receive_stmt = Statement::Let(LetStmt {
                                ifmut: false,
                                name: format!("{}_val", port_name),
                                ty: None,
                                init: Some(Expr::Match {
                                    expr: Box::new(Expr::MethodCall(
                                        Box::new(Expr::Reference(
                                            Box::new(Expr::Path(
                                                vec!["self".to_string(), port_name.clone()],
                                                PathType::Member,
                                            )),
                                            true,
                                            false,
                                        )),
                                        "try_recv".to_string(),
                                        Vec::new(),
                                    )),
                                    arms: vec![
                                        MatchArm {
                                            pattern: "Ok(val)".to_string(),
                                            guard: None,
                                            body: Block {
                                                stmts: vec![],
                                                expr: Some(Box::new(Expr::Ident("val".to_string()))),
                                            },
                                        },
                                        MatchArm {
                                            pattern: "Err(_)".to_string(),
                                            guard: None,
                                            body: Block {
                                                stmts: vec![],
                                                expr: Some(Box::new(Expr::Ident("None".to_string()))),
                                            },
                                        },
                                    ],
                                }),
                            });

                            stmts.push(receive_stmt);
                        }
                    }
                }
            }
        }

        stmts
    }

    /// 生成状态转换逻辑
    fn generate_state_transition_logic(&self, transitions: &[Transition]) -> Vec<Statement> {
        let mut stmts = Vec::new();

        // 添加注释
        stmts.push(Statement::Expr(Expr::Ident("// --- BA 宏步执行 ---".to_string())));

        // 生成状态转换循环
        let mut match_arms = Vec::new();

        // 为每个状态生成匹配分支
        for transition in transitions {
            for source_state in &transition.source_states {
                let arm = self.generate_state_match_arm(transition, source_state);
                match_arms.push(arm);
            }
        }

        // 构建match表达式
        let match_expr = Expr::Match {
            expr: Box::new(Expr::Ident("state".to_string())),
            arms: match_arms,
        };

        // 包装在循环中
        stmts.push(Statement::Expr(Expr::Loop(Box::new(Block {
            stmts: vec![
                Statement::Expr(match_expr),
                Statement::Expr(Expr::Ident("break; // 到 complete state 停止 BA 宏步".to_string())),
            ],
            expr: None,
        }))));

        stmts
    }

    /// 生成状态匹配分支
    fn generate_state_match_arm(&self, transition: &Transition, source_state: &str) -> MatchArm {
        let mut stmts = Vec::new();

        // 处理转换条件
        if let Some(condition) = &transition.behavior_condition {
            match condition {
                BehaviorCondition::Dispatch(dispatch_cond) => {
                    // 处理 "on dispatch" 条件
                    stmts.push(Statement::Expr(Expr::Ident(format!("// on dispatch → {}", transition.destination_state))));
                    stmts.push(Statement::Let(LetStmt {
                        ifmut: true,
                        name: "state".to_string(),
                        ty: None,
                        init: Some(Expr::Path(
                            vec!["State".to_string(), transition.destination_state.clone()],
                            PathType::Member,
                        )),
                    }));
                    
                    // 检查目标状态是否需要continue
                    if self.should_continue_state(&transition.destination_state) {
                        stmts.push(Statement::Expr(Expr::Ident("continue; // 不是 complete，要继续".to_string())));
                    } else {
                        stmts.push(Statement::Expr(Expr::Ident("// complete，停".to_string())));
                    }
                }
                BehaviorCondition::Execute(execute_cond) => {
                    // 处理执行条件
                    stmts.extend(self.generate_execute_condition_code(execute_cond, transition));
                }
            }
        } else {
            // 无条件转换
            stmts.push(Statement::Let(LetStmt {
                ifmut: true,
                name: "state".to_string(),
                ty: None,
                init: Some(Expr::Path(
                    vec!["State".to_string(), transition.destination_state.clone()],
                    PathType::Member,
                )),
            }));
            
            if self.should_continue_state(&transition.destination_state) {
                stmts.push(Statement::Expr(Expr::Ident("continue; // 不是 complete，要继续".to_string())));
            } else {
                stmts.push(Statement::Expr(Expr::Ident("// complete，停".to_string())));
            }
        }

        // 处理动作
        if let Some(actions) = &transition.actions {
            stmts.extend(self.generate_action_code(actions));
        }

        MatchArm {
            pattern: format!("State::{}", source_state),
            guard: None,
            body: Block {
                stmts,
                expr: None,
            },
        }
    }

    /// 生成执行条件代码
    fn generate_execute_condition_code(&self, execute_cond: &DispatchConjunction, transition: &Transition) -> Vec<Statement> {
        let mut stmts = Vec::new();

        // 处理端口检查条件
        for trigger in &execute_cond.dispatch_triggers {
            match trigger {
                DispatchTrigger::InEventPort(port_name) => {
                    // 生成端口值检查
                    let port_var = format!("{}_val", port_name.to_lowercase());
                    stmts.push(Statement::Expr(Expr::If {
                        condition: Box::new(Expr::BinaryOp(
                            Box::new(Expr::Ident(port_var.clone())),
                            "==".to_string(),
                            Box::new(Expr::Literal(Literal::Bool(true))),
                        )),
                        then_branch: Block {
                            stmts: vec![
                                Statement::Let(LetStmt {
                                    ifmut: true,
                                    name: "state".to_string(),
                                    ty: None,
                                    init: Some(Expr::Path(
                                        vec!["State".to_string(), transition.destination_state.clone()],
                                        PathType::Member,
                                    )),
                                }),
                            ],
                            expr: None,
                        },
                        else_branch: None,
                    }));
                }
                DispatchTrigger::InEventDataPort(port_name) => {
                    // 处理事件数据端口
                    let port_var = format!("{}_val", port_name.to_lowercase());
                    stmts.push(Statement::Expr(Expr::IfLet {
                        pattern: "Some(val)".to_string(),
                        value: Box::new(Expr::Ident(port_var.clone())),
                        then_branch: Block {
                            stmts: vec![
                                Statement::Let(LetStmt {
                                    ifmut: true,
                                    name: "state".to_string(),
                                    ty: None,
                                    init: Some(Expr::Path(
                                        vec!["State".to_string(), transition.destination_state.clone()],
                                        PathType::Member,
                                    )),
                                }),
                            ],
                            expr: None,
                        },
                        else_branch: None,
                    }));
                }
            }
        }

        stmts
    }

    /// 生成动作代码
    fn generate_action_code(&self, actions: &BehaviorActionBlock) -> Vec<Statement> {
        let mut stmts = Vec::new();

        match &actions.actions {
            BehaviorActions::Sequence(seq) => {
                for action in &seq.actions {
                    stmts.extend(self.generate_single_action(action));
                }
            }
            BehaviorActions::Set(set) => {
                for action in &set.actions {
                    stmts.extend(self.generate_single_action(action));
                }
            }
            BehaviorActions::Single(action) => {
                stmts.extend(self.generate_single_action(action));
            }
        }

        stmts
    }

    /// 生成单个动作代码
    fn generate_single_action(&self, action: &BehaviorAction) -> Vec<Statement> {
        let mut stmts = Vec::new();

        match action {
            BehaviorAction::Basic(basic_action) => {
                stmts.extend(self.generate_basic_action(basic_action));
            }
            BehaviorAction::Block(block) => {
                stmts.extend(self.generate_action_code(block));
            }
            BehaviorAction::If(if_stmt) => {
                stmts.extend(self.generate_if_statement(if_stmt));
            }
            _ => {
                // 其他类型的动作暂时跳过
                stmts.push(Statement::Expr(Expr::Ident("// TODO: Unsupported action type".to_string())));
            }
        }

        stmts
    }

    /// 生成基本动作代码
    fn generate_basic_action(&self, action: &BasicAction) -> Vec<Statement> {
        let mut stmts = Vec::new();

        match action {
            BasicAction::Assignment(assignment) => {
                stmts.extend(self.generate_assignment_action(assignment));
            }
            BasicAction::Communication(comm) => {
                stmts.extend(self.generate_communication_action(comm));
            }
            BasicAction::Timed(timed) => {
                stmts.extend(self.generate_timed_action(timed));
            }
        }

        stmts
    }

    /// 生成赋值动作
    fn generate_assignment_action(&self, assignment: &AssignmentAction) -> Vec<Statement> {
        let mut stmts = Vec::new();

        let target_name = match &assignment.target {
            Target::LocalVariable(name) => name.clone(),
            _ => "unknown".to_string(),
        };

        let value_expr = match &assignment.value {
            AssignmentValue::Expression(expr) => self.convert_value_expression(expr),
            AssignmentValue::Any => Expr::Literal(Literal::Int(0)), // 默认值
        };

        stmts.push(Statement::Let(LetStmt {
            ifmut: true,
            name: target_name,
            ty: None,
            init: Some(value_expr),
        }));

        stmts
    }

    /// 生成通信动作
    fn generate_communication_action(&self, comm: &CommunicationAction) -> Vec<Statement> {
        let mut stmts = Vec::new();

        match comm {
            CommunicationAction::PortCommunication(port_comm) => {
                match port_comm {
                    PortCommunication::Output { port, value } => {
                        let port_name = port.to_lowercase();
                        let value_expr = if let Some(val) = value {
                            self.convert_value_expression(val)
                        } else {
                            Expr::Literal(Literal::Bool(true)) // 默认发送true
                        };

                        stmts.push(Statement::Expr(Expr::IfLet {
                            pattern: "Some(sender)".to_string(),
                            value: Box::new(Expr::Reference(
                                Box::new(Expr::Path(
                                    vec!["self".to_string(), port_name],
                                    PathType::Member,
                                )),
                                true,
                                false,
                            )),
                            then_branch: Block {
                                stmts: vec![
                                    Statement::Let(LetStmt {
                                        ifmut: false,
                                        name: "_".to_string(),
                                        ty: None,
                                        init: Some(Expr::MethodCall(
                                            Box::new(Expr::Ident("sender".to_string())),
                                            "send".to_string(),
                                            vec![value_expr],
                                        )),
                                    }),
                                ],
                                expr: None,
                            },
                            else_branch: None,
                        }));
                    }
                    _ => {
                        stmts.push(Statement::Expr(Expr::Ident("// TODO: Unsupported port communication".to_string())));
                    }
                }
            }
            _ => {
                stmts.push(Statement::Expr(Expr::Ident("// TODO: Unsupported communication action".to_string())));
            }
        }

        stmts
    }

    /// 生成定时动作
    fn generate_timed_action(&self, _timed: &TimedAction) -> Vec<Statement> {
        // 定时动作暂时跳过
        vec![Statement::Expr(Expr::Ident("// TODO: Timed action not implemented".to_string()))]
    }

    /// 生成if语句
    fn generate_if_statement(&self, _if_stmt: &IfStatement) -> Vec<Statement> {
        // if语句暂时跳过
        vec![Statement::Expr(Expr::Ident("// TODO: If statement not implemented".to_string()))]
    }

    /// 判断状态是否需要continue
    fn should_continue_state(&self, state_name: &str) -> bool {
        // 根据状态修饰符判断
        // complete状态不需要continue，其他状态需要
        // 这里简化处理，实际应该根据状态定义中的修饰符判断
        !state_name.contains("Complete")
    }

    /// 转换AADL类型到Rust类型
    fn convert_aadl_type_to_rust(&self, aadl_type: &str) -> Type {
        match aadl_type.to_lowercase().as_str() {
            "integer" | "i32" | "base_types::integer" => Type::Named("i32".to_string()),
            "boolean" | "bool" => Type::Named("bool".to_string()),
            "string" => Type::Named("String".to_string()),
            "real" | "f64" => Type::Named("f64".to_string()),
            _ => Type::Named(aadl_type.to_string()),
        }
    }

    /// 解析初始值
    fn parse_initial_value(&self, init_value: &str, rust_type: &Type) -> Expr {
        match rust_type {
            Type::Named(type_name) => {
                match type_name.as_str() {
                    "i32" => {
                        if let Ok(val) = init_value.parse::<i64>() {
                            Expr::Literal(Literal::Int(val))
                        } else {
                            Expr::Literal(Literal::Int(0))
                        }
                    }
                    "bool" => {
                        match init_value.to_lowercase().as_str() {
                            "true" => Expr::Literal(Literal::Bool(true)),
                            "false" => Expr::Literal(Literal::Bool(false)),
                            _ => Expr::Literal(Literal::Bool(false)),
                        }
                    }
                    "String" => Expr::Literal(Literal::Str(init_value.to_string())),
                    "f64" => {
                        if let Ok(val) = init_value.parse::<f64>() {
                            Expr::Literal(Literal::Float(val))
                        } else {
                            Expr::Literal(Literal::Float(0.0))
                        }
                    }
                    _ => Expr::Literal(Literal::Int(0)),
                }
            }
            _ => Expr::Literal(Literal::Int(0)),
        }
    }

    /// 为类型生成默认值
    fn generate_default_value_for_type(&self, rust_type: &Type) -> Expr {
        match rust_type {
            Type::Named(type_name) => {
                match type_name.as_str() {
                    "i32" => Expr::Literal(Literal::Int(0)),
                    "bool" => Expr::Literal(Literal::Bool(false)),
                    "String" => Expr::Literal(Literal::Str("".to_string())),
                    "f64" => Expr::Literal(Literal::Float(0.0)),
                    _ => Expr::Literal(Literal::Int(0)),
                }
            }
            _ => Expr::Literal(Literal::Int(0)),
        }
    }

    /// 转换值表达式
    fn convert_value_expression(&self, expr: &ValueExpression) -> Expr {
        // 简化处理，只处理基本的关系表达式
        let left = self.convert_relation(&expr.left);
        
        if expr.operations.is_empty() {
            left
        } else {
            // 处理逻辑操作
            let mut result = left;
            for op in &expr.operations {
                let right = self.convert_relation(&op.right);
                result = Expr::BinaryOp(
                    Box::new(result),
                    match op.operator {
                        LogicalOperator::And => "&&".to_string(),
                        LogicalOperator::Or => "||".to_string(),
                        LogicalOperator::Xor => "^".to_string(),
                    },
                    Box::new(right),
                );
            }
            result
        }
    }

    /// 转换关系表达式
    fn convert_relation(&self, _relation: &Relation) -> Expr {
        // 简化处理，返回默认值
        Expr::Literal(Literal::Bool(true))
    }

    /// 获取组件类型
    fn get_component_type(&self, impl_: &ComponentImplementation) -> Option<&ComponentType> {
        // 这里需要从外部传入组件类型信息
        // 暂时返回None，实际使用时需要修改
        None
    }
} 