use std::{fmt::Display, marker::PhantomData};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: String,
    pub depth: usize,
    pub items: Vec<Item>,
}

pub trait Visitor {
    fn visit_node(&mut self, _node: &mut Node) {}
    fn visit_attribute(&mut self, _attr: &mut String) {}
}

pub struct Walker<'a> {
    stack: Vec<*mut Node>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Iterator for Walker<'a> {
    type Item = &'a mut Node;
    fn next(&mut self) -> Option<Self::Item> {
        match self.stack.pop() {
            Some(node_ptr) => {
                let node = unsafe { &mut *node_ptr };
                for node in node.immediate_node_iter_mut().rev() {
                    self.stack.push(node as *mut Node);
                }
                Some(node)
            }
            _ => None,
        }
    }
}

impl Node {
    pub fn walk_mut(&mut self, visitor: &mut impl Visitor) {
        visitor.visit_node(self);
        for item in &mut self.items {
            match item {
                Item::Attribute(attr) => visitor.visit_attribute(attr),
                Item::Node(node) => node.walk_mut(visitor),
                Item::Nothing => {}
            };
        }
    }

    /// Returns an iterator that iterates over immediate children that are nodes.
    pub fn immediate_node_iter(&self) -> impl DoubleEndedIterator<Item = &Node> {
        self.items.iter().flat_map(|node| node.as_node())
    }

    /// Returns an iterator that iterates over immediate children that are nodes.
    pub fn immediate_node_iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Node> {
        self.items.iter_mut().flat_map(|node| node.as_node_mut())
    }

    /// Returns an iterator that iterates over immediate children that are attributes.
    pub fn immediate_attribute_iter(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.items.iter().flat_map(|node| node.as_attribute())
    }

    /// Returns an iterator that iterates over immediate children that are attributes.
    pub fn immediate_attribute_iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut String> {
        self.items
            .iter_mut()
            .flat_map(|node| node.as_attribute_mut())
    }

    /// Returns an iterator that iterates over all nodes in the tree.
    pub fn node_iter_mut<'a>(&'a mut self) -> Walker<'a> {
        Walker {
            stack: vec![self as *mut Node],
            _lifetime: Default::default(),
        }
    }

    /// Returns an iterator that iterates over all nodes in the tree.
    pub fn node_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Node> + 'a> {
        let parent_it = [self].into_iter();
        let item_it = self
            .items
            .iter()
            .flat_map(|item| item.as_node())
            .map(|node| node.node_iter())
            .flatten();

        Box::new(parent_it.chain(item_it))
    }

    /// Appends a new node to the parent node. Node is assumed to be well-formed, i.e. all `depth` values must be set correctly.
    pub fn append_node(&mut self, mut node: Node) {
        node.node_iter_mut().for_each(|node| {
            node.depth += self.depth;
        });
        self.items.push(Item::Node(node));
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
                .filter(|&item| !item.is_nothing())
                .map(|item| format!("{}", item))
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Nothing,
    Attribute(String),
    Node(Node),
}

impl Item {
    /// Returns a node only if the item is a node.
    pub fn as_node(&self) -> Option<&Node> {
        match self {
            Item::Node(node) => Some(node),
            _ => None,
        }
    }

    /// Returns a node only if the item is a node.
    pub fn as_node_mut(&mut self) -> Option<&mut Node> {
        match self {
            Item::Node(node) => Some(node),
            _ => None,
        }
    }

    /// Returns the item as a node. Panics if it’s not a node.
    pub fn into_node(self) -> Node {
        match self {
            Item::Node(node) => node,
            _ => panic!(),
        }
    }

    /// Returns true if the item is nothing.
    pub fn is_nothing(&self) -> bool {
        match self {
            Item::Nothing => true,
            _ => false,
        }
    }

    /// Returns a string only if the item is an attribute.
    pub fn as_attribute(&self) -> Option<&str> {
        match self {
            Item::Attribute(attribute) => Some(attribute),
            _ => None,
        }
    }

    /// Returns a string only if the item is an attribute.
    pub fn as_attribute_mut(&mut self) -> Option<&mut String> {
        match self {
            Item::Attribute(attribute) => Some(attribute),
            _ => None,
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Attribute(str) => write!(f, "{}", str),
            Item::Node(node) => write!(f, "{}", node),
            Item::Nothing => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::Parser;
    #[test]
    fn node_iter() {
        let table = [(
            r#"
                (module
                    (func (a))
                    (func (b) (c))
                    (func))
            "#,
            &["module", "func", "a", "func", "b", "c", "func"],
        )];
        for (input, expected) in table {
            let mut parser = Parser::new(input);
            let ast = parser.parse().unwrap();
            let nodes: Vec<String> = ast.node_iter().map(|node| node.name.clone()).collect();
            assert_eq!(&nodes, expected)
        }
    }

    #[test]
    fn node_iter_mut() {
        let input = r#"
            (module $u
                (func $v)
                (func (b $w) $x (c $y))
                (func $z))
        "#;
        let expected = r#"(module $u0 (func $v0) (func (b $w0) $x0 (c $y0)) (func $z0))"#;
        let mut parser = Parser::new(input);
        let mut ast = parser.parse().unwrap();
        for node in ast.node_iter_mut() {
            for attr in node.immediate_attribute_iter_mut() {
                *attr += "0";
            }
        }
        assert_eq!(&format!("{}", ast), expected)
    }
}
