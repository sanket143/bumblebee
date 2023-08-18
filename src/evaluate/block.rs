use std::path::Path;

use oxc::{
    allocator::Allocator,
    ast::AstKind,
    parser::{Parser, ParserReturn},
    span::SourceType,
};

/// Represents either a Block statement or a function statement
struct Block<'a> {
    // will come back to this
    allocator: &'a Allocator,
    source_type: SourceType,
    source_text: String,
}

impl<'a> Block<'a> {
    /// Get all the defined variables in the block
    /// and their tentative types
    pub fn evaluate(&self) {}

    pub fn from_file(file_path: &Path, allocator: &'a Allocator) -> Self {
        let source_text = std::fs::read_to_string(file_path).unwrap();
        let source_type = SourceType::from_path(file_path).unwrap();

        Self {
            allocator,
            source_type,
            source_text,
        }
    }

    pub fn from_source(
        source_text: String,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> Self {
        Self {
            allocator,
            source_type,
            source_text,
        }
    }

    pub fn parse(&'a self) -> ParserReturn<'a> {
        let parser_ret =
            Parser::new(self.allocator, self.source_text.as_str(), self.source_type).parse();

        parser_ret
    }
}

#[cfg(test)]
mod test {
    use crate::evaluate::visitor::Visitor;
    use oxc::{allocator::Allocator, ast::Visit, span::SourceType};

    use super::Block;

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
        let block = Block::from_source(
            source_text.to_string(),
            SourceType::from_path("index.js").unwrap(),
            &allocator,
        );

        let ret = block.parse();
        let program = allocator.alloc(ret.program);

        let mut visitor = Visitor::new(source_text);
        visitor.visit_program(program);
    }
}
