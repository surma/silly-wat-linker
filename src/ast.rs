use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub items: Vec<Item>,
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

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Attribute(str) => write!(f, "{}", str),
            Item::Node(node) => write!(f, "{}", node),
        }
    }
}
