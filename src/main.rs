use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{Argument, Expression, Program},
    AstKind,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_semantic::{
    AstNode, Reference, Semantic, SemanticBuilder, SemanticBuilderReturn, SymbolId,
};
use oxc_span::{Atom, GetSpan, SourceType};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use std::{ffi::OsStr, path::PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Hash, PartialEq, Eq)]
struct Query {
    symbol: String,       // e.g. call() symbol
    symbol_path: PathBuf, // from ./factory.js file
}

struct Project<'a> {
    allocator: Allocator,
    services: Vec<Service<'a>>,
}

impl<'a> Project<'a> {
    pub fn new() -> Self {
        let allocator = Allocator::default();

        Project {
            allocator,
            services: vec![],
        }
    }
    pub fn add_service(&'a mut self, root_path: &PathBuf, source_path: &PathBuf) {
        let source_text = std::fs::read_to_string(&source_path).unwrap();
        let source_type = SourceType::from_path(&source_path).unwrap();

        let ParserReturn { program, .. } =
            Parser::new(&self.allocator, &source_text, source_type).parse();
        let service = Service::build(root_path.into(), source_path.to_owned(), &program).unwrap();

        self.services.push(service);
    }
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

    /// what do we need?
    /// nodeId, more useful
    /// we can get node_id from symbol_id but can't go other way around I think
    /// yes, no symbol_id from node_id
    pub fn get_symbol_id(&self, symbol_name: &str) -> Option<SymbolId> {
        let symbol_table = self.semantic.symbols();

        let symbol_id = symbol_table
            .symbol_ids()
            .find(|&id| symbol_table.get_name(id) == symbol_name);

        if let Some(symbol_id) = symbol_id {
            self.semantic.symbol_declaration(symbol_id);
        }

        symbol_id
    }

    /// what should this return
    /// should `query` be mutable?
    pub fn find_references(&self, query: &Query) {
        let symbol_table = self.semantic.symbols();
        let query_source_path =
            resolve_import_path(&self.root_path, query.symbol_path.to_str().unwrap()).unwrap();

        // TODO: clean this path up
        let symbol_source_path = resolve_import_path(
            &self.root_path.join(".."),
            self.source_path.to_str().unwrap(),
        )
        .unwrap();

        println!("Finding references in: {}", self.source_path.display());

        for id in symbol_table.symbol_ids() {
            if symbol_table.get_name(id) == query.symbol {
                let declaration = self.semantic.symbol_declaration(id);

                if query_source_path == symbol_source_path {
                    // can we store all of these as symbolIds? and dump the declaration of all of these
                    // in the file in the end?
                    // it'll also be easier to maintain the unique symbolIds that way.
                    //
                    // One more check in declaration, if it's not an import but a declaration
                    // then check if the declaration file and query symbol file path is same
                    // How do I know what's the file of the declaration? source_path? I guess
                    //
                    // symbol_id of the declaration being calculated here
                    debug_ast_node(declaration, &self.semantic);
                } else {
                    // Check if the declaration is an import or require statement
                    // If it is then we need to check the source path
                    // If that's the same as the query or not
                    //
                    // How do I know if the declaration is an import?
                    let import_path = check_import(&self.root_path, declaration, &self.semantic);

                    if let Some(import_path) = import_path {
                        let import_path = self.root_path.join(import_path);
                        let query_source_path = resolve_import_path(
                            &self.root_path,
                            query.symbol_path.to_str().unwrap(),
                        )
                        .unwrap();

                        // there could be symbols with same name in multiple files
                        // verify if the query symbol is of same imported from same file as
                        // mentioned in the query
                        if import_path != query_source_path {
                            continue;
                        }
                    }
                }

                let references = self.semantic.symbol_references(id);
                for reference in references {
                    debug_reference(reference, &self.semantic);
                }
            }
        }
    }
}

fn resolve_import_path(root_path: &PathBuf, specifier: &str) -> Result<PathBuf> {
    let options = ResolveOptions {
        extensions: vec![".js".into()],
        extension_alias: vec![(".js".into(), vec![".ts".into(), ".js".into()])],
        condition_names: vec!["node".into(), "import".into(), "require".into()],
        ..ResolveOptions::default()
    };

    let full_path = Resolver::new(options)
        .resolve(root_path, specifier)?
        .full_path();

    Ok(full_path)
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

fn check_import(root_path: &PathBuf, node: &AstNode, semantic: &Semantic) -> Option<PathBuf> {
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
        return Some(resolve_import_path(root_path, specifier.into()).unwrap());
    }

    None
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
            "[DBG_AST_NODE] {}",
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

// TODO: Handle symbol_is_mutated
// Get whether a symbol is mutated (i.e. assigned to).
// If symbol is const, always returns false. Otherwise, returns true if the symbol is assigned to somewhere in AST.
#[tokio::main]
async fn main() -> Result<()> {
    let root_path = "/home/snket143/Remote/personal/bumblebee/test-dir";

    // first update the queries to get declaration and add their symbolId? or maybe nodeId?
    // that'll require a service which parses and ...
    // maybe I can have 2 lists, one just to keep track if we've visisted or not
    // and the other to actually iterate and evaluate in the service
    let mut query_set = HashSet::from([
        Query {
            // I somehow want to get symbol_id here
            // How should this queries be modified in runtime such that
            // more impacted areas can be added as queries in it
            // or should it be a separate mutatabl vector?
            symbol: "call".into(),
            symbol_path: PathBuf::from("./factory.js"),
        },
        Query {
            symbol: "a".into(),
            symbol_path: PathBuf::from("./utils.js"),
        },
    ]);

    let mut services = HashMap::new();
    let allocator = Allocator::default();

    let mut queries = query_set.iter().map(|query| {
        let source_path = Path::new(root_path).join(&query.symbol_path);
        let source_text = std::fs::read_to_string(&source_path).unwrap();
        let source_type = SourceType::from_path(&source_path).unwrap();

        let ParserReturn { program, .. } =
            Parser::new(&allocator, &source_text, source_type).parse();

        // I don't wanna do this
        // This will build it twice, once here and other one when we traverse the project
        let service = Service::build(root_path.into(), source_path.to_owned(), &program).unwrap();

        // what do I do?
        services.insert(source_path.clone(), service);
        // TODO: Make this async
        for query in &query_set {
            println!(
                "{}: {}: {:?}",
                source_path.display(),
                query.symbol,
                service.get_symbol_id(&query.symbol)
            );
            service.find_references(query);
        }
    });

    // let impacted_declarations = HashSet::new();

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
            for query in &query_set {
                println!(
                    "{}: {}: {:?}",
                    source_path.display(),
                    query.symbol,
                    service.get_symbol_id(&query.symbol)
                );
                service.find_references(query);
            }
        }
    }

    Ok(())
}
