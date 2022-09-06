use std::iter::Peekable;

use crate::ast::{Item, Node};

pub fn pretty_print(node: &Node) -> String {
    let mut buffer: String = String::new();
    pretty_print_node_internal(node, 0, &mut buffer);
    buffer
}

pub fn has_at_most_one_simple_attribute(node: &Node) -> bool {
    node.items.len() <= 1
        && node
            .items
            .get(0)
            .map(|item| item.as_attribute().is_some())
            .unwrap_or(true)
}

pub fn is_single_line_node_type(node: &Node) -> bool {
    match node.name.as_ref() {
        "param" | "local" | "export" | "table" | "memory" | "import" | "global" => true,
        _ => false,
    }
}

fn is_function_header_item(item: &Item) -> bool {
    match item {
        Item::Attribute(attr) => attr.starts_with("$"),
        Item::Node(node) => match node.name.as_ref() {
            "param" | "result" | "export" => true,
            _ => false,
        },
        _ => true,
    }
}

fn item_has_node_name(v: Option<&&Item>, name: &str) -> bool {
    v.and_then(|v| v.as_node())
        .map(|v| v.name == name)
        .unwrap_or(false)
}

fn item_attribute_matches_predicate<F>(v: Option<&&Item>, pred: F) -> bool
where
    F: Fn(&str) -> bool,
{
    v.and_then(|v| v.as_attribute())
        .map(|v| pred(v))
        .unwrap_or(false)
}

fn item_matches_predicate<F>(v: Option<&&Item>, pred: F) -> bool
where
    F: Fn(&Item) -> bool,
{
    v.map(|v| pred(v)).unwrap_or(false)
}

fn pretty_print_func(node: &Node, level: usize, buffer: &mut String) {
    assert!(node.name == "func");
    *buffer += &format!("\n\n{}({}", "\t".repeat(level), node.name);
    let mut it = node.items.iter().peekable();

    // Print function name and export if any
    if item_attribute_matches_predicate(it.peek(), |v| v.starts_with("$")) {
        *buffer += " ";
        *buffer += it.next().unwrap().as_attribute().unwrap();
    }

    if item_has_node_name(it.peek(), "export") {
        *buffer += &format!(" {}", it.next().unwrap());
    }

    // Print function header
    while item_matches_predicate(it.peek(), |v| is_function_header_item(v)) {
        pretty_print_item(it.next().unwrap(), level + 1, buffer)
    }

    // Print locals
    if item_has_node_name(it.peek(), "local") {
        *buffer += "\n";
        while item_has_node_name(it.peek(), "local") {
            pretty_print_item(it.next().unwrap(), level + 1, buffer);
        }
    }

    // Print body
    if it.peek().is_some() {
        *buffer += "\n";
        for item in it {
            pretty_print_item(item, level + 1, buffer);
        }
    }
    *buffer += ")"
}

fn pretty_print_item(item: &Item, level: usize, buffer: &mut String) {
    match item {
        Item::Attribute(attr) => {
            *buffer += &format!("\n{}{}", "\t".repeat(level), attr);
        }
        Item::Node(node) => {
            pretty_print_node_internal(node, level, buffer);
        }
        Item::Nothing => {}
    }
}

fn pretty_print_call(node: &Node, level: usize, buffer: &mut String) {
    assert!(node.name == "call");
    *buffer += &format!("\n{}({}", "\t".repeat(level), node.name);
    *buffer += &format!(
        " {}",
        node.items
            .get(0)
            .and_then(|item| item.as_attribute())
            .unwrap_or("")
    );
    for item in node.items.iter().skip(1) {
        pretty_print_item(item, level + 1, buffer);
    }
    *buffer += ")";
}
fn pretty_print_as_single_line(node: &Node, level: usize, buffer: &mut String) {
    *buffer += &format!("\n{}{}", "\t".repeat(level), node);
}

fn pretty_print_node_internal(node: &Node, level: usize, buffer: &mut String) {
    if is_single_line_node_type(node) || has_at_most_one_simple_attribute(node) {
        pretty_print_as_single_line(node, level, buffer);
    } else if node.name == "func" {
        pretty_print_func(node, level, buffer);
    } else if node.name == "call" {
        pretty_print_call(node, level, buffer);
    } else {
        *buffer += &format!("\n{}({}", "\t".repeat(level), node.name);
        for item in node.items.iter() {
            pretty_print_item(item, level + 1, buffer);
        }
        *buffer += ")";
    }
}
