use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "aadl.pest"]
pub struct AADLParser;