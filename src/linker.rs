use std::collections::HashSet;

use crate::ast::Node;
use crate::features::Feature;
use crate::loader::{FileSystemLoader, Loader};
use crate::parser;
use crate::Result;

pub struct Linker {
    loader: Box<dyn Loader>,
    pub(crate) loaded_modules: HashSet<String>,
    pub features: Vec<Feature>,
}

impl Linker {
    pub fn new(loader: Box<dyn Loader>) -> Linker {
        Linker {
            loader,
            loaded_modules: HashSet::new(),
            features: vec![],
        }
    }

    pub fn link_raw<T: AsRef<str>>(&mut self, content: T) -> Result<Node> {
        let module = parser::Parser::new(content).parse()?;
        self.link_module(module)
    }

    pub fn link_file(&mut self, path: &str) -> Result<Node> {
        let module = self.load_module(path)?;
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
    fn canonicalize(&mut self, path: &str) -> Result<String> {
        self.loader.canonicalize(path)
    }

    fn load_raw(&mut self, path: &str) -> Result<Vec<u8>> {
        self.loader.load_raw(path)
    }

    // Linker dedupes by returning an empty module when a module is loaded the second time.
    // FIXME: This is not a great way to dedupe.
    fn load_module(&mut self, path: &str) -> Result<Node> {
        let canonical_path = self.canonicalize(path)?;

        let contents = if self.loaded_modules.contains(&canonical_path) {
            "(module)".to_string().into_bytes()
        } else {
            let contents = self.loader.load_raw(path)?;
            self.loaded_modules.insert(canonical_path.clone());
            contents
        };

        let contents = String::from_utf8(contents).map_err(|err| format!("{}", err))?;
        let module = parser::Parser::new(contents).parse()?;
        Ok(module)
    }
}
