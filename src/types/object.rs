use oxc::ast::ast::{BindingPattern, BindingPatternKind, ObjectPattern, PropertyKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Variable {
    pub name: String,
    pub is_phantom: bool,
    /// leaf properties which are in scope of the function
    pub exposed_properties: Vec<String>,
}

impl Variable {
    pub fn new(name: String, is_phantom: bool) -> Self {
        Self {
            name,
            is_phantom,
            ..Default::default()
        }
    }
}

#[allow(unused)]
pub trait Serializer<T> {
    fn serialize(&self) -> Option<T>;
}

#[allow(unused)]
impl<'a> Serializer<Variable> for BindingPattern<'a> {
    fn serialize(&self) -> Option<Variable> {
        match &self.kind {
            BindingPatternKind::BindingIdentifier(pattern) => {
                Some(Variable::new(pattern.name.to_string(), false))
            }
            // BindingPatternKind::ObjectPattern(pattern) => pattern.serialize(),
            BindingPatternKind::ArrayPattern(pattern) => {
                println!("ArrayPattern: {:#?}", pattern);
                Some(Variable::new("param".to_owned(), true))
            }
            // BindingPatternKind::AssignmentPattern(pattern) => None,
            _ => Some(Variable::new("param".to_owned(), true)),
        }
    }
}

impl<'a> Serializer<Value> for ObjectPattern<'a> {
    fn serialize(&self) -> Option<Value> {
        todo!("serializer for ObjectPattern");
        // let mut result = json!({});
        // self.properties.iter().for_each(|_x| {
        //     let key = x
        //         .key
        //         .serialize()
        //         .unwrap_or(Value::String("<undefined>".to_owned()));
        //     let value = x
        //         .value
        //         .serialize()
        //         .unwrap_or(Value::String("<undefined>".to_owned()));

        //     result[key.as_str().unwrap().to_string()] = value;
        // });

        // Some(result)
    }
}

impl<'a> Serializer<Value> for PropertyKey<'a> {
    fn serialize(&self) -> Option<Value> {
        match &self {
            PropertyKey::Identifier(iden) => Some(Value::String(iden.name.to_string())),
            PropertyKey::PrivateIdentifier(iden) => Some(Value::String(iden.name.to_string())),
            PropertyKey::Expression(_expr) => None,
        }
    }
}
