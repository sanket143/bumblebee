use std::{rc::Rc, sync::Arc};

use oxc::allocator::Allocator;
use walkdir::WalkDir;

use crate::document::Document;

pub struct Queso<'a> {
    documents: Vec<Document<'a>>,
}

impl<'a> Queso<'a> {
    pub fn new(folder_name: &'a str, allocator: &'a Allocator) -> Self {
        let mut paths: Vec<Document> = Vec::new();
        for path in WalkDir::new(folder_name)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
        {
            let content = std::fs::read_to_string(path.path()).unwrap();
            let document = Document::new(path.path().to_str().unwrap().into(), content, allocator);

            paths.push(document);
        }

        println!("{}", paths.len());

        Self {
            documents: Vec::new(),
        }
    }
}
