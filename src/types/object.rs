use oxc::ast::ast::{BindingPattern, BindingPatternKind, ObjectPattern, PropertyKey};
use serde_json::{json, Value};

#[allow(unused)]
pub trait Serializer {
    fn serialize(&self) -> Option<Value>;
}

#[allow(unused)]
impl<'a> Serializer for BindingPattern<'a> {
    fn serialize(&self) -> Option<Value> {
        match &self.kind {
            BindingPatternKind::BindingIdentifier(pattern) => {
                Some(Value::String(pattern.name.to_string()))
            }
            BindingPatternKind::ObjectPattern(pattern) => pattern.serialize(),
            BindingPatternKind::ArrayPattern(pattern) => None,
            BindingPatternKind::AssignmentPattern(pattern) => None,
        }
    }
}

impl<'a> Serializer for ObjectPattern<'a> {
    fn serialize(&self) -> Option<Value> {
        let mut result = json!({});
        self.properties.iter().for_each(|x| {
            let key = x
                .key
                .serialize()
                .unwrap_or(Value::String("<undefined>".to_owned()));
            let value = x
                .value
                .serialize()
                .unwrap_or(Value::String("<undefined>".to_owned()));

            result[key.as_str().unwrap().to_string()] = value;
        });

        Some(result)
    }
}

impl<'a> Serializer for PropertyKey<'a> {
    fn serialize(&self) -> Option<Value> {
        match &self {
            PropertyKey::Identifier(iden) => Some(Value::String(iden.name.to_string())),
            PropertyKey::PrivateIdentifier(iden) => Some(Value::String(iden.name.to_string())),
            PropertyKey::Expression(expr) => None,
        }
    }
}
