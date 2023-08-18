use crate::types::Serializer;
use oxc::ast::{
    ast::{ComputedMemberExpression, Expression, FormalParameters},
    AstKind, Visit,
};
use serde_json::{json, Value};
use std::collections::HashMap;

pub(super) struct Visitor<'a> {
    // how to store the type of the variable such that it can later be verified
    source_text: &'a str,
    variables: HashMap<String, Value>,
    scope: u8,
    is_variable: bool,
    is_callee: bool,
    functions: Vec<&'a str>,
    functions_data: Value,
    callee: String,
    tracked_variables: Vec<&'a str>,
}

#[derive(Debug)]
struct QuesoFunction {
    name: String,
}

#[allow(unused)]
impl<'a> Visitor<'a> {
    fn print(&self, input: &str) {
        // println!(
        //     "[{}] {}() {}",
        //     self.scope,
        //     self.functions.last().unwrap_or(&"<root>"),
        //     input
        // );
    }

    pub fn new(source_text: &'a str) -> Self {
        Self {
            source_text,
            variables: HashMap::new(),
            functions: Vec::new(),
            functions_data: json!({}),
            scope: 0,
            is_variable: false,
            is_callee: false,
            callee: String::new(),
            tracked_variables: Vec::new(),
        }
    }

    fn visit_function_parameters(&self, function_name: String) {}
}

#[allow(unused)]
impl<'a> Visit<'a> for Visitor<'a> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match &kind {
            AstKind::IdentifierReference(iden) => {
                println!("IDEN, REFERE {}", iden.name);
            }
            AstKind::VariableDeclarator(decl) => {
                // process variable to get the initial type
                self.is_variable = true;
            }
            AstKind::BindingIdentifier(iden) => {
                if self.is_variable {
                    self.tracked_variables.push(iden.name.as_str());
                    self.print(iden.name.as_str());
                }
            }
            AstKind::Function(function) => {
                if let Some(name) = &function.id {
                    let function_name = name.name.as_str();

                    self.functions_data[function_name] = json!({ "name": function_name});
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
            }
            AstKind::Function(function) => {
                let source_text = self.source_text;

                if let Some(name) = &function.id {
                    let function_name = self.functions.pop().unwrap_or("");
                    let mut value = self.functions_data[&function_name].to_string();
                    value.push('\n');

                    self.functions_data
                        .as_object_mut()
                        .unwrap()
                        .remove(&function_name.to_owned());
                }
                // println!("variables: {:#?}", self.tracked_variables);
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
        if params.rest.is_some() {
            // println!("WARNING: RestElement is not handled yet!");
        }

        let cur_function = self.functions.last().unwrap();
        let mut function_params: Vec<Value> = vec![];

        for (index, param) in (0_u8..).zip(&params.items) {
            let params = param.pattern.serialize().unwrap();
            let mut param_name = params.name;
            if params.is_phantom {
                param_name = format!("<{}-{index}>", param_name);
            }

            // println!("params: {param_name}");
            function_params.push(Value::String(param_name));
        }
        self.functions_data[cur_function]["params"] = Value::Array(function_params);

        if let Some(rest) = &params.rest {
            self.visit_rest_element(rest);
        }
    }

    fn visit_computed_member_expression(&mut self, expr: &'a ComputedMemberExpression<'a>) {
        let cur_function = &self.functions.last().unwrap();

        if let Expression::Identifier(iden) = &expr.object {
            if let Expression::StringLiteral(value) = &expr.expression {
                self.functions_data[cur_function]["variables"] = Value::Array(vec![]);
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
