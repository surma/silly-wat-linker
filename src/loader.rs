use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::Node;
use crate::parser::Parser;
use crate::Result;

pub trait Loader {
    fn load(&mut self, path: &str) -> Result<Option<Node>>;
}

pub struct FileSystemLoader {
    root: PathBuf,
    loaded_files: HashSet<PathBuf>,
}

impl FileSystemLoader {
    pub fn new<T: AsRef<Path>>(root: T) -> FileSystemLoader {
        FileSystemLoader {
            root: root.as_ref().to_path_buf(),
            loaded_files: HashSet::new(),
        }
    }
}

impl Loader for FileSystemLoader {
    fn load(&mut self, path: &str) -> Result<Option<Node>> {
        let file_path = self.root.join(path);
        if self.loaded_files.contains(&file_path) {
            return Ok(None);
        }

        let contents = fs::read_to_string(&file_path).map_err(|err| format!("{}", err))?;
        let ast = Parser::new(contents).parse()?;
        self.loaded_files.insert(file_path);
        Ok(Some(ast))
    }
}

pub struct MockLoader {
    pub map: HashMap<String, String>,
}

impl Loader for MockLoader {
    fn load(&mut self, path: &str) -> Result<Option<Node>> {
        let contents = self.map.get(path).ok_or(format!("Unknown file {}", path))?;
        let ast = Parser::new(contents).parse()?;
        Ok(Some(ast))
    }
}
