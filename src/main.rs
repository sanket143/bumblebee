mod query;
mod service;

use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn, SymbolId};
use oxc_span::{GetSpan, SourceType};
use query::Query;
use service::Service;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    mem::ManuallyDrop,
    path::Path,
};
use std::{ffi::OsStr, path::PathBuf};
use walkdir::WalkDir;

struct ServiceReference<'a> {
    service: &'a Service<'a>,
    reference_symbol_ids: &'a mut HashSet<SymbolId>,
}

impl<'a> ServiceReference<'a> {
    pub fn new(service: &'a Service<'a>, reference_symbol_ids: &'a mut HashSet<SymbolId>) -> Self {
        Self {
            service,
            reference_symbol_ids,
        }
    }

    pub fn find_references(&mut self, query: &Query) {
        self.service
            .find_references(self.reference_symbol_ids, query);
    }
}

struct Bumblebee<'a> {
    root_path: &'a Path,
    target_dir: &'a Path,
    allocator: &'a Allocator,
    queries: HashSet<Query>,
    services: HashMap<PathBuf, &'a mut ServiceReference<'a>>,
}

impl<'a> Bumblebee<'a> {
    pub fn new(root_path: &'a Path, target_dir: &'a Path, allocator: &'a mut Allocator) -> Self {
        Self {
            root_path,
            target_dir,
            allocator,
            queries: Default::default(),
            services: Default::default(),
        }
    }

    pub fn evaluate_query(&mut self, query: Query) {
        let source_path = self
            .root_path
            .join(query.symbol_path())
            .canonicalize()
            .unwrap();
        let source_text = std::fs::read_to_string(&source_path).unwrap();
        let source_type = SourceType::from_path(&source_path).unwrap();
        let source_text_ref = self.allocator.alloc_str(&source_text);

        let ParserReturn { program, .. } = &**self.allocator.alloc(ManuallyDrop::new(
            Parser::new(self.allocator, source_text_ref, source_type).parse(),
        ));

        let SemanticBuilderReturn { semantic, .. } = SemanticBuilder::new().build(program);
        let service = &**self.allocator.alloc(ManuallyDrop::new(
            Service::build(self.root_path.into(), source_path.to_owned(), semantic).unwrap(),
        ));
        let reference_symbol_ids = &mut **self.allocator.alloc(ManuallyDrop::new(HashSet::new()));
        let symbol_id = service.get_symbol_id(query.symbol());

        if let Some(symbol_id) = symbol_id {
            let symbol_name = service.get_semantic().scoping().symbol_name(symbol_id);

            self.queries.insert(Query::new(
                symbol_id,
                symbol_name.into(),
                query.symbol_path().to_path_buf(),
            ));
        }

        let service_reference =
            &mut **self
                .allocator
                .alloc(ManuallyDrop::new(ServiceReference::new(
                    service,
                    reference_symbol_ids,
                )));

        self.services.insert(source_path.clone(), service_reference);
    }

    pub fn update_services(&mut self) -> Result<()> {
        for entry in WalkDir::new(self.root_path).into_iter().flatten() {
            if entry.path().extension() == Some(OsStr::new("js")) {
                let reference_symbol_ids =
                    &mut **self.allocator.alloc(ManuallyDrop::new(HashSet::new()));
                let source_path = self.root_path.join(entry.path()).canonicalize().unwrap();

                if self.services.get_mut(&source_path).is_none() {
                    let source_text = std::fs::read_to_string(&source_path)?;
                    let source_type = SourceType::from_path(&source_path)?;
                    let source_text_ref = self.allocator.alloc_str(&source_text);

                    let parser_return = self.allocator.alloc(ManuallyDrop::new(
                        Parser::new(self.allocator, source_text_ref, source_type).parse(),
                    ));

                    let SemanticBuilderReturn { semantic, .. } =
                        SemanticBuilder::new().build(&parser_return.program);

                    let service = &**self.allocator.alloc(ManuallyDrop::new(Service::build(
                        self.root_path.into(),
                        source_path.to_owned(),
                        semantic,
                    )?));

                    let service_reference =
                        &mut **self
                            .allocator
                            .alloc(ManuallyDrop::new(ServiceReference::new(
                                service,
                                reference_symbol_ids,
                            )));

                    self.services
                        .insert(source_path.to_owned(), service_reference);
                }
            }
        }

        Ok(())
    }

    pub fn find_references_recursively(&mut self) {
        for query in self.queries.iter() {
            for (_, service_reference) in self.services.iter_mut() {
                (*service_reference).find_references(query);

                println!(
                    "service_reference: {:#?}",
                    service_reference.reference_symbol_ids
                );
            }
        }
    }

    pub fn dump_reference_files(&self) {
        std::fs::create_dir_all(self.target_dir).ok();

        self.services
            .iter()
            .for_each(|(source_path, service_reference)| {
                println!(
                    "{}: {:?}",
                    source_path.display(),
                    service_reference.reference_symbol_ids
                );
                if !service_reference.reference_symbol_ids.is_empty() {
                    let mut reference_symbol_ids: Vec<SymbolId> = service_reference
                        .reference_symbol_ids
                        .iter()
                        .copied()
                        .collect();
                    reference_symbol_ids.sort_unstable();
                    let relative_path = source_path.strip_prefix(self.root_path).unwrap();
                    let target_path = self.target_dir.join(relative_path);
                    let mut file_stream = File::create(&target_path).unwrap();

                    reference_symbol_ids.iter().for_each(|symbol_id| {
                        let node_id = service_reference
                            .service
                            .get_semantic()
                            .scoping()
                            .symbol_declaration(*symbol_id);
                        let node = service_reference
                            .service
                            .get_semantic()
                            .nodes()
                            .get_node(node_id);
                        let span = node.span();
                        let text = service_reference
                            .service
                            .get_semantic()
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
    }
}

fn eval_dir<'a>(bumblebee: &'a mut Bumblebee<'a>) -> Result<()> {
    let queries = [Query::new_with_symbol(
        "call".into(),
        PathBuf::from("./factory.js"),
    )];

    for query in queries {
        println!("{:?}", query);
        bumblebee.evaluate_query(query);
    }

    bumblebee.update_services().unwrap();
    bumblebee.find_references_recursively();
    bumblebee.dump_reference_files();

    Ok(())
}

// TODO: Handle symbol_is_mutated
// Get whether a symbol is mutated (i.e. assigned to).
// If symbol is const, always returns false. Otherwise, returns true if the symbol is assigned to somewhere in AST.
#[tokio::main]
async fn main() -> Result<()> {
    let mut allocator = Allocator::default();
    let home = std::env::current_dir().unwrap();
    let root_path = home.join("test-dir");
    let target_dir = Path::new("output");
    let mut bumblebee = Bumblebee::new(&root_path, target_dir, &mut allocator);

    eval_dir(&mut bumblebee)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_test() {
        let mut allocator = Allocator::default();
        let home = std::env::current_dir().unwrap();
        let root_path = home.join("test-dir");
        let target_dir = Path::new("output");
        let mut bumblebee = Bumblebee::new(&root_path, target_dir, &mut allocator);
        assert!(eval_dir(&mut bumblebee).is_ok());
    }
}
