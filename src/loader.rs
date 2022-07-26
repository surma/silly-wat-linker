use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::Node;
use crate::error::{Result, SWLError};
use crate::parser::Parser;

pub trait Loader {
    fn canonicalize(&mut self, path: &str) -> Result<String>;
    fn load_raw(&mut self, path: &str) -> Result<Vec<u8>>;
    fn load_module(&mut self, path: &str) -> Result<Node> {
        let contents = self.load_raw(path)?;
        let contents = String::from_utf8(contents).map_err(|err| SWLError::Other(err.into()))?;
        let module = Parser::new(contents).parse()?;
        Ok(module)
    }
}

pub struct FileSystemLoader {
    root: PathBuf,
}

impl FileSystemLoader {
    pub fn new<T: AsRef<Path>>(root: T) -> FileSystemLoader {
        FileSystemLoader {
            root: root.as_ref().to_path_buf(),
        }
    }
}

impl Loader for FileSystemLoader {
    fn canonicalize(&mut self, path: &str) -> Result<String> {
        let file_path = self.root.join(path);
        Ok(file_path.to_str().unwrap().to_string())
    }

    fn load_raw(&mut self, path: &str) -> Result<Vec<u8>> {
        let canonical_path = self.canonicalize(path)?;
        let contents = fs::read(&canonical_path).map_err(|err| SWLError::Other(err.into()))?;
        Ok(contents)
    }
}

pub struct MockLoader {
    pub map: HashMap<String, Vec<u8>>,
}

impl Loader for MockLoader {
    fn canonicalize(&mut self, path: &str) -> Result<String> {
        Ok(path.to_string())
    }

    fn load_raw(&mut self, path: &str) -> Result<Vec<u8>> {
        let contents = self
            .map
            .get(path)
            .ok_or(SWLError::Simple(format!("Unknown file {}", path)))?
            .clone();
        Ok(contents)
    }
}
