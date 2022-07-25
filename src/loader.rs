use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::Node;
use crate::parser::Parser;
use crate::Result;

#[derive(Debug, Clone)]
pub struct LoadRecord<T> {
    pub contents: T,
    pub canonical_path: String,
}

pub trait Loader {
    fn load_raw(&mut self, path: &str) -> Result<LoadRecord<Vec<u8>>>;
    fn load_module(&mut self, path: &str) -> Result<LoadRecord<Node>> {
        let record = self.load_raw(path)?;
        let contents = String::from_utf8(record.contents).map_err(|err| format!("{}", err))?;
        let module = Parser::new(contents).parse()?;
        Ok(LoadRecord {
            canonical_path: record.canonical_path,
            contents: module,
        })
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
    fn load_raw(&mut self, path: &str) -> Result<LoadRecord<Vec<u8>>> {
        let file_path = self.root.join(path);
        let contents = fs::read(&file_path).map_err(|err| format!("{}", err))?;
        Ok(LoadRecord {
            // FIXME: Better non-utf8 handling
            canonical_path: file_path.to_str().unwrap().to_string(),
            contents,
        })
    }
}

pub struct MockLoader {
    pub map: HashMap<String, Vec<u8>>,
}

impl Loader for MockLoader {
    fn load_raw(&mut self, path: &str) -> Result<LoadRecord<Vec<u8>>> {
        let contents = self
            .map
            .get(path)
            .ok_or(format!("Unknown file {}", path))?
            .clone();
        Ok(LoadRecord {
            // FIXME: Better non-utf8 handling
            canonical_path: path.to_string(),
            contents,
        })
    }
}
