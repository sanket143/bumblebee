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
use queso::Queso;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, fs::OpenOptions};
use std::{fs::File, io::Write};

mod document;
mod evaluate;
mod queso;
mod types;

fn main() {
    let folder_name = "./test-dir";
    let allocator = Allocator::default();
    let queso = Queso::new(folder_name, &allocator);
}
