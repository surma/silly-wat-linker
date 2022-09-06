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

fn pretty_print_func_params(node: &Node, level: usize, buffer: &mut String) {
    assert!(node.name == "func");
    let mut it = node.items.iter().peekable();

    // Print function name if any
    if it
        .peek()
        .and_then(|v| v.as_attribute())
        .map(|v| v.starts_with("$"))
        .unwrap_or(false)
    {
        *buffer += " ";
        *buffer += it.next().unwrap().as_attribute().unwrap();
    }
    // Print function header
    while it
        .peek()
        .map(|item| is_function_header_item(item))
        .unwrap_or(false)
    {
        pretty_print_item(it.next().unwrap(), level, buffer)
    }
    if it.peek().is_some() {
        *buffer += "\n";
    }
    // Print locals
    while it
        .peek()
        .and_then(|item| item.as_node())
        .map(|node| node.name == "local")
        .unwrap_or(false)
    {
        pretty_print_item(it.next().unwrap(), level, buffer);
    }
    if it.peek().is_some() {
        *buffer += "\n";
    }
    // Print body
    for item in it {
        pretty_print_item(item, level, buffer);
    }
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

fn pretty_print_node_internal(node: &Node, level: usize, buffer: &mut String) {
    if is_single_line_node_type(node) || has_at_most_one_simple_attribute(node) {
        *buffer += &format!("\n{}{}", "\t".repeat(level), node);
        return;
    } else if node.name == "func" {
        // Extra new-line before a function
        *buffer += &format!("\n\n{}({}", "\t".repeat(level), node.name);
        pretty_print_func_params(node, level + 1, buffer);
    } else if node.name == "call" {
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
    } else {
        *buffer += &format!("\n{}({}", "\t".repeat(level), node.name);
        for item in node.items.iter() {
            pretty_print_item(item, level + 1, buffer);
        }
    }
    *buffer += ")";
}
