use anyhow::Result;
use oxc_ast::{
    ast::{
        Argument, ArrayPattern, BindingPattern, BindingPatternKind, Expression, ObjectPattern,
        VariableDeclarator,
    },
    AstKind,
};
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_semantic::{AstNode, NodeId, Reference, Semantic, SymbolId};
use oxc_span::Atom;
use std::{collections::HashSet, path::PathBuf};

use crate::query::Query;

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

fn get_symbol_ids_from_binding_pattern(
    binding_pattern: &BindingPattern,
    symbol_ids: &mut Vec<SymbolId>,
) {
    match &binding_pattern.kind {
        BindingPatternKind::BindingIdentifier(binding_indentifier) => {
            symbol_ids.push(binding_indentifier.symbol_id());
        }
        BindingPatternKind::ObjectPattern(object_pattern) => {
            get_symbol_ids_from_object_pattern(object_pattern, symbol_ids)
        }
        BindingPatternKind::ArrayPattern(array_pattern) => {
            get_symbol_ids_from_array_pattern(array_pattern, symbol_ids)
        }
        BindingPatternKind::AssignmentPattern(_) => {}
    }
}

fn get_symbol_ids_from_array_pattern(array_pattern: &ArrayPattern, symbol_ids: &mut Vec<SymbolId>) {
    for element in array_pattern.elements.iter().flatten() {
        get_symbol_ids_from_binding_pattern(element, symbol_ids);
    }
}

fn get_symbol_ids_from_object_pattern(
    object_pattern: &ObjectPattern,
    symbol_ids: &mut Vec<SymbolId>,
) {
    for prop in object_pattern.properties.iter() {
        get_symbol_ids_from_binding_pattern(&prop.value, symbol_ids);
    }
}

fn get_symbol_ids_from_variable_declarator(
    node: &VariableDeclarator,
    symbol_ids: &mut Vec<SymbolId>,
) {
    get_symbol_ids_from_binding_pattern(&node.id, symbol_ids);
}

fn debug_ast_node(node: &AstNode, semantic: &Semantic) -> (Option<NodeId>, Vec<SymbolId>) {
    let nodes = semantic.nodes();
    let mut answer = (None, Vec::new());

    for ancestor in nodes.ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::Program(_) => {}
            AstKind::Function(func) => {
                if let Some(id) = &func.id {
                    answer.1.push(id.symbol_id());
                }
                answer.0 = Some(ancestor.id());
            }
            AstKind::VariableDeclarator(vd) => {
                get_symbol_ids_from_variable_declarator(vd, &mut answer.1);
                // Recursively parse until we get list of binding identifiers
                // BindingPatternKind
                // - Identifier
                // - Object
                //   - properties
                //     - [BindingProperty]
                //       - value = BindingPattern
                // - Array
                // - Assignment
                // answer.1 = vd.id.kind.get_identifier_name();
                answer.0 = Some(ancestor.id());
            }
            _ => {
                answer.0 = Some(ancestor.id());
            }
        }
    }

    answer.1.iter().for_each(|x| {
        let symbol_name = semantic.scoping().symbol_name(*x);
        println!("SymbolName: {}", symbol_name);
    });

    answer
}

fn debug_reference(
    reference: &Reference,
    semantic: &Semantic,
    reference_node_ids: &mut HashSet<NodeId>,
) {
    let id = reference.symbol_id().unwrap();
    let references = semantic.symbol_references(id);

    let (node_id, _) = debug_ast_node(semantic.nodes().get_node(reference.node_id()), semantic);

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

pub struct Service<'a> {
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

    pub fn get_semantic(&'a self) -> &'a Semantic<'a> {
        &self.semantic
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
            resolve_import_path(&self.root_path, query.symbol_path().to_str().unwrap()).unwrap();

        // TODO: clean this path up
        let symbol_source_path = resolve_import_path(
            &self.root_path.join(".."),
            self.source_path.to_str().unwrap(),
        )
        .unwrap();

        println!("Finding references in: {}", self.source_path.display());

        for id in scoping.symbol_ids() {
            if scoping.symbol_name(id) == query.symbol() {
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
                    if let (Some(node_id), _symbol_ids) =
                        debug_ast_node(declaration, &self.semantic)
                    {
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
                            query.symbol_path().to_str().unwrap(),
                        )
                        .unwrap();

                        // there could be symbols with same name in multiple files
                        // verify if the query symbol is of same imported from same file as
                        // mentioned in the query
                        if import_path != query_source_path {
                            continue;
                        } else if let (Some(node_id), _) =
                            debug_ast_node(declaration, &self.semantic)
                        {
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
