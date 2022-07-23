use std::collections::HashMap;

use crate::ast::Node;
use crate::loader::{FileSystemLoader, LoadRecord, Loader};
use crate::passes::Pass;
use crate::Result;

pub struct Linker {
    loader: Box<dyn Loader>,
    pub(crate) loaded_files: HashMap<String, LoadRecord>,
    pub passes: Vec<Pass>,
}

impl Linker {
    pub fn new(loader: Box<dyn Loader>) -> Linker {
        Linker {
            loader,
            loaded_files: HashMap::new(),
            passes: vec![],
        }
    }

    pub fn link(&mut self, path: &str) -> Result<Node> {
        let mut module = self.load(path)?.module;

        for pass in self.passes.clone() {
            pass(&mut module, self)?;
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