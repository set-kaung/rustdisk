use crate::tree::node::Node;

pub struct InfoOptions {
    pub depth_level: u16,
    pub shorten: bool,
    pub max_len: u16,
    pub dir_only: bool,
    pub show_percent_only: bool,
    pub show_size_only: bool,
    pub prefix_filters: Vec<String>,
    pub sort_by_size: bool,
}

pub fn print_entries(node: &mut Node, options: InfoOptions) {
    let mut stack: Vec<Node> = Vec::new();
    if options.sort_by_size {
        node.children.sort_by(|a, b| a.size.cmp(&b.size).reverse());
        node.children.reverse();
    }
    stack.append(&mut node.children);

    while let Some(mut child) = stack.pop() {
        if child.depth <= options.depth_level {
            let mut p = child.path.to_str().unwrap().to_string();
            if options.shorten {
                p = shorten_name(p, options.max_len);
            }
            println!("{} {}", child.size, p);
        }

        if options.sort_by_size {
            child.children.sort_by(|a, b| a.size.cmp(&b.size).reverse());
            child.children.reverse();
        }
        stack.append(&mut child.children);
    }
    let p = node.path.to_str().unwrap().to_string();
    println!("{} {}", node.size, p);
}

fn shorten_name(name: String, max_len: u16) -> String {
    let max = max_len as usize;
    let chars: Vec<char> = name.chars().collect();
    if chars.len() <= max {
        return name;
    }
    let half = (max - 1) / 2;
    let left: String = chars[..half].iter().collect();
    let right: String = chars[chars.len() - (max - half - 1)..].iter().collect();
    format!("{}…{}", left, right)
}
