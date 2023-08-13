use oxc::{
    allocator::Allocator,
    ast::{
        ast::{ComputedMemberExpression, Expression, FormalParameters},
        AstKind, Visit,
    },
    parser::Parser,
    semantic::SemanticBuilder,
    span::SourceType,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::{fs::File, io::Write};

use crate::types::object::Serializer;

mod types;

#[allow(unused)]
struct Queso {
    source_text: String,
    variables: Vec<String>,
    scope: u8,
    is_variable: bool,
    is_callee: bool,
    functions: Vec<String>,
    functions_data: Value,
    callee: String,
    writer: File,
}

#[derive(Serialize, Deserialize, Debug)]
struct QuesoFunction {
    name: String,
}

#[allow(unused)]
impl Queso {
    fn print(&self, input: &str) {
        // println!("{:?}", self.functions);
        println!(
            "[{}] {}() {}",
            self.scope,
            self.functions.last().unwrap_or(&"<root>".to_owned()),
            input
        );
    }

    fn new(source_text: String, writer: File) -> Self {
        Self {
            source_text,
            variables: Vec::new(),
            functions: Vec::new(),
            functions_data: json!({}),
            writer,
            scope: 0,
            is_variable: false,
            is_callee: false,
            callee: String::new(),
        }
    }

    fn visit_function_parameters(&self, function_name: String) {}
}

#[allow(unused)]
impl<'a> Visit<'a> for Queso {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match &kind {
            AstKind::VariableDeclarator(decl) => {
                self.is_variable = true;
            }
            AstKind::BindingIdentifier(iden) => {
                if self.is_variable {
                    self.print(iden.name.as_str());
                }
            }
            AstKind::Function(function) => {
                if let Some(name) = &function.id {
                    let function_name = name.name.to_string();

                    self.functions_data[function_name.clone()] = json!({ "name": function_name});
                    self.functions.push(function_name);
                }
                self.scope += 1;
            }
            AstKind::BlockStatement(block) => {
                self.scope += 1;
            }
            AstKind::CallExpression(call) => {
                self.is_callee = true;
            }
            _ => {}
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        match &kind {
            AstKind::VariableDeclarator(decl) => {
                self.is_variable = false;
                // println!("[{}] Enter: {:#?}", self.scope, decl.id);
            }
            AstKind::Function(function) => {
                let source_text = self.source_text.as_str();
                let start: usize = function.params.span.start as usize;
                let end: usize = function.params.span.end as usize;
                let sub = &source_text[start..end];
                println!("{:#?}", sub);
                if let Some(name) = &function.id {
                    let function_name = self.functions.pop().unwrap_or("".to_owned());
                    let mut value = self.functions_data[&function_name].to_string();
                    value.push('\n');

                    self.functions_data
                        .as_object_mut()
                        .unwrap()
                        .remove(&function_name);
                    self.writer
                        .write_all(value.as_bytes())
                        .expect("should be able to write");
                }
                self.scope -= 1;
            }
            AstKind::BlockStatement(block) => {
                self.scope -= 1;
            }
            AstKind::CallExpression(call) => {}
            _ => {
                self.is_callee = true;
            }
        }
    }

    fn visit_formal_parameters(&mut self, params: &'a FormalParameters<'a>) {
        let cur_function = self.functions.last().unwrap();
        let mut function_params: Vec<Value> = vec![];
        for param in &params.items {
            let params = param.pattern.serialize().unwrap();
            function_params.push(params);
        }
        self.functions_data[cur_function]["params"] = Value::Array(function_params);

        if let Some(rest) = &params.rest {
            self.visit_rest_element(rest);
        }
    }

    fn visit_computed_member_expression(&mut self, expr: &'a ComputedMemberExpression<'a>) {
        if let Expression::Identifier(iden) = &expr.object {
            if let Expression::StringLiteral(value) = &expr.expression {
                self.print(format!("{}[{}]", iden.name, value.value).as_str());
            }
        }
    }

    fn visit_static_member_expression(
        &mut self,
        expr: &'a oxc::ast::ast::StaticMemberExpression<'a>,
    ) {
        if let Expression::Identifier(iden) = &expr.object {
            self.print(format!("{}.{}", iden.name, expr.property.name).as_str());
        }
    }
}

fn main() {
    custom_oxc();
    // evaluate_oxc();
}

#[allow(unused)]
fn evaluate_oxc() {
    let source_text = std::fs::read_to_string("./test.js").unwrap();
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("index.js").unwrap();
    let ret = Parser::new(&allocator, source_text.as_str(), source_type).parse();

    let program = allocator.alloc(ret.program);
    let semantic_ret = SemanticBuilder::new(source_text.as_str(), source_type)
        .with_trivias(&ret.trivias)
        .build(program);
    let semantic = semantic_ret.semantic;
    println!("{:#?}", &semantic.scopes());
    println!("{:#?}", &semantic.module_record());
    println!("{:#?}", &semantic.symbols());

    // for node in semantic.nodes().iter() {
    //     // node.traverse();
    //     // if let AstKind::Function(func) = &node.kind() {
    //     //     let jsdoc = semantic.jsdoc().get_by_node(node);
    //     //     println!("{:#?}", jsdoc);
    //     // }
    // }
}

fn custom_oxc() {
    let source_text = std::fs::read_to_string("./test.js").unwrap();
    let writer = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(r#"out.file"#)
        .unwrap();

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("index.js").unwrap();
    let ret = Parser::new(&allocator, source_text.as_str(), source_type).parse();

    let program = allocator.alloc(ret.program);
    let mut queso = Queso::new(source_text.clone(), writer);
    queso.visit_program(program);
    queso.writer.flush().unwrap();
    // let semantic_ret = SemanticBuilder::new(source_text, source_type)
    //     .with_trivias(&ret.trivias)
    //     .build(program);
    // let semantic = semantic_ret.semantic;
    // println!("{:#?}", &semantic.scopes());
    // println!("{:#?}", &semantic.module_record());
    // println!("{:#?}", &semantic.symbols());

    // for node in semantic.nodes().iter() {
    //     // node.traverse();
    //     // if let AstKind::Function(func) = &node.kind() {
    //     //     let jsdoc = semantic.jsdoc().get_by_node(node);
    //     //     println!("{:#?}", jsdoc);
    //     // }
    // }
}
