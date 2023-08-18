use crate::types::object::Serializer;
use oxc::{
    allocator::Allocator,
    ast::{
        ast::{ComputedMemberExpression, Expression, FormalParameters},
        AstKind, Visit,
    },
    parser::Parser,
    span::SourceType,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, fs::OpenOptions};
use std::{fs::File, io::Write};

mod evaluate;
mod types;

fn main() {
    let source_text = std::fs::read_to_string("./test.js").unwrap();
    let writer = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(r#"out.json"#)
        .unwrap();

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("index.js").unwrap();
    let ret = Parser::new(&allocator, source_text.as_str(), source_type).parse();

    let program = allocator.alloc(ret.program);
}
