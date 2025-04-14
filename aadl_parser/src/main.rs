pub mod aadlight_parser;

use aadlight_parser::AADLParser;
use pest::{iterators::Pair, Parser};

fn print_pair(pair: Pair<aadlight_parser::Rule>, indent: usize) {
    println!(
        "{}{:?}: {:?}",
        "  ".repeat(indent),
        pair.as_rule(),
        pair.as_str()
    );

    for inner in pair.into_inner() {
        print_pair(inner, indent + 1);
    }
}

fn main() {
    let aadl_input = r#"
        system MySystem
        implementation MySystem.impl
        subcomponents
            cpu: processor;
        end subcomponents;
        end MySystem;
    "#;

    match AADLParser::parse(aadlight_parser::Rule::file, aadl_input) {
        Ok(pairs) => {
            for pair in pairs {
                print_pair(pair, 0);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}