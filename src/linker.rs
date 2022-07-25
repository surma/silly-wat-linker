use std::collections::HashMap;

use crate::ast::Node;
use crate::features::Feature;
use crate::loader::{FileSystemLoader, LoadRecord, Loader};
use crate::parser;
use crate::Result;

pub struct Linker {
    loader: Box<dyn Loader>,
    pub(crate) loaded_files: HashMap<String, LoadRecord>,
    pub features: Vec<Feature>,
}

impl Linker {
    pub fn new(loader: Box<dyn Loader>) -> Linker {
        Linker {
            loader,
            loaded_files: HashMap::new(),
            features: vec![],
        }
    }

    pub fn link_raw<T: AsRef<str>>(&mut self, content: T) -> Result<Node> {
        let module = parser::Parser::new(content).parse()?;
        self.link_module(module)
    }

    pub fn link_file(&mut self, path: &str) -> Result<Node> {
        let module = self.load(path)?.module;
        self.link_module(module)
    }

    pub fn link_module(&mut self, mut module: Node) -> Result<Node> {
        for feature in self.features.clone() {
            feature(&mut module, self)?;
        }
        Ok(module)
    }
}

impl Default for Linker {
    fn default() -> Self {
        Linker::new(Box::new(FileSystemLoader::new(
            std::env::current_dir().unwrap(),
        )))
    }
}

impl Loader for Linker {
    fn load(&mut self, path: &str) -> Result<LoadRecord> {
        let lr = match self.loaded_files.get(path) {
            Some(lr) => LoadRecord {
                // FIXME: This is not a great way to dedupe importing the same file.
                module: Node {
                    name: "module".to_string(),
                    items: vec![],
                    depth: lr.module.depth,
                },
                ..lr.clone()
            },
            None => {
                let lr = self.loader.load(path)?;
                self.loaded_files
                    .insert(lr.canonical_path.clone(), lr.clone());
                lr
            }
        };
        Ok(lr)
    }
}
