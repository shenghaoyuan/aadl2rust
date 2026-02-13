use crate::aadl_ast2rust_code::intermediate_ast::*;

use crate::aadl_ast2rust_code::converter::AadlConverter;
use crate::aadl_ast2rust_code::tool::*;
use crate::ast::aadl_ast_cj::*;

pub fn convert_process_component(
    temp_converter: &AadlConverter,
    comp: &ComponentType,
) -> Vec<Item> {
    let mut items = Vec::new();

    // 1. Struct definition
    let mut fields = temp_converter.convert_type_features(&comp.features, comp.identifier.clone()); // Feature list

    // Add CPU ID field
    fields.push(Field {
        name: "cpu_id".to_string(),
        ty: Type::Named("isize".to_string()),
        docs: vec!["// Process CPU ID".to_string()],
        attrs: Vec::new(),
    });

    let struct_def = StructDef {
        name: format!("{}Process", to_upper_camel_case(&comp.identifier)),
        fields, // Feature list
        properties: temp_converter.convert_properties(ComponentRef::Type(comp)), // Property list, TODO: this seems unused, because current examples have no process properties
        generics: Vec::new(),
        derives: vec!["Debug".to_string()],
        docs: temp_converter.create_component_type_docs(comp),
        vis: Visibility::Public, // Default: public
    };
    items.push(Item::Struct(struct_def));

    items
}
