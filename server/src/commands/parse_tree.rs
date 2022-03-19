use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{format_err, Result};
use serde_json::Value;
use slog_scope::warn;
use tree_sitter::{Parser, TreeCursor};

use crate::url_norm::FromJson;

use super::Invokeable;

pub struct TreeSitterSExpr {
    pub tree_sitter: Rc<RefCell<Parser>>,
}

impl Invokeable for TreeSitterSExpr {
    fn run_command(&self, _: &Path, arguments: &[Value]) -> Result<Value> {
        let path = PathBuf::from_json(arguments.get(0).unwrap())?;

        warn!("parsing"; "path" => path.to_str().unwrap().to_string());

        let source = fs::read_to_string(path)?;

        let tree = match self.tree_sitter.borrow_mut().parse(source, None) {
            Some(tree) => tree,
            None => return Err(format_err!("tree-sitter parsing resulted in no parse tree")),
        };

        let mut cursor = tree.walk();

        let rendered = render_parse_tree(&mut cursor);

        Ok(serde_json::value::Value::String(rendered))
    }
}

fn render_parse_tree(cursor: &mut TreeCursor) -> String {
    let mut string = String::new();

    let mut indent = 0;
    let mut visited_children = false;

    loop {
        let node = cursor.node();

        let display_name = if node.is_missing() {
            format!("MISSING {}", node.kind())
        } else if node.is_named() {
            node.kind().to_string()
        } else {
            "".to_string()
        };

        if visited_children {
            if cursor.goto_next_sibling() {
                visited_children = false;
            } else if cursor.goto_parent() {
                visited_children = true;
                indent -= 1;
            } else {
                break;
            }
        } else {
            if !display_name.is_empty() {
                let start = node.start_position();
                let end = node.end_position();

                let field_name = match cursor.field_name() {
                    Some(name) => name.to_string() + ": ",
                    None => "".to_string(),
                };

                string += ("  ".repeat(indent)
                    + format!("{}{} [{}, {}] - [{}, {}]\n", field_name, display_name, start.row, start.column, end.row, end.column)
                        .trim_start())
                .as_str();
            }

            if cursor.goto_first_child() {
                visited_children = false;
                indent += 1;
            } else {
                visited_children = true;
            }
        }
    }

    string
}
