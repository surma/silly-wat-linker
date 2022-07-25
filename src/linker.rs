use std::collections::HashMap;

use crate::ast::Node;
use crate::features::Feature;
use crate::loader::{FileSystemLoader, LoadRecord, Loader};
use crate::parser;
use crate::Result;

pub struct Linker {
    loader: Box<dyn Loader>,
    pub(crate) loaded_modules: HashMap<String, String>,
    pub features: Vec<Feature>,
}

impl Linker {
    pub fn new(loader: Box<dyn Loader>) -> Linker {
        Linker {
            loader,
            loaded_modules: HashMap::new(),
            features: vec![],
        }
    }

    pub fn link_raw<T: AsRef<str>>(&mut self, content: T) -> Result<Node> {
        let module = parser::Parser::new(content).parse()?;
        self.link_module(module)
    }

    pub fn link_file(&mut self, path: &str) -> Result<Node> {
        let module = self.load_module(path)?.contents;
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
    fn load_raw(&mut self, path: &str) -> Result<LoadRecord<String>> {
        self.loader.load_raw(path)
    }

    // Linker dedupes by returning an empty module when a module is loaded the second time.
    // FIXME: This is not a great way to dedupe.
    fn load_module(&mut self, path: &str) -> Result<LoadRecord<Node>> {
        let lr = self.loaded_modules.get(path).cloned() ;
        println!("Cache hit for {}? {:?}", path, lr);

        let lr = match lr {
            Some(canonical_path) => LoadRecord {
                contents: "(module)".to_string(),
                canonical_path
            },
            None => {
                let lr = self.loader.load_raw(path)?;
                self.loaded_modules
                    .insert(lr.canonical_path.clone(), lr.canonical_path.clone());
                lr
            }
        };
        println!("> {:?}", lr);
        Ok(LoadRecord {
            canonical_path: lr.canonical_path.clone(),
            contents: parser::Parser::new(lr.contents).parse()?
        })
    }
}
