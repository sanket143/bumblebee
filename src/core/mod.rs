use crate::query::Query;
use crate::service::Service;
use crate::service::ServiceReference;
use anyhow::Result;
use dunce::realpath;
use ignore::Walk;
use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{NodeId, SemanticBuilder, SemanticBuilderReturn};
use oxc_span::{GetSpan, SourceType};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    mem::ManuallyDrop,
    path::Path,
};
use std::{ffi::OsStr, path::PathBuf};

/// The main Bumblebee struct that handles code analysis
pub struct Bumblebee<'a> {
    root_path: &'a Path,
    target_dir: &'a Path,
    allocator: &'a Allocator,
    queries: HashSet<Query>,
    services: HashMap<PathBuf, &'a mut ServiceReference<'a>>,
}

impl<'a> Bumblebee<'a> {
    /// Creates a new Bumblebee instance
    pub fn new(root_path: &'a Path, target_dir: &'a Path, allocator: &'a mut Allocator) -> Self {
        let path_buf = realpath(root_path).expect("Invalid project path");
        let path = &**allocator.alloc(ManuallyDrop::new(path_buf));

        Self {
            root_path: path.as_path(),
            target_dir,
            allocator,
            queries: Default::default(),
            services: Default::default(),
        }
    }

    /// Evaluates a query to find references to a symbol
    pub fn evaluate_query(&mut self, query: Query) {
        let source_path =
            realpath(self.root_path.join(query.symbol_path())).expect("Invalid query source path!");
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
        let reference_node_ids = &mut **self.allocator.alloc(ManuallyDrop::new(HashSet::new()));
        let reference_symbol_ids = &mut **self.allocator.alloc(ManuallyDrop::new(HashSet::new()));
        let symbol_id = service.get_symbol_id(query.symbol());

        if let Some(symbol_id) = symbol_id {
            let symbol_name = service.semantic().scoping().symbol_name(symbol_id);

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
                    reference_node_ids,
                    reference_symbol_ids,
                )));

        self.services.insert(source_path.clone(), service_reference);
    }

    /// Updates the services by scanning the root directory for JavaScript files
    pub fn update_services(&mut self) -> Result<()> {
        for entry in Walk::new(self.root_path).flatten() {
            if entry.path().extension() == Some(OsStr::new("js")) {
                let reference_node_ids =
                    &mut **self.allocator.alloc(ManuallyDrop::new(HashSet::new()));
                let reference_symbol_ids =
                    &mut **self.allocator.alloc(ManuallyDrop::new(HashSet::new()));
                let source_path =
                    realpath(self.root_path.join(entry.path())).expect("Invalid source path!");

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
                                reference_node_ids,
                                reference_symbol_ids,
                            )));

                    self.services
                        .insert(source_path.to_owned(), service_reference);
                }
            }
        }

        Ok(())
    }

    /// Recursively finds all references to the queried symbols
    pub fn find_references_recursively(&mut self) {
        let mut queries: Vec<Query> = self.queries.iter().cloned().collect();
        let queries_original_len = queries.len();
        let mut queries_len = queries_original_len;
        let mut i = 0;

        // Using a while loop instead of iterator to handle the dynamic growth of queries
        while i < queries_len {
            self.services
                .iter_mut()
                .map(|(source_path, service_reference)| {
                    let scoping = service_reference.service().semantic().scoping();
                    (source_path, service_reference, scoping)
                })
                .for_each(|(source_path, service_reference, scoping)| {
                    let query = &queries[i];
                    service_reference.find_references(query);

                    let symbol_ids = service_reference.reference_symbol_ids();

                    queries.extend(symbol_ids.iter().filter_map(|symbol_id| {
                        let symbol_name = scoping.symbol_name(*symbol_id);
                        let query = Query::new(
                            *symbol_id,
                            symbol_name.to_owned(),
                            source_path.to_path_buf(),
                        );

                        if !self.queries.contains(&query) {
                            return Some(query);
                        }

                        None
                    }));
                    queries_len = queries.len();
                    println!("IN: {}", queries.len());
                });

            if i >= queries_original_len {
                self.queries.insert(queries[i].clone());
            }

            i += 1;
        }
    }

    /// Dumps all found references to files in the target directory
    pub fn dump_reference_files(&'a self) {
        std::fs::create_dir_all(self.target_dir).ok();

        self.services
            .iter()
            .for_each(|(source_path, service_reference)| {
                println!(
                    "{}: {:?}",
                    source_path.display(),
                    service_reference.reference_node_ids()
                );
                if !service_reference.reference_node_ids().is_empty() {
                    let mut reference_node_ids: Vec<NodeId> = service_reference
                        .reference_node_ids()
                        .iter()
                        .copied()
                        .collect();
                    reference_node_ids.sort_unstable();

                    let relative_path = source_path.strip_prefix(self.root_path).unwrap();
                    let target_path = self.target_dir.join(relative_path);

                    std::fs::create_dir_all(target_path.parent().unwrap()).ok();
                    let mut file_stream = File::create(&target_path).unwrap();

                    reference_node_ids.iter().for_each(|node_id| {
                        let node = service_reference
                            .service()
                            .semantic()
                            .nodes()
                            .get_node(*node_id);
                        let span = node.span();
                        let text = service_reference
                            .service()
                            .semantic()
                            .source_text()
                            .get((span.start as usize)..(span.end as usize))
                            .unwrap();

                        file_stream
                            .write_all((text.to_owned() + "\n\n").as_bytes())
                            .ok();
                    });
                }
            });
    }
}
