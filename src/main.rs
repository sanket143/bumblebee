use anyhow::Result;
use core::hash::Hash;
use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_semantic::{
    AstNode, NodeId, Reference, Semantic, SemanticBuilder, SemanticBuilderReturn, SymbolId,
};
use oxc_span::{Atom, GetSpan, SourceType};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::Hasher,
    io::Write,
    mem::ManuallyDrop,
    path::Path,
};
use std::{ffi::OsStr, path::PathBuf};
use walkdir::WalkDir;

#[derive(PartialEq, Eq)]
struct Query {
    symbol: String, // e.g. call() symbol
    symbol_id: Option<SymbolId>,
    symbol_path: PathBuf, // from ./factory.js file
}

impl Hash for Query {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.symbol_id.hash(state);
        self.symbol_path.hash(state);
    }
}

impl Query {
    pub fn udpate_sumbol_id(&mut self, symbol_id: SymbolId) {
        self.symbol_id = Some(symbol_id);
    }
}

struct Service<'a> {
    semantic: Semantic<'a>,
    root_path: PathBuf,
    source_path: PathBuf,
}

impl<'a> Service<'a> {
    pub fn build(root_path: PathBuf, source_path: PathBuf, semantic: Semantic<'a>) -> Result<Self> {
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
        let scoping = self.semantic.scoping();

        let symbol_id = scoping
            .symbol_ids()
            .find(|&id| scoping.symbol_name(id) == symbol_name);

        if let Some(symbol_id) = symbol_id {
            self.semantic.symbol_declaration(symbol_id);
        }

        symbol_id
    }

    /// what should this return
    /// should `query` be mutable?
    pub fn find_references(&self, reference_node_ids: &mut HashSet<NodeId>, query: &Query) {
        let scoping = self.semantic.scoping();
        let query_source_path =
            resolve_import_path(&self.root_path, query.symbol_path.to_str().unwrap()).unwrap();

        // TODO: clean this path up
        let symbol_source_path = resolve_import_path(
            &self.root_path.join(".."),
            self.source_path.to_str().unwrap(),
        )
        .unwrap();

        println!("Finding references in: {}", self.source_path.display());

        for id in scoping.symbol_ids() {
            if scoping.symbol_name(id) == query.symbol {
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
                    if let Some(node_id) = debug_ast_node(declaration, &self.semantic) {
                        reference_node_ids.insert(node_id);
                    };
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
                        } else if let Some(node_id) = debug_ast_node(declaration, &self.semantic) {
                            reference_node_ids.insert(node_id);
                        };
                    }
                }

                let references = self.semantic.symbol_references(id);
                for reference in references {
                    debug_reference(reference, &self.semantic, reference_node_ids);
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

fn check_require<'a>(node: &'a AstNode) -> Option<Atom<'a>> {
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
                import_node = check_require(ancestor);

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

fn debug_ast_node(node: &AstNode, semantic: &Semantic) -> Option<NodeId> {
    let nodes = semantic.nodes();
    let mut answer = None;

    for ancestor in nodes.ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::Program(_) => {}
            _ => {
                answer = Some(ancestor.id());
            }
        }
    }

    answer
}

fn debug_reference(
    reference: &Reference,
    semantic: &Semantic,
    reference_node_ids: &mut HashSet<NodeId>,
) {
    let id = reference.symbol_id().unwrap();
    let references = semantic.symbol_references(id);

    let node_id = debug_ast_node(semantic.nodes().get_node(reference.node_id()), semantic);

    if let Some(node_id) = node_id {
        println!("{:?}", node_id);
        reference_node_ids.insert(node_id);
    }

    for refer in references {
        if refer.symbol_id() != reference.symbol_id() {
            debug_reference(refer, semantic, reference_node_ids);
        }
    }
}

pub fn eval_dir(root_path: &Path) -> Result<()> {
    // first update the queries to get declaration and add their symbolId? or maybe nodeId?
    // that'll require a service which parses and ...
    // maybe I can have 2 lists, one just to keep track if we've visisted or not
    // and the other to actually iterate and evaluate in the service
    //
    // Allocator has been a mystery to me
    // What are we even trying to achieve here?
    let mut allocator = Allocator::default();
    let mut queries = [Query {
        symbol: "call".into(),
        symbol_path: PathBuf::from("./factory.js"),
        symbol_id: None,
    }];
    let mut query_set = HashSet::new();
    let mut services = HashMap::new();
    let target_dir = Path::new("output");

    for query in queries.iter_mut() {
        let source_path = root_path.join(&query.symbol_path).canonicalize().unwrap();

        let source_text = std::fs::read_to_string(&source_path).unwrap();
        let source_type = SourceType::from_path(&source_path).unwrap();

        let source_text_ref = allocator.alloc_str(&source_text); // Hypothetical method

        let ParserReturn { program, .. } = &**allocator.alloc(ManuallyDrop::new(
            Parser::new(&allocator, source_text_ref, source_type).parse(),
        ));

        let SemanticBuilderReturn { semantic, .. } = SemanticBuilder::new().build(program);
        let service = Service::build(root_path.into(), source_path.to_owned(), semantic).unwrap();

        let symbol_id = service.get_symbol_id(&query.symbol);
        if let Some(symbol_id) = symbol_id {
            query.udpate_sumbol_id(symbol_id);
        }

        query_set.insert(query);
        services.insert(source_path.clone(), (service, HashSet::new()));
    }

    // let impacted_declarations = HashSet::new();

    // what should be the query structure
    // we'll see if there's any git diff parser, or a patch parser
    // TODO: Make this async
    for entry in WalkDir::new(root_path).into_iter().flatten() {
        if entry.path().extension() == Some(OsStr::new("js")) {
            let reference_node_ids: HashSet<NodeId> = HashSet::new();
            let source_path = root_path.join(entry.path()).canonicalize().unwrap();

            if services.get_mut(&source_path).is_none() {
                let source_text = std::fs::read_to_string(&source_path)?;
                let source_type = SourceType::from_path(&source_path)?;
                let source_text_ref = allocator.alloc_str(&source_text);

                let parser_return = allocator.alloc(ManuallyDrop::new(
                    Parser::new(&allocator, source_text_ref, source_type).parse(),
                ));

                let SemanticBuilderReturn { semantic, .. } =
                    SemanticBuilder::new().build(&parser_return.program);

                let service = Service::build(root_path.into(), source_path.to_owned(), semantic)?;

                services.insert(source_path.to_owned(), (service, reference_node_ids));
            }
        }
    }

    for query in query_set.iter() {
        for (_source_path, service) in services.iter_mut() {
            service.0.find_references(&mut service.1, query);
        }
    }

    std::fs::create_dir_all(target_dir).ok();

    services
        .iter()
        .for_each(|(source_path, (service, reference_node_ids))| {
            if !reference_node_ids.is_empty() {
                let mut reference_node_ids: Vec<NodeId> =
                    reference_node_ids.iter().copied().collect();
                reference_node_ids.sort_unstable();
                let relative_path = source_path.strip_prefix(root_path).unwrap();
                let target_path = target_dir.join(relative_path);
                // println!("{}: {:?}", target_path.display(), reference_node_ids);
                let mut file_stream = File::create(&target_path).unwrap();

                reference_node_ids.iter().for_each(|node_id| {
                    let node = service.semantic.nodes().get_node(*node_id);
                    let span = node.span();
                    let text = service
                        .semantic
                        .source_text()
                        .get((span.start as usize)..(span.end as usize))
                        .unwrap();

                    file_stream
                        .write_all((text.to_owned() + "\n\n").as_bytes())
                        .ok();
                    // println!("{}: {}", target_path.display(), text);
                });
            }
        });

    allocator.reset();
    Ok(())
}

// TODO: Handle symbol_is_mutated
// Get whether a symbol is mutated (i.e. assigned to).
// If symbol is const, always returns false. Otherwise, returns true if the symbol is assigned to somewhere in AST.
#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_test() {
        let home = std::env::current_dir().unwrap();
        assert!(eval_dir(&home.join("test-dir")).is_ok());
    }
}
