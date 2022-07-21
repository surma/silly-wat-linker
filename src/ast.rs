use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub depth: usize,
    pub items: Vec<Item>,
}

pub struct Walker<'a> {
    node: &'a Node,
    i: Option<usize>,
}

impl<'a> Iterator for Walker<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        match self.i {
            None => {
                self.i = Some(0);
                Some(self.node)
            }
            Some(i) => {
                let maybeItem = self
                    .node
                    .items
                    .iter()
                    .enumerate()
                    .skip(i)
                    .find(|&(idx, item)| item.as_node().is_some());
                if let Some((idx, _)) = maybeItem {
                    self.i = Some(idx + 1);
                }
                maybeItem
                    .map(|(idx, item)| item)
                    .and_then(|item| item.as_node())
            }
        }
    }
}

impl<'a> IntoIterator for &'a Node {
    type IntoIter = Walker<'a>;
    type Item = &'a Node;

    fn into_iter(self) -> Self::IntoIter {
        Walker {
            node: self,
            i: None,
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}{}{})",
            self.name,
            if self.items.len() > 0 { " " } else { "" },
            self.items
                .iter()
                .map(|item| format!("{}", item))
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    Attribute(String),
    Node(Node),
}

impl Item {
    fn as_node(&self) -> Option<&Node> {
        match self {
            Item::Node(node) => Some(node),
            _ => None,
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Attribute(str) => write!(f, "{}", str),
            Item::Node(node) => write!(f, "{}", node),
        }
    }
}
#[cfg(test)]
mod test {
    use crate::parser::Parser;
    #[test]
    fn table_test() {
        let table = [(
            r#"
									(module
										(func $1)
										(func $2)
										(func $3))
								"#,
            &["module", "func", "func", "func"],
        )];
        for (input, expected) in table {
            let mut parser = Parser::new(input);
            let ast = parser.parse().unwrap();
            let nodes: Vec<String> = ast.into_iter().map(|node| node.name.clone()).collect();
            assert_eq!(&nodes, expected)
        }
    }
}
