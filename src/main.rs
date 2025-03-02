use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{Argument, Expression, Program},
    AstKind,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_semantic::{AstNode, Reference, Semantic, SemanticBuilder, SemanticBuilderReturn};
use oxc_span::{Atom, GetSpan, SourceType};
use std::{ffi::OsStr, path::PathBuf};
use walkdir::WalkDir;

struct Query {
    symbol: String,       // e.g. call() symbol
    symbol_path: PathBuf, // from ./factory.js file
}

struct Service<'a> {
    semantic: Semantic<'a>,
    root_path: PathBuf,
    source_path: PathBuf,
}

impl<'a> Service<'a> {
    pub fn build(
        root_path: PathBuf,
        source_path: PathBuf,
        program: &'a Program<'a>,
    ) -> Result<Self> {
        let SemanticBuilderReturn { semantic, .. } = SemanticBuilder::new().build(program);

        Ok(Self {
            semantic,
            root_path,
            source_path,
        })
    }

    pub fn find_references(&self, query: &Query) {
        let symbol_table = self.semantic.symbols();
        // first look for the reference

        for id in symbol_table.symbol_ids() {
            // println!("{:?}, {}", id, symbol_table.get_name(id));
            if symbol_table.get_name(id) == query.symbol {
                let references = self.semantic.symbol_references(id);
                let declaration = self.semantic.symbol_declaration(id);

                // Check if the declaration is an import or require statement
                // If it is then we need to check the source path
                // If that's the same as the query or not
                //
                // How do I know if the declaration is an import?
                check_import(declaration, &self.semantic);

                // can we store all of these as symbolIds? and dump the declaration of all of these
                // in the file in the end?
                // it'll also be easier to maintain the unique symbolIds that way.
                debug_ast_node(declaration, &self.semantic);

                for reference in references {
                    debug_reference(reference, &self.semantic);
                }
            }
        }
    }
}

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

fn check_require<'a>(node: &'a AstNode, semantic: &'a Semantic) -> Option<Atom<'a>> {
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
            let symbol_id = vd.id.get_binding_identifiers()[0].symbol_id();
            // I forgot why I was doing this?
            // Why do I need the node?
            // I guess to get the sumbol_id and using that symbol_id to find further
            // impacted areas (references)
            let node_id = semantic.symbols().get_declaration(symbol_id);
            // println!("{:#?}", semantic.nodes().get_node(node_id));
        }
    }

    specifier
}

fn check_import(node: &AstNode, semantic: &Semantic) -> bool {
    let nodes = semantic.nodes();
    let mut import_node = None;

    for ancestor in nodes.ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::Program(_) => {
                break;
            }
            AstKind::ModuleDeclaration(oxc_ast::ast::ModuleDeclaration::ImportDeclaration(id)) => {
                import_node = Some(id.source.value);
                break;
            }
            AstKind::VariableDeclarator(_) => {
                import_node = check_require(ancestor, semantic);

                if import_node.is_some() {
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
    if let Some(specifier) = import_node {
        resolve_import_path(specifier.into());
        return true;
    }

    false
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
    let root_path = "/home/snket143/Remote/personal/bumblebee/test-dir";
    let queries = [
        Query {
            symbol: "call".into(),
            symbol_path: PathBuf::from("./factory.js"),
        },
        Query {
            symbol: "a".into(),
            symbol_path: PathBuf::from("./utils.js"),
        },
    ];

    // what should be the query structure
    // we'll see if there's any git diff parser, or a patch parser
    // TODO: Make this async
    for entry in WalkDir::new("./test-dir").into_iter().flatten() {
        if entry.path().extension() == Some(OsStr::new("js")) {
            let source_path = entry.path();
            let source_text = std::fs::read_to_string(source_path)?;
            let allocator = Allocator::default();
            let source_type = SourceType::from_path(source_path)?;

            let ParserReturn { program, .. } =
                Parser::new(&allocator, &source_text, source_type).parse();
            let service = Service::build(root_path.into(), source_path.to_owned(), &program)?;

            // TODO: Make this async
            for query in &queries {
                service.find_references(query);
            }
        }
    }

    Ok(())
}
