use std::fs::read_to_string;

use anyhow::Result;
use rust_lsp::lsp_types::{Location, Position, Range};
use slog_scope::{info, debug};
use tree_sitter::{Node, Parser, Point, Tree, Query, QueryCursor};
use url::Url;

macro_rules! find_function_str {
    () => { 
        r#"
            (
                (function_declarator 
                    (identifier) @function)
                    (#match? @function "{}")
            )
        "#
    };
}
pub struct ParserContext<'a> {
    source: String,
    tree: Tree,
    parser: &'a mut Parser,
}

impl<'a> ParserContext<'a> {
    pub fn new(parser: &'a mut Parser, document_uri: &Url) -> Result<Self> {
        let source = read_to_string(document_uri.path())?;

        let tree = parser.parse(&source, None).unwrap();

        Ok(ParserContext { source, tree, parser })
    }

    pub fn find_definitions(&self, document_uri: &Url, point: Position) -> Result<Vec<Location>> {
        let current_node = match self.find_node_at_point(point) {
            Some(node) => node,
            None => return Ok(vec![]),
        };

        let parent = match current_node.parent() {
            Some(parent) => parent,
            None => return Ok(vec![]),
        };

        let query = match (current_node.kind(), parent.kind()) {
            (_, "call_expression") => {
                format!(find_function_str!(), current_node.utf8_text(self.source.as_bytes())?)
            }
            _ => return Ok(vec![]),
        };

        let ts_query = Query::new(tree_sitter_glsl::language(), query.as_str())?;

        let mut query_cursor = QueryCursor::new();

        let mut locations = vec![];

        for m in query_cursor.matches(&ts_query, self.tree.root_node(), self.source.as_bytes()) {
            for capture in m.captures {
                let start = capture.node.start_position();
                let end = capture.node.end_position();

                locations.push(Location {
                    uri: document_uri.clone(),
                    range: Range {
                        start: Position {
                            line: start.row as u32,
                            character: start.column as u32,
                        },
                        end: Position {
                            line: end.row as u32,
                            character: end.column as u32,
                        },
                    },
                });
            }
        }

        info!("finished searching for definitions"; "definitions" => format!("{:?}", locations));

        Ok(locations)
    }

    pub fn find_references(&self) -> Result<Vec<Location>> {
        Ok(vec![])
    }

    fn root_node(&self) -> Node {
        self.tree.root_node()
    }

    fn find_node_at_point(&self, point: Position) -> Option<Node> {
        match self.root_node().named_descendant_for_point_range(
            Point {
                row: point.line as usize,
                column: (point.character - 1) as usize,
            },
            Point {
                row: point.line as usize,
                column: point.character as usize,
            },
        ) {
            Some(node) => {
                debug!("found a node"; "node" => format!("{:?}", node));
                Some(node)
            },
            None => None,
        }
    }
}
