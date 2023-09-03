use oxc::{allocator::Allocator, span::SourceType};

pub struct Document<'a> {
    uri: String,
    content: String,
    allocator: &'a Allocator,
    source_type: SourceType,
}

impl<'a> Document<'a> {
    pub fn new(uri: String, content: String, allocator: &'a Allocator) -> Self {
        Self {
            uri: uri.clone(),
            content,
            allocator,
            source_type: SourceType::from_path(uri.clone()).unwrap(),
        }
    }

    /// Parse the document to extract functions
    /// - input type definitions
    /// - return types
    pub fn parse(&self) {
        println!("{}", self.content);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_function_names() {
        let source_text = r#"
function giveMeThat(data, config) {
  console.log(data.id);
  console.log(data.items, data["id"]);
  const input = data.items[0].id;

  call(input);
}

function call({ token }, { config }, ...args) {
  console.log(toast);
}

giveMeThat(data);
        "#;

        let allocator = Allocator::default();
        let document = Document::new("test.js".to_owned(), source_text.to_owned(), &allocator);

        let ret = document.parse();
    }
}
