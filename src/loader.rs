use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::Node;
use crate::parser::Parser;
use crate::Result;

#[derive(Debug, Clone)]
pub struct LoadRecord {
    pub module: Node,
    pub canonical_path: String,
}

pub trait Loader {
    fn load(&mut self, path: &str) -> Result<LoadRecord>;
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
    fn load(&mut self, path: &str) -> Result<LoadRecord> {
        let file_path = self.root.join(path);

        let contents = fs::read_to_string(&file_path).map_err(|err| format!("{}", err))?;
        let module = Parser::new(contents).parse()?;
        Ok(LoadRecord {
            module,
            // FIXME: Better non-utf8 handling
            canonical_path: file_path.to_str().unwrap().to_string(),
        })
    }
}

pub struct MockLoader {
    pub map: HashMap<String, String>,
}

impl Loader for MockLoader {
    fn load(&mut self, path: &str) -> Result<LoadRecord> {
        let contents = self.map.get(path).ok_or(format!("Unknown file {}", path))?;
        let module = Parser::new(contents).parse()?;
        Ok(LoadRecord {
            module,
            // FIXME: Better non-utf8 handling
            canonical_path: path.to_string(),
        })
    }
}
