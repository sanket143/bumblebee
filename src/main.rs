use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_semantic::{AstNode, Reference, Semantic, SemanticBuilder, SemanticBuilderReturn};
use oxc_span::{GetSpan, SourceType};
use std::ffi::OsStr;
use walkdir::WalkDir;

fn resolve_import_path(specifier: &str) {
    println!("IMPORT:{}", specifier);

    let options = ResolveOptions {
        extensions: vec![".js".into()],
        extension_alias: vec![(".js".into(), vec![".ts".into(), ".js".into()])],
        condition_names: vec!["node".into(), "import".into(), "require".into()],
        ..ResolveOptions::default()
    };

    match Resolver::new(options).resolve(
        "/home/snket143/Remote/personal/bumblebee/test-dir",
        specifier,
    ) {
        Err(error) => println!("Error: {error}"),
        Ok(resolution) => println!("Resolved: {:?}", resolution.full_path()),
    }
}

fn debug_require(node: &AstNode, semantic: &Semantic) -> Option<String> {
    let vd = node.kind().as_variable_declarator();
    let mut specifier = None;

    if let Some(vd) = vd {
        if let Some(Expression::CallExpression(exp)) = &vd.init {
            if exp.callee_name().unwrap() == "require" {
                // we assume that require will always have exactly 1 arguemnt
                if let Argument::StringLiteral(sl) = &exp.arguments[0] {
                    specifier = Some(sl.value);
                }
            }
        }

        if specifier.is_some() {
            println!(
                "IDEN: {:#?}",
                vd.id.get_binding_identifiers()[0].symbol_id()
            );

            // I forgot why I was doing this?
            // Why do I need the node?
            // I guess to get the sumbol_id and using that symbol_id to find further
            // impacted areas (references)
            let node_id = semantic
                .symbols()
                .get_declaration(vd.id.get_binding_identifiers()[0].symbol_id());
            println!("{:#?}", semantic.nodes().get_node(node_id));
        }
    }

    if let Some(specifier) = specifier {
        resolve_import_path(specifier.into());
        return Some(specifier.into());
    }

    None
}

fn debug_import(node: &AstNode, semantic: &Semantic) {
    let nodes = semantic.nodes();
    let mut answer = None;

    for ancestor in nodes.ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::Program(_) => {
                break;
            }
            AstKind::ModuleDeclaration(oxc_ast::ast::ModuleDeclaration::ImportDeclaration(id)) => {
                answer = Some(id.source.value);
                break;
            }
            AstKind::VariableDeclarator(_) => {
                if debug_require(ancestor, semantic).is_some() {
                    println!("{:#?}", ancestor);
                    break;
                }
            }
            _ => {}
        }
    }

    // I somehow also need to keep track of what symbol were there in the import
    // or I can assume that we're finding reference of only 1 symbol at a time
    // and so there will never be the case when we reach require or import where
    // that symbol was not referred
    if let Some(specifier) = answer {
        resolve_import_path(specifier.into());
    }
}

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
    // what should be the query structure
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
                // can I update the symbol ids in all files such that they refer to the
                // correct symbol in the other file.
                // e.g. call imported in index.js and call of factory.js should have same symbol_id
                // that'll be too expensive I guess, too many symbols
                println!("{:?}, {}", id, symbol_table.get_name(id));
                if symbol_table.get_name(id) == "call" || symbol_table.get_name(id) == "a" {
                    let references = semantic.symbol_references(id);
                    let declaration = semantic.symbol_declaration(id);

                    // print!("DECLARATION:");
                    debug_import(declaration, &semantic);
                    debug_ast_node(declaration, &semantic);

                    for reference in references {
                        debug_reference(reference, &semantic);
                    }
                }
            }
        }

        println!("===============================================");
    }

    Ok(())
}
