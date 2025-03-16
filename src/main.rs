mod query;
mod service;

use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{NodeId, SemanticBuilder, SemanticBuilderReturn};
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

pub fn eval_dir(root_path: &Path) -> Result<()> {
    // first update the queries to get declaration and add their symbolId? or maybe nodeId?
    // that'll require a service which parses and ...
    // maybe I can have 2 lists, one just to keep track if we've visisted or not
    // and the other to actually iterate and evaluate in the service
    //
    // Allocator has been a mystery to me
    // What are we even trying to achieve here?
    let mut allocator = Allocator::default();
    let mut queries = [Query::new_with_symbol(
        "call".into(),
        PathBuf::from("./factory.js"),
    )];
    let mut query_set = HashSet::new();
    let mut services = HashMap::new();
    let target_dir = Path::new("output");

    for query in queries.iter_mut() {
        let source_path = root_path.join(query.symbol_path()).canonicalize().unwrap();
        let source_text = std::fs::read_to_string(&source_path).unwrap();
        let source_type = SourceType::from_path(&source_path).unwrap();
        let source_text_ref = allocator.alloc_str(&source_text);

        let ParserReturn { program, .. } = &**allocator.alloc(ManuallyDrop::new(
            Parser::new(&allocator, source_text_ref, source_type).parse(),
        ));

        let SemanticBuilderReturn { semantic, .. } = SemanticBuilder::new().build(program);
        let service = Service::build(root_path.into(), source_path.to_owned(), semantic).unwrap();
        let symbol_id = service.get_symbol_id(query.symbol());

        if let Some(symbol_id) = symbol_id {
            query.udpate_symbol_id(symbol_id);
        }

        query_set.insert(query);
        services.insert(source_path.clone(), (service, HashSet::new()));
    }

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
                    let node = service.get_semantic().nodes().get_node(*node_id);
                    let span = node.span();
                    let text = service
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
