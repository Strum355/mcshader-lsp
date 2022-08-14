use std::{collections::HashMap, fs::read_to_string, path::Path, vec};

use anyhow::Result;
use logging::{trace, debug, info};
use sourcefile::LineMap;
use tower_lsp::lsp_types::{DocumentSymbol, Location, Position, Range, SymbolKind};
use tree_sitter::{Node, Parser, Point, Query, QueryCursor, Tree};
use url::Url;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
struct SymbolName(String);

impl SymbolName {
    // construct a new SymbolName from a node and its node ID for overload disambiguating.
    fn new(node: &Node, source: &str, node_id: usize) -> Self {
        let mut fqname = vec![format!("{}[{}]", node.utf8_text(source.as_bytes()).unwrap(), node_id)];

        // first node will always have a parent
        let mut prev = *node;
        let mut node = node.parent().unwrap();

        loop {
            match (node.kind(), prev.kind()) {
                ("function_definition", "compound_statement") => {
                    let func_ident = node.child_by_field_name("declarator").unwrap().child(0).unwrap();
                    fqname.push(format!("{}[{}]", func_ident.utf8_text(source.as_bytes()).unwrap(), func_ident.id()));
                }
                ("struct_specifier", "field_declaration_list") => {
                    let struct_ident = node.child_by_field_name("name").unwrap();
                    fqname.push(format!(
                        "{}[{}]",
                        struct_ident.utf8_text(source.as_bytes()).unwrap(),
                        struct_ident.id()
                    ));
                }
                _ => (),
            }

            prev = node;
            node = match node.parent() {
                Some(n) => n,
                None => break,
            };
        }

        fqname.reverse();
        SymbolName(fqname.join("/"))
    }

    fn parent(&self) -> Option<Self> {
        self.0.rsplit_once('/').map(|(left, _)| SymbolName(left.to_string()))
    }
}

impl logging::Value for SymbolName {
    fn serialize(&self, record: &logging::Record, key: logging::Key, serializer: &mut dyn logging::Serializer) -> logging::Result {
        self.0.serialize(record, key, serializer)
    }
}

macro_rules! find_function_def_str {
    () => {
        r#"
            (
                (function_declarator 
                    (identifier) @function)
                (#match? @function "^{}$")
            )
        "#
    };
}

macro_rules! find_function_refs_str {
    () => {
        r#"
            (
                (call_expression 
                    (identifier) @call)
                (#match? @call "^{}$")
            )
        "#
    };
}

macro_rules! find_variable_def_str {
    () => {
        r#"
            [
                (init_declarator 
                    (identifier) @variable)
                    
                (parameter_declaration
                    (identifier) @variable)
                    
                (declaration
                    (identifier) @variable)
                
                (#match? @variable "^{}$")
            ]
        "#
    };
}

const LIST_SYMBOLS_STR: &str = r#"
    ; global consts
    (declaration
        (type_qualifier) @const_qualifier
            (init_declarator
                (identifier) @const_ident))
    (#match? @const_qualifier "^const")
    
    ; global uniforms, varyings, struct variables etc
    (translation_unit
    	(declaration
    		(identifier) @ident))
        
    ; #defines
    (preproc_def
        (identifier) @define_ident)
    
    ; function definitions
    (function_declarator
        (identifier) @func_ident)

    ; struct definitions
    (struct_specifier
        (type_identifier) @struct_ident)

    ; struct fields
    (struct_specifier
        (field_declaration_list
            (field_declaration
                [
                  (field_identifier) @field_ident
                  (array_declarator
                      (field_identifier) @field_ident)
                 ])) @field_list)
"#;

pub struct ParserContext<'a> {
    source: String,
    tree: Tree,
    linemap: LineMap,
    parser: &'a mut Parser,
}

impl<'a> ParserContext<'a> {
    pub fn new(parser: &'a mut Parser, path: &Path) -> Result<Self> {
        let source = read_to_string(path)?;

        let tree = parser.parse(&source, None).unwrap();

        let linemap = LineMap::new(&source);

        Ok(ParserContext {
            source,
            tree,
            linemap,
            parser,
        })
    }

    pub fn list_document_symbols(&self, _path: &Path) -> Result<Option<Vec<DocumentSymbol>>> {
        let query = Query::new(tree_sitter_glsl::language(), LIST_SYMBOLS_STR)?;
        let mut query_cursor = QueryCursor::new();

        let mut parent_child_vec: Vec<(Option<SymbolName>, DocumentSymbol)> = vec![];
        let mut fqname_to_index: HashMap<SymbolName, usize> = HashMap::new();

        for (m, _) in query_cursor.captures(&query, self.root_node(), self.source.as_bytes()) {
            if m.captures.is_empty() {
                continue;
            }

            let mut capture_iter = m.captures.iter();

            let capture = capture_iter.next().unwrap();
            let capture_name = query.capture_names()[capture.index as usize].as_str();

            trace!("next capture name"; "name" => capture_name, "capture" => format!("{:?}", capture));

            let (kind, node) = match capture_name {
                "const_qualifier" => (SymbolKind::CONSTANT, capture_iter.next().unwrap().node),
                "ident" => (SymbolKind::VARIABLE, capture.node),
                "func_ident" => (SymbolKind::FUNCTION, capture.node),
                "define_ident" => (SymbolKind::STRING, capture.node),
                "struct_ident" => (SymbolKind::STRUCT, capture.node),
                "field_list" => (SymbolKind::FIELD, capture_iter.next().unwrap().node),
                _ => (SymbolKind::NULL, capture.node),
            };

            let range = Range {
                start: Position {
                    line: node.start_position().row as u32,
                    character: node.start_position().column as u32,
                },
                end: Position {
                    line: node.end_position().row as u32,
                    character: node.end_position().column as u32,
                },
            };

            let name = node.utf8_text(self.source.as_bytes()).unwrap().to_string();

            let fqname = SymbolName::new(&node, self.source.as_str(), node.id());

            debug!("found symbol"; "node_name" => &name, "kind" => format!("{:?}", kind), "fqname" => &fqname);

            let child_symbol = DocumentSymbol {
                name,
                detail: None,
                kind,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: None,
            };
            parent_child_vec.push((fqname.parent(), child_symbol));
            trace!("inserting fqname"; "fqname" => &fqname, "index" => parent_child_vec.len() - 1);
            fqname_to_index.insert(fqname, parent_child_vec.len() - 1);
        }

        for i in 1..parent_child_vec.len() {
            let (left, right) = parent_child_vec.split_at_mut(i);
            let parent = &right[0].0;
            let child = &right[0].1;
            if let Some(parent) = parent {
                trace!("finding parent"; "parent_symbol_name" => &parent, "child" => format!("{:?}", child), "split_point" => i, "left_len" => left.len(), "right_len" => right.len());
                let parent_index = fqname_to_index.get(parent).unwrap();
                let parent_sym = &mut left[*parent_index];
                parent_sym.1.children.get_or_insert_default().push(right[0].1.clone())
            }
        }

        let symbols = parent_child_vec
            .iter()
            .filter(|tuple| tuple.0.is_none())
            .map(|tuple| tuple.1.clone())
            .collect();

        Ok(Some(symbols))
    }

    pub fn find_definitions(&self, path: &Path, point: Position) -> Result<Option<Vec<Location>>> {
        let current_node = match self.find_node_at_point(point) {
            Some(node) => node,
            None => return Ok(None),
        };

        let parent = match current_node.parent() {
            Some(parent) => parent,
            None => return Ok(None),
        };

        debug!("matching location lookup method for parent-child tuple"; "parent" => parent.kind(), "child" => current_node.kind());

        let locations = match (current_node.kind(), parent.kind()) {
            (_, "call_expression") => {
                let query_str = format!(find_function_def_str!(), current_node.utf8_text(self.source.as_bytes())?);
                self.simple_global_search(path, &query_str)?
            }
            ("identifier", "argument_list")
            | ("identifier", "field_expression")
            | ("identifier", "binary_expression")
            | ("identifier", "assignment_expression") => self.tree_climbing_search(path, current_node)?,
            _ => return Ok(None),
        };

        info!("finished searching for definitions"; "count" => locations.len(), "definitions" => format!("{:?}", locations));

        Ok(Some(locations))
    }

    pub fn find_references(&self, path: &Path, point: Position) -> Result<Option<Vec<Location>>> {
        let current_node = match self.find_node_at_point(point) {
            Some(node) => node,
            None => return Ok(None),
        };

        let parent = match current_node.parent() {
            Some(parent) => parent,
            None => return Ok(None),
        };

        let locations = match (current_node.kind(), parent.kind()) {
            (_, "function_declarator") => {
                let query_str = format!(find_function_refs_str!(), current_node.utf8_text(self.source.as_bytes())?);
                self.simple_global_search(path, &query_str)?
            }
            _ => return Ok(None),
        };

        info!("finished searching for references"; "count" => locations.len(), "references" => format!("{:?}", locations));

        Ok(Some(locations))
    }

    fn tree_climbing_search(&self, path: &Path, start_node: Node) -> Result<Vec<Location>> {
        let mut locations = vec![];

        let node_text = start_node.utf8_text(self.source.as_bytes())?;

        let query_str = format!(find_variable_def_str!(), node_text);

        debug!("built query string"; "query" => &query_str);

        let mut parent = start_node.parent();

        loop {
            if parent.is_none() {
                trace!("no more parent left, found nothing");
                break;
            }

            let query = Query::new(tree_sitter_glsl::language(), &query_str)?;
            let mut query_cursor = QueryCursor::new();

            trace!("running tree-sitter query for node"; "node" => format!("{:?}", parent.unwrap()), "node_text" => parent.unwrap().utf8_text(self.source.as_bytes()).unwrap());

            for m in query_cursor.matches(&query, parent.unwrap(), self.source.as_bytes()) {
                for capture in m.captures {
                    let start = capture.node.start_position();
                    let end = capture.node.end_position();

                    locations.push(Location {
                        uri: Url::from_file_path(path).unwrap(),
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

            if !locations.is_empty() {
                break;
            }

            parent = parent.unwrap().parent();
        }

        Ok(locations)
    }

    fn simple_global_search(&self, path: &Path, query_str: &str) -> Result<Vec<Location>> {
        let query = Query::new(tree_sitter_glsl::language(), query_str)?;
        let mut query_cursor = QueryCursor::new();

        let mut locations = vec![];

        for m in query_cursor.matches(&query, self.root_node(), self.source.as_bytes()) {
            for capture in m.captures {
                let start = capture.node.start_position();
                let end = capture.node.end_position();

                locations.push(Location {
                    uri: Url::from_file_path(path).unwrap(),
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

        Ok(locations)
    }

    fn root_node(&self) -> Node {
        self.tree.root_node()
    }

    fn find_node_at_point(&self, pos: Position) -> Option<Node> {
        // if we're at the end of an ident, we need to look _back_ one char instead
        // for tree-sitter to find the right node.
        let look_behind = {
            let offset = self.linemap.offset_for_position(pos);
            let char_at = self.source.as_bytes()[offset];
            trace!("looking for non-alpha for point adjustment";
                "offset" => offset, 
                "char" => char_at as char,
                "point" => format!("{:?}", pos),
                "look_behind" => !char_at.is_ascii_alphabetic());
            !char_at.is_ascii_alphabetic()
        };

        let mut start = Point {
            row: pos.line as usize,
            column: pos.character as usize,
        };
        let mut end = Point {
            row: pos.line as usize,
            column: pos.character as usize,
        };

        if look_behind {
            start.column -= 1;
        } else {
            end.column += 1;
        }

        match self.root_node().named_descendant_for_point_range(start, end) {
            Some(node) => {
                debug!("found a node"; 
                    "node" => format!("{:?}", node),
                    "text" => node.utf8_text(self.source.as_bytes()).unwrap(),
                    "start" => format!("{}", start),
                    "end" => format!("{}", end));
                Some(node)
            }
            None => None,
        }
    }
}
