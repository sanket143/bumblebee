use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{AstNode, Reference, Semantic, SemanticBuilder, SemanticBuilderReturn};
use oxc_span::{GetSpan, SourceType};
use std::ffi::OsStr;
use walkdir::WalkDir;

fn debug_ast_node(node: &AstNode, semantic: &Semantic) {
    let nodes = semantic.nodes();
    let mut answer = None;

    for ancestor in nodes.ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::Program(_) => {}
            _ => {
                answer = Some(ancestor);
            }
        }
    }

    if let Some(answer) = answer {
        let span = answer.span();
        println!(
            "[DBG_AST_NODE] {:?} {}",
            answer.scope_id(),
            semantic
                .source_text()
                .get((span.start as usize)..(span.end as usize))
                .unwrap()
        );
    }
}

fn debug_reference(reference: &Reference, semantic: &Semantic) {
    let id = reference.symbol_id().unwrap();
    let references = semantic.symbol_references(id);

    debug_ast_node(semantic.nodes().get_node(reference.node_id()), semantic);

    for refer in references {
        if refer.symbol_id() != reference.symbol_id() {
            debug_reference(refer, semantic);
        }
    }
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
                program,
                errors: parsing_errors,
                panicked,
                ..
            } = Parser::new(&allocator, source_text.as_str(), source_type).parse();

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

            if !errors.is_empty() {
                for error in errors {
                    eprintln!("{error:?}");
                }

                panic!("Failed to build Semantic for Counter component.");
            }

            let symbol_table = semantic.symbols();

            for id in symbol_table.symbol_ids() {
                if symbol_table.get_name(id) == "call" {
                    let references = semantic.symbol_references(id);
                    let declaration = semantic.symbol_declaration(id);

                    print!("DECLARATION:");
                    debug_ast_node(declaration, &semantic);

                    for reference in references {
                        debug_reference(reference, &semantic);
                    }
                }
            }
        }
    }

    Ok(())
}
