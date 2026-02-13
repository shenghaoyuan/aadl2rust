#![allow(unused_mut, clippy::manual_strip)]
use super::ast::aadl_ast_cj::*;
use crate::aadlight_parser;
use pest::iterators::Pair;

// Helper function: extract identifier text from a Pair
pub fn extract_identifier(pair: Pair<aadlight_parser::Rule>) -> String {
    pair.as_str().trim().to_string()
}

/// Transform an annex subclause
/// Handles the annex_subclause rule
pub fn transform_annex_subclause(pair: Pair<aadlight_parser::Rule>) -> Option<AnnexSubclause> {
    let mut inner_iter = pair.into_inner();

    // The first element should be annex_identifier
    let identifier_pair = inner_iter.next().unwrap();
    let identifier = transform_annex_identifier(identifier_pair);

    // The second element should be annex_content (enclosed in {** **})
    let content_pair = inner_iter.next().unwrap();
    let content = transform_annex_content(content_pair);

    Some(AnnexSubclause {
        identifier,
        content,
    })
}

/// Transform an annex identifier
/// Handles the annex_identifier rule
pub fn transform_annex_identifier(pair: Pair<aadlight_parser::Rule>) -> AnnexIdentifier {
    match pair.as_str().trim() {
        "Behavior_specification" => AnnexIdentifier::BehaviorSpecification,
        "EMV2" => AnnexIdentifier::EMV2,
        _ => {
            // For other identifiers, default to BehaviorSpecification for now
            // Can be extended as needed
            AnnexIdentifier::BehaviorSpecification
        }
    }
}

/// Transform annex content
/// Handles the annex_content rule
pub fn transform_annex_content(pair: Pair<aadlight_parser::Rule>) -> AnnexContent {
    // Check whether there is any content
    let inner_pairs: Vec<_> = pair.into_inner().collect();

    if inner_pairs.is_empty() {
        return AnnexContent::None;
    }

    // Look for behavior_annex_content
    for inner in inner_pairs {
        if inner.as_rule() == aadlight_parser::Rule::behavior_annex_content {
            if let Some(behavior_content) = transform_behavior_annex_content(inner) {
                return AnnexContent::BehaviorAnnex(behavior_content);
            }
        }
    }

    // If no valid content is found, return None
    AnnexContent::None
}

/// Transform Behavior Annex content
/// Handles the behavior_annex_content rule
pub fn transform_behavior_annex_content(
    pair: Pair<aadlight_parser::Rule>,
) -> Option<BehaviorAnnexContent> {
    let mut state_variables = None;
    let mut states = None;
    let mut transitions = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            aadlight_parser::Rule::state_variables => {
                state_variables = Some(transform_state_variables(inner));
            }
            aadlight_parser::Rule::states => {
                states = Some(transform_states(inner));
            }
            aadlight_parser::Rule::transitions => {
                transitions = Some(transform_transitions(inner));
            }
            _ => {}
        }
    }

    Some(BehaviorAnnexContent {
        state_variables,
        states,
        transitions,
    })
}

/// Transform state variable declarations
/// Handles the state_variables rule
pub fn transform_state_variables(pair: Pair<aadlight_parser::Rule>) -> Vec<StateVariable> {
    let mut variables = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == aadlight_parser::Rule::state_variable_declaration {
            variables.push(transform_state_variable_declaration(inner));
        }
    }

    variables
}

/// Transform a single state variable declaration
/// Handles the state_variable_declaration rule
pub fn transform_state_variable_declaration(pair: Pair<aadlight_parser::Rule>) -> StateVariable {
    let mut inner_iter = pair.into_inner();

    let identifier = extract_identifier(inner_iter.next().unwrap());
    // let _colon = inner_iter.next(); // Skip ":"
    let data_type = extract_identifier(inner_iter.next().unwrap());

    let mut initial_value = None;
    if let Some(assignment_pair) = inner_iter.next() {
        if assignment_pair.as_rule() == aadlight_parser::Rule::assignment_operator {
            // Skip ":="
            if let Some(value_pair) = inner_iter.next() {
                if value_pair.as_rule() == aadlight_parser::Rule::behavior_expression {
                    initial_value = Some(extract_identifier(value_pair));
                }
            }
        }
    }

    StateVariable {
        identifier,
        data_type,
        initial_value,
    }
}

/// Transform state definitions
/// Handles the states rule
pub fn transform_states(pair: Pair<aadlight_parser::Rule>) -> Vec<State> {
    let mut state_list = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == aadlight_parser::Rule::state_declaration {
            state_list.push(transform_state_declaration(inner));
        }
    }

    state_list
}

/// Transform a single state declaration
/// Handles the state_declaration rule
pub fn transform_state_declaration(pair: Pair<aadlight_parser::Rule>) -> State {
    let mut inner_iter = pair.into_inner();

    let mut identifiers = Vec::new();
    let mut modifiers = Vec::new();

    // Process identifier list
    for inner in inner_iter {
        match inner.as_rule() {
            aadlight_parser::Rule::identifier => {
                identifiers.push(extract_identifier(inner));
            }
            aadlight_parser::Rule::state_modifier => {
                modifiers.push(transform_state_modifier(inner));
            }
            _ => {}
        }
    }

    State {
        identifiers,
        modifiers,
    }
}

/// Transform a state modifier
/// Handles the state_modifier rule
pub fn transform_state_modifier(pair: Pair<aadlight_parser::Rule>) -> StateModifier {
    match pair.as_str().trim() {
        "initial" => StateModifier::Initial,
        "complete" => StateModifier::Complete,
        "final" => StateModifier::Final,
        // Other modifiers can be added as needed
        _ => StateModifier::Complete, // Default value
    }
}

/// Transform transition definitions
/// Handles the transitions rule
pub fn transform_transitions(pair: Pair<aadlight_parser::Rule>) -> Vec<Transition> {
    let mut transition_list = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == aadlight_parser::Rule::transition_declaration {
            transition_list.push(transform_transition_declaration(inner));
        }
    }

    transition_list
}

/// Transform a single transition declaration
/// Handles the transition_declaration rule
/// Syntax: identifier ~ "-[" ~ guard? ~ "]->" ~ identifier ~ behavior_action_block? ~ ";"
pub fn transform_transition_declaration(pair: Pair<aadlight_parser::Rule>) -> Transition {
    let mut inner_iter = pair.into_inner();

    // The first identifier is the source state
    let source_state = extract_identifier(inner_iter.next().unwrap());
    let source_states = vec![source_state];

    // Skip "-["
    // let _dash_bracket = inner_iter.next();

    let mut behavior_condition = None;
    let mut destination_state = String::new();
    let mut actions = None;

    // Handle optional guard condition
    if let Some(guard_pair) = inner_iter.next() {
        if guard_pair.as_rule() == aadlight_parser::Rule::guard {
            behavior_condition = Some(transform_guard(guard_pair));
        } else {
            // If it is not a guard, then this is the destination state
            destination_state = extract_identifier(guard_pair);
        }
    }

    // Skip "]->"
    // let _arrow_bracket = inner_iter.next();

    // If destination state is not yet set, the next identifier is the destination
    if destination_state.is_empty() {
        if let Some(dest_pair) = inner_iter.next() {
            if dest_pair.as_rule() == aadlight_parser::Rule::identifier {
                destination_state = extract_identifier(dest_pair);
            }
        }
    }

    // Handle optional action block
    if let Some(action_pair) = inner_iter.next() {
        if action_pair.as_rule() == aadlight_parser::Rule::behavior_action_block {
            actions = Some(transform_behavior_action_block(action_pair));
        }
    }

    Transition {
        transition_identifier: None, // No separate identifier for transitions yet
        priority: None,
        source_states,
        destination_state,
        behavior_condition,
        actions,
    }
}

/// Transform a guard condition
/// Handles the guard rule
pub fn transform_guard(pair: Pair<aadlight_parser::Rule>) -> BehaviorCondition {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        aadlight_parser::Rule::dispatch_condition => {
            // Handle "on dispatch"
            transform_dispatch_condition(inner)
        }
        aadlight_parser::Rule::execute_condition => {
            // Handle execute_condition: unary_boolean_operator? ~ identifier
            transform_execute_condition(inner)
        }
        _ => {
            // Default handling
            BehaviorCondition::Execute(DispatchConjunction {
                not: false,
                dispatch_triggers: vec![],
                number: None,
                less_than: false,
            })
        }
    }
}

/// Transform a dispatch condition
/// Handles the dispatch_condition rule
fn transform_dispatch_condition(pair: Pair<aadlight_parser::Rule>) -> BehaviorCondition {
    let mut inner_iter = pair.into_inner();

    // Skip "on dispatch"
    // Handle optional dispatch_trigger_condition
    let mut trigger_condition = None;
    let mut frozen_ports = None;

    for inner in inner_iter {
        match inner.as_rule() {
            aadlight_parser::Rule::dispatch_trigger_condition => {
                trigger_condition = Some(transform_dispatch_trigger_condition(inner));
            }
            aadlight_parser::Rule::frozen_ports => {
                frozen_ports = Some(transform_frozen_ports(inner));
            }
            _ => {}
        }
    }

    BehaviorCondition::Dispatch(DispatchCondition {
        trigger_condition,
        frozen_ports,
    })
}

/// Transform an execute condition
/// Handles the execute_condition rule
fn transform_execute_condition(pair: Pair<aadlight_parser::Rule>) -> BehaviorCondition {
    let inner_iter = pair.into_inner();
    let mut has_not = false;
    let mut identifier = String::new();
    let mut number = String::new();
    let mut less_than = false;
    // Traverse all inner elements
    for inner in inner_iter {
        match inner.as_rule() {
            aadlight_parser::Rule::unary_boolean_operator => {
                has_not = true;
            }
            aadlight_parser::Rule::identifier => {
                identifier = inner.as_str().to_string();
            }
            aadlight_parser::Rule::number => {
                number = inner.as_str().trim().to_string();
            }
            aadlight_parser::Rule::less_than_operator => {
                less_than = true;
            }
            _ => {
                // Ignore other rules
            }
        }
    }

    // Construct DispatchConjunction
    let conjunction = DispatchConjunction {
        not: has_not,
        dispatch_triggers: vec![DispatchTrigger::InEventPort(identifier)],
        number: Some(number),
        less_than,
    };

    BehaviorCondition::Execute(conjunction)
}

/// Transform a dispatch trigger condition
/// Handles the dispatch_trigger_condition rule
fn transform_dispatch_trigger_condition(
    pair: Pair<aadlight_parser::Rule>,
) -> DispatchTriggerCondition {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        aadlight_parser::Rule::dispatch_trigger_logical_expression => {
            DispatchTriggerCondition::LogicalExpression(
                transform_dispatch_trigger_logical_expression(inner),
            )
        }
        aadlight_parser::Rule::provides_subprogram_access_identifier => {
            DispatchTriggerCondition::SubprogramAccess(inner.as_str().to_string())
        }
        _ if inner.as_str() == "stop" => DispatchTriggerCondition::Stop,
        _ => {
            // Default handling
            DispatchTriggerCondition::Stop
        }
    }
}

/// Transform a dispatch trigger logical expression
/// Handles the dispatch_trigger_logical_expression rule
fn transform_dispatch_trigger_logical_expression(
    pair: Pair<aadlight_parser::Rule>,
) -> DispatchTriggerLogicalExpression {
    let mut conjunctions = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == aadlight_parser::Rule::dispatch_conjunction {
            conjunctions.push(transform_dispatch_conjunction(inner));
        }
    }

    DispatchTriggerLogicalExpression {
        dispatch_conjunctions: conjunctions,
    }
}

/// Transform a dispatch conjunction
/// Handles the dispatch_conjunction rule
fn transform_dispatch_conjunction(pair: Pair<aadlight_parser::Rule>) -> DispatchConjunction {
    let mut triggers = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == aadlight_parser::Rule::dispatch_trigger {
            triggers.push(transform_dispatch_trigger(inner));
        }
    }

    DispatchConjunction {
        not: false, // Simplified handling; 'not' is not supported yet
        dispatch_triggers: triggers,
        number: None,
        less_than: false,
    }
}

/// Transform a dispatch trigger
/// Handles the dispatch_trigger rule
fn transform_dispatch_trigger(pair: Pair<aadlight_parser::Rule>) -> DispatchTrigger {
    let identifier = pair.as_str().to_string();

    // Simplified handling: assume all are event ports
    DispatchTrigger::InEventPort(identifier)
}

/// Transform frozen ports
/// Handles the frozen_ports rule
fn transform_frozen_ports(pair: Pair<aadlight_parser::Rule>) -> Vec<String> {
    let mut ports = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == aadlight_parser::Rule::identifier {
            ports.push(inner.as_str().to_string());
        }
    }

    ports
}

/// Transform a behavior action block
/// Handles the behavior_action_block rule
/// Syntax: "{" ~ behavior_actions ~ "}" ~ ("timeout" ~ behavior_time)?
pub fn transform_behavior_action_block(pair: Pair<aadlight_parser::Rule>) -> BehaviorActionBlock {
    let mut inner_iter = pair.into_inner();

    // Skip opening "{"
    // let _open_brace = inner_iter.next();

    // Handle behavior_actions
    let behavior_actions_pair = inner_iter.next().unwrap();
    let actions = transform_behavior_actions(behavior_actions_pair);

    // Skip closing "}"
    // let _close_brace = inner_iter.next();

    // Handle optional timeout
    let mut timeout = None;
    if let Some(timeout_pair) = inner_iter.next() {
        if timeout_pair.as_rule() == aadlight_parser::Rule::behavior_time {
            timeout = Some(transform_behavior_time(timeout_pair));
        }
    }

    BehaviorActionBlock { actions, timeout }
}

/// Transform behavior actions
/// Handles the behavior_actions rule
/// Syntax: behavior_action_sequence | behavior_action_set | behavior_action
pub fn transform_behavior_actions(pair: Pair<aadlight_parser::Rule>) -> BehaviorActions {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        aadlight_parser::Rule::behavior_action_sequence => {
            BehaviorActions::Sequence(transform_behavior_action_sequence(inner))
        }
        aadlight_parser::Rule::behavior_action_set => {
            BehaviorActions::Set(transform_behavior_action_set(inner))
        }
        aadlight_parser::Rule::behavior_action => {
            // Wrap a single action into a sequence
            let action = transform_behavior_action(inner);
            BehaviorActions::Sequence(BehaviorActionSequence {
                actions: vec![action],
            })
        }
        _ => {
            // Default handling
            BehaviorActions::Sequence(BehaviorActionSequence {
                actions: vec![BehaviorAction::Basic(BasicAction::Assignment(
                    AssignmentAction {
                        target: Target::LocalVariable("default".to_string()),
                        value: AssignmentValue::Any,
                    },
                ))],
            })
        }
    }
}

/// Transform a behavior action sequence
/// Handles the behavior_action_sequence rule
/// Syntax: behavior_action ~ (";" ~ behavior_action)+
pub fn transform_behavior_action_sequence(
    pair: Pair<aadlight_parser::Rule>,
) -> BehaviorActionSequence {
    let mut actions = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == aadlight_parser::Rule::behavior_action {
            actions.push(transform_behavior_action(inner));
        }
    }

    BehaviorActionSequence { actions }
}

/// Transform a behavior action set
/// Handles the behavior_action_set rule
/// Syntax: behavior_action ~ ("&" ~ behavior_action)+
pub fn transform_behavior_action_set(pair: Pair<aadlight_parser::Rule>) -> BehaviorActionSet {
    let mut actions = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == aadlight_parser::Rule::behavior_action {
            actions.push(transform_behavior_action(inner));
        }
    }

    BehaviorActionSet { actions }
}

/// Transform a single behavior action
/// Handles the behavior_action rule
pub fn transform_behavior_action(pair: Pair<aadlight_parser::Rule>) -> BehaviorAction {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        aadlight_parser::Rule::basic_action => BehaviorAction::Basic(transform_basic_action(inner)),
        aadlight_parser::Rule::behavior_action_block => {
            BehaviorAction::Block(Box::new(transform_behavior_action_block(inner)))
        }
        aadlight_parser::Rule::if_statement => BehaviorAction::If(transform_if_statement(inner)),
        aadlight_parser::Rule::for_statement => BehaviorAction::For(transform_for_statement(inner)),
        aadlight_parser::Rule::forall_statement => {
            BehaviorAction::Forall(transform_forall_statement(inner))
        }
        aadlight_parser::Rule::while_statement => {
            BehaviorAction::While(transform_while_statement(inner))
        }
        aadlight_parser::Rule::do_until_statement => {
            BehaviorAction::DoUntil(transform_do_until_statement(inner))
        }
        _ => {
            // Default handling
            BehaviorAction::Basic(BasicAction::Assignment(AssignmentAction {
                target: Target::LocalVariable("default".to_string()),
                value: AssignmentValue::Any,
            }))
        }
    }
}

/// Transform a basic action
/// Handles the basic_action rule
pub fn transform_basic_action(pair: Pair<aadlight_parser::Rule>) -> BasicAction {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        aadlight_parser::Rule::assignment_action => transform_assignment_action(inner),
        aadlight_parser::Rule::communication_action => transform_communication_action(inner),
        aadlight_parser::Rule::computation_action => transform_computation_action(inner),
        _ => {
            // Default assignment action
            BasicAction::Assignment(AssignmentAction {
                target: Target::LocalVariable("default".to_string()),
                value: AssignmentValue::Any,
            })
        }
    }
}

/// Transform an assignment action
/// Handles the assignment_action rule
pub fn transform_assignment_action(pair: Pair<aadlight_parser::Rule>) -> BasicAction {
    let mut inner_iter = pair.into_inner();

    let target_str = extract_identifier(inner_iter.next().unwrap());
    // let _assign = inner_iter.next(); // Skip ":="

    // Handle assignment expression or "any"
    let value = if let Some(value_pair) = inner_iter.next() {
        match value_pair.as_rule() {
            aadlight_parser::Rule::behavior_expression => {
                // Handle behavior expression
                AssignmentValue::Expression(transform_behavior_expression(value_pair))
            }
            _ => {
                // Check whether it is "any"
                if value_pair.as_str() == "any" {
                    AssignmentValue::Any
                } else {
                    // Otherwise, try to handle it as an expression
                    AssignmentValue::Expression(transform_behavior_expression(value_pair))
                }
            }
        }
    } else {
        AssignmentValue::Any
    };

    // Determine target type based on target_str
    let target = {
        use crate::transform::get_global_port_manager;
        if let Ok(manager) = get_global_port_manager().lock() {
            if manager.is_outgoing_port(&target_str) {
                Target::OutgoingPort(target_str)
            } else {
                Target::LocalVariable(target_str)
            }
        } else {
            // If the port manager cannot be obtained, default to a local variable
            Target::LocalVariable(target_str)
        }
    };

    BasicAction::Assignment(AssignmentAction { target, value })
}

/// Transform a communication action
/// Handles the communication_action rule
pub fn transform_communication_action(pair: Pair<aadlight_parser::Rule>) -> BasicAction {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        aadlight_parser::Rule::port_communication => {
            let mut inner_iter = inner.into_inner();
            BasicAction::Communication(CommunicationAction::PortCommunication(
                PortCommunication::Output {
                    port: extract_identifier(inner_iter.next().unwrap()),
                    value: Some(transform_behavior_expression(inner_iter.next().unwrap())),
                },
            ))
        }
        aadlight_parser::Rule::data_access_communication => {
            BasicAction::Communication(CommunicationAction::DataAccessCommunication(
                DataAccessCommunication::RequiredDataAccess {
                    name: extract_identifier(inner),
                    direction: DataAccessDirection::Input,
                },
            ))
        }
        aadlight_parser::Rule::broadcast_action => {
            BasicAction::Communication(CommunicationAction::Broadcast(Broadcast::Input))
        }
        _ => {
            // Default port communication
            BasicAction::Communication(CommunicationAction::PortCommunication(
                PortCommunication::Output {
                    port: "".to_string(),
                    value: None,
                },
            ))
        }
    }
}

/// Transform a computation action
/// Handles the computation_action rule
pub fn transform_computation_action(pair: Pair<aadlight_parser::Rule>) -> BasicAction {
    let inner = pair.into_inner().next().unwrap();

    BasicAction::Timed(TimedAction {
        start_time: BehaviorTime {
            value: IntegerValue::Constant(extract_identifier(inner)),
            unit: "ms".to_string(),
        },
        end_time: None,
    })
}

/// Transform annexes clause (used for component types and implementations)
/// This function is called by transform.rs
pub fn transform_annexes_clause(pair: Pair<aadlight_parser::Rule>) -> Vec<AnnexSubclause> {
    let mut annexes = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == aadlight_parser::Rule::annex_subclause {
            if let Some(annex) = transform_annex_subclause(inner) {
                annexes.push(annex);
            }
        }
    }

    annexes
}

/// Transform behavior time
/// Handles the behavior_time rule
pub fn transform_behavior_time(_pair: Pair<aadlight_parser::Rule>) -> BehaviorTime {
    // Temporarily return a default value; can be refined later
    BehaviorTime {
        value: IntegerValue::Constant("0".to_string()),
        unit: "ms".to_string(),
    }
}

/// Transform an if statement
/// Handles the if_statement rule
pub fn transform_if_statement(_pair: Pair<aadlight_parser::Rule>) -> IfStatement {
    // Temporarily return a default value; can be refined later
    IfStatement {
        condition: BehaviorExpression {
            disjunctions: vec![],
        },
        then_actions: Box::new(BehaviorActions::Sequence(BehaviorActionSequence {
            actions: vec![],
        })),
        elsif_branches: vec![],
        else_actions: None,
    }
}

/// Transform a for statement
/// Handles the for_statement rule
pub fn transform_for_statement(_pair: Pair<aadlight_parser::Rule>) -> ForStatement {
    // Temporarily return a default value; can be refined later
    ForStatement {
        element_identifier: "".to_string(),
        data_classifier: "".to_string(),
        element_values: ElementValues::IntegerRange(IntegerRange {
            lower: IntegerValue::Constant("0".to_string()),
            upper: IntegerValue::Constant("0".to_string()),
        }),
        actions: Box::new(BehaviorActions::Sequence(BehaviorActionSequence {
            actions: vec![],
        })),
    }
}

/// Transform a forall statement
/// Handles the forall_statement rule
pub fn transform_forall_statement(_pair: Pair<aadlight_parser::Rule>) -> ForallStatement {
    // Temporarily return a default value; can be refined later
    ForallStatement {
        element_identifier: "".to_string(),
        data_classifier: "".to_string(),
        element_values: ElementValues::IntegerRange(IntegerRange {
            lower: IntegerValue::Constant("0".to_string()),
            upper: IntegerValue::Constant("0".to_string()),
        }),
        actions: Box::new(BehaviorActions::Sequence(BehaviorActionSequence {
            actions: vec![],
        })),
    }
}

/// Transform a while statement
/// Handles the while_statement rule
pub fn transform_while_statement(_pair: Pair<aadlight_parser::Rule>) -> WhileStatement {
    // Temporarily return a default value; can be refined later
    WhileStatement {
        condition: BehaviorExpression {
            disjunctions: vec![],
        },
        actions: Box::new(BehaviorActions::Sequence(BehaviorActionSequence {
            actions: vec![],
        })),
    }
}

/// Transform a do-until statement
/// Handles the do_until_statement rule
pub fn transform_do_until_statement(_pair: Pair<aadlight_parser::Rule>) -> DoUntilStatement {
    // Temporarily return a default value; can be refined later
    DoUntilStatement {
        actions: Box::new(BehaviorActions::Sequence(BehaviorActionSequence {
            actions: vec![],
        })),
        condition: BehaviorExpression {
            disjunctions: vec![],
        },
    }
}

/// Transform a behavior expression
/// Handles the behavior_expression rule
pub fn transform_behavior_expression(pair: Pair<aadlight_parser::Rule>) -> ValueExpression {
    // behavior_expression = value_expression
    // value_expression = relation ~ (logical_operator ~ relation)*
    let mut inner_iter = pair.into_inner().next().unwrap().into_inner(); // Need to go one level deeper
    let first_relation = inner_iter.next().unwrap();

    let left = transform_relation(first_relation);
    let mut operations = Vec::new();

    // Handle subsequent logical operations
    while let Some(logical_op) = inner_iter.next() {
        let operator = match logical_op.as_str() {
            "and" => LogicalOperator::And,
            "or" => LogicalOperator::Or,
            "xor" => LogicalOperator::Xor,
            _ => panic!("Unknown logical operator: {}", logical_op.as_str()),
        };

        let right_relation = inner_iter.next().unwrap();
        let right = transform_relation(right_relation);

        operations.push(LogicalOperation { operator, right });
    }

    ValueExpression { left, operations }
}

fn transform_relation(pair: Pair<aadlight_parser::Rule>) -> Relation {
    // relation = simple_expression ~ (relational_operator ~ simple_expression)?
    let mut inner_iter = pair.into_inner();
    let left = transform_simple_expression(inner_iter.next().unwrap());

    let comparison = if let Some(relational_op) = inner_iter.next() {
        let operator = match relational_op.as_str() {
            "=" => RelationalOperator::Equal,
            "!=" => RelationalOperator::NotEqual,
            "<" => RelationalOperator::LessThan,
            "<=" => RelationalOperator::LessThanOrEqual,
            ">" => RelationalOperator::GreaterThan,
            ">=" => RelationalOperator::GreaterThanOrEqual,
            _ => panic!("Unknown relational operator: {}", relational_op.as_str()),
        };

        let right = transform_simple_expression(inner_iter.next().unwrap());
        Some(Comparison { operator, right })
    } else {
        None
    };

    Relation { left, comparison }
}

fn transform_simple_expression(pair: Pair<aadlight_parser::Rule>) -> SimpleExpression {
    // simple_expression = unary_adding_operator? ~ term ~ (binary_adding_operator ~ term)*
    let mut inner_iter = pair.into_inner();

    // Handle unary adding operator
    let sign = if let Some(unary_op) = inner_iter.peek() {
        if unary_op.as_rule() == aadlight_parser::Rule::unary_adding_operator {
            let op = inner_iter.next().unwrap();
            match op.as_str() {
                "+" => Some(UnaryAddingOperator::Plus),
                "-" => Some(UnaryAddingOperator::Minus),
                _ => panic!("Unknown unary adding operator: {}", op.as_str()),
            }
        } else {
            None
        }
    } else {
        None
    };

    let left = transform_term(inner_iter.next().unwrap());
    let mut operations = Vec::new();

    // Handle binary adding operations
    while let Some(binary_op) = inner_iter.next() {
        // Check whether it is a binary adding operator
        if binary_op.as_rule() == aadlight_parser::Rule::binary_adding_operator {
            let operator = match binary_op.as_str() {
                "+" => AdditiveOperator::Add,
                "-" => AdditiveOperator::Subtract,
                _ => panic!("Unknown binary adding operator: {}", binary_op.as_str()),
            };

            let right_term = transform_term(inner_iter.next().unwrap());
            // Convert Factor into BasicExpression
            let right_basic = match &right_term.left {
                Factor::Value(Value::Variable(ValueVariable::LocalVariable(name))) => {
                    BasicExpression::BehaviorVariable(name.clone())
                }
                Factor::Value(Value::Constant(ValueConstant::Numeric(num))) => {
                    BasicExpression::NumericOrConstant(num.clone())
                }
                Factor::Value(Value::Constant(ValueConstant::Boolean(b))) => {
                    BasicExpression::NumericOrConstant(if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    })
                }
                _ => BasicExpression::BehaviorVariable("temp".to_string()), // Default value
            };
            let right = AddExpression {
                left: right_basic,
                operations: Vec::new(), // Temporarily empty; can be refined later
            };
            operations.push(AdditiveOperation { operator, right });
        } else {
            // If it is not a binary adding operator, it belongs to a higher-level expression
            // Stop processing and let the upper-level function handle it
            break;
        }
    }
    SimpleExpression {
        sign,
        left,
        operations,
    }
}

fn transform_term(pair: Pair<aadlight_parser::Rule>) -> Term {
    // term = factor ~ (multiplying_operator ~ factor)*
    let mut inner_iter = pair.into_inner();
    let left = transform_factor(inner_iter.next().unwrap());
    let mut operations = Vec::new();

    // Handle multiplicative operations
    while let Some(mult_op) = inner_iter.next() {
        // Check whether it is a multiplicative operator
        if mult_op.as_rule() == aadlight_parser::Rule::multiplying_operator {
            let operator = match mult_op.as_str() {
                "*" => MultiplicativeOperator::Multiply,
                "/" => MultiplicativeOperator::Divide,
                "mod" => MultiplicativeOperator::Modulo,
                "rem" => MultiplicativeOperator::Remainder,
                _ => panic!("Unknown multiplying operator: {}", mult_op.as_str()),
            };

            let right_factor = transform_factor(inner_iter.next().unwrap());
            // Convert Factor into BasicExpression
            let right_basic = match &right_factor {
                Factor::Value(Value::Variable(ValueVariable::LocalVariable(name))) => {
                    BasicExpression::BehaviorVariable(name.clone())
                }
                Factor::Value(Value::Constant(ValueConstant::Numeric(num))) => {
                    BasicExpression::NumericOrConstant(num.clone())
                }
                Factor::Value(Value::Constant(ValueConstant::Boolean(b))) => {
                    BasicExpression::NumericOrConstant(if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    })
                }
                _ => BasicExpression::BehaviorVariable("temp".to_string()), // Default value
            };
            operations.push(MultiplicativeOperation {
                operator,
                right: right_basic,
            });
        } else {
            // If it is not a multiplicative operator, it belongs to a higher-level expression
            // Stop processing and let the upper-level function handle it
            break;
        }
    }

    Term { left, operations }
}

fn transform_factor(pair: Pair<aadlight_parser::Rule>) -> Factor {
    // factor = value | (value ~ binary_numeric_operator ~ value)
    //        | (unary_numeric_operator ~ value)
    //        | (unary_boolean_operator ~ value)
    let mut inner_iter = pair.into_inner();
    let first = inner_iter.next().unwrap();

    match first.as_rule() {
        aadlight_parser::Rule::value => {
            let value = transform_value(first);
            // Check for a binary numeric operator
            if let Some(binary_op) = inner_iter.next() {
                if binary_op.as_rule() == aadlight_parser::Rule::binary_numeric_operator {
                    let operator = match binary_op.as_str() {
                        "**" => BinaryNumericOperator::Power,
                        _ => panic!("Unknown binary numeric operator: {}", binary_op.as_str()),
                    };

                    let right = transform_value(inner_iter.next().unwrap());
                    Factor::BinaryNumeric {
                        left: value,
                        operator,
                        right,
                    }
                } else {
                    // If it is not a binary operator, fall back to the first value
                    Factor::Value(value)
                }
            } else {
                Factor::Value(value)
            }
        }
        aadlight_parser::Rule::unary_numeric_operator => {
            let operator = match first.as_str() {
                "abs" => UnaryNumericOperator::Abs,
                _ => panic!("Unknown unary numeric operator: {}", first.as_str()),
            };

            let value = transform_value(inner_iter.next().unwrap());
            Factor::UnaryNumeric { operator, value }
        }
        aadlight_parser::Rule::unary_boolean_operator => {
            let operator = match first.as_str() {
                "not" => UnaryBooleanOperator::Not,
                _ => panic!("Unknown unary boolean operator: {}", first.as_str()),
            };

            let value = transform_value(inner_iter.next().unwrap());
            Factor::UnaryBoolean { operator, value }
        }
        aadlight_parser::Rule::factor => {
            // Handle nested factor rule
            transform_factor(first)
        }
        _ => panic!("Unexpected factor rule: {:?}", first.as_rule()),
    }
}

fn transform_value(pair: Pair<aadlight_parser::Rule>) -> Value {
    // value = value_constant | value_variable | ("(" ~ simple_expression ~ ")")
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        aadlight_parser::Rule::value_constant => Value::Constant(transform_value_constant(inner)),
        aadlight_parser::Rule::value_variable => Value::Variable(transform_value_variable(inner)),
        aadlight_parser::Rule::simple_expression => {
            // Handle parenthesized expression: (simple_expression)
            let expr = transform_simple_expression(inner);
            // Note: here SimpleExpression must be converted into ValueExpression
            // Since ValueExpression requires a Relation, we construct a simple one
            let relation = Relation {
                left: expr,
                comparison: None,
            };
            let value_expr = ValueExpression {
                left: relation,
                operations: Vec::new(),
            };
            Value::Expression(Box::new(value_expr))
        }
        _ => panic!("Unknown value rule: {:?}", inner.as_rule()),
    }
}

fn transform_value_constant(pair: Pair<aadlight_parser::Rule>) -> ValueConstant {
    // value_constant = number | boolean | string_literal
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        aadlight_parser::Rule::number => ValueConstant::Numeric(inner.as_str().trim().to_string()),
        aadlight_parser::Rule::boolean => {
            let val = match inner.as_str() {
                "true" => true,
                "false" => false,
                _ => panic!("Invalid boolean: {}", inner.as_str()),
            };
            ValueConstant::Boolean(val)
        }
        aadlight_parser::Rule::string_literal => {
            let raw = inner.as_str();
            let value = if raw.starts_with('"') && raw.ends_with('"') {
                raw[1..raw.len() - 1].to_string()
            } else {
                raw.to_string()
            };
            ValueConstant::String(value)
        }
        _ => panic!("Unknown value constant rule: {:?}", inner.as_rule()),
    }
}

fn transform_value_variable(pair: Pair<aadlight_parser::Rule>) -> ValueVariable {
    // value_variable = (identifier ~ "'count")
    //                | (identifier ~ "'fresh")
    //                | (identifier ~ "?")
    //                | identifier
    let text = pair.as_str();

    if text.ends_with("'count") {
        let port = text[..text.len() - 6].to_string();
        ValueVariable::PortCount(port)
    } else if text.ends_with("'fresh") {
        let port = text[..text.len() - 7].to_string();
        ValueVariable::PortFresh(port)
    } else if text.ends_with("?") {
        let port = text[..text.len() - 1].to_string();
        ValueVariable::IncomingPortCheck(port)
    } else {
        // Default to a local variable
        ValueVariable::LocalVariable(text.to_string())
    }
}
