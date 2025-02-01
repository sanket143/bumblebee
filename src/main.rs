use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_ast::ast::Function;
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::SourceType;
use oxc_traverse::{traverse_mut, Traverse};
use std::ffi::OsStr;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Default, Clone)]
struct File {
    filepath: String,
}

impl File {
    pub fn new(filepath: String) -> Self {
        Self { filepath }
    }
}

impl<'a> Traverse<'a> for File {
    fn enter_arrow_function_expression(
        &mut self,
        node: &mut oxc_ast::ast::ArrowFunctionExpression<'a>,
        ctx: &mut oxc_traverse::TraverseCtx<'a>,
    ) {
        println!("{:#?}", &node);
    }
}

struct Block {
    //?
}

#[tokio::main]
async fn main() -> Result<()> {
    for entry in WalkDir::new("./test-dir").into_iter().flatten() {
        if entry.path().extension() == Some(OsStr::new("js")) {
            println!("{}", entry.path().display());
            let source_path = entry.path();
            let source_text = std::fs::read_to_string(entry.path())?;
            let allocator = Allocator::default();
            let source_type = SourceType::from_path(source_path)?;

            let mut errors = Vec::new();
            let ParserReturn {
                mut program,
                errors: parsing_errors,
                panicked,
                ..
            } = Parser::new(&allocator, source_text.as_str(), source_type).parse();
            let mut file = File::new(source_path.to_str().unwrap().to_owned());

            errors.extend(parsing_errors);

            if panicked {
                for error in &errors {
                    eprintln!("{error:?}");
                }

                panic!("Parsing failed.");
            }

            let SemanticBuilderReturn {
                semantic,
                errors: semantic_errors,
            } = SemanticBuilder::new().build(&program);

            errors.extend(semantic_errors);

            if errors.is_empty() {
                println!("parsing and semantic analysis completed successfully.");
            } else {
                for error in errors {
                    eprintln!("{error:?}");
                }

                panic!("Failed to build Semantic for Counter component.");
            }

            // println!("{:#?}", program);

            let (symbol_table, scope_tree) = semantic.into_symbol_table_and_scope_tree();

            traverse_mut(
                &mut file,
                &allocator,
                &mut program,
                symbol_table,
                scope_tree,
            );
            // we need to process each file
            // keep track of function call references
            // - in other functions
            // - other object functions
            // - callbacks?
        }
    }

    Ok(())
}
