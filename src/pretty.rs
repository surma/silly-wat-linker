use crate::ast::{Item, Node};

pub fn pretty_print(node: &Node) -> String {
    let mut buffer: String = String::new();
    pretty_print_internal(node, 0, &mut buffer);
    buffer
}

fn pretty_print_internal(node: &Node, mut level: usize, buffer: &mut String) {
    *buffer += &format!("\n{}({}", "  ".repeat(level), node.name);
    level += 1;
    for item in node.items.iter() {
        match item {
            Item::Attribute(attr) => {
                *buffer += &format!("\n{}{}", "  ".repeat(level), attr);
            }
            Item::Node(node) => {
                if node.name.as_str().ends_with(".const") {
                    *buffer += &format!("\n{}{}", "  ".repeat(level), node)
                } else {
                    match node.name.as_ref() {
                        "param" | "global.get" | "local.get" | "start" | "result" | "local"
                        | "export" | "table" | "memory" | "import" | "global" => {
                            *buffer += &format!("\n{}{}", "  ".repeat(level), node)
                        }
                        _ => pretty_print_internal(node, level, buffer),
                    };
                }
            }
            Item::Nothing => {}
        }
    }
    level -= 1;
    *buffer += ")";
    if node.name == "func" {
        *buffer += "\n";
    }
}
