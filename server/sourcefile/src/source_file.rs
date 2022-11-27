// use core::cell::OnceCell;

use anyhow::Result;
use filesystem::NormalizedPathBuf;
use tree_sitter::{Parser, Query, QueryCursor, Tree};
use tree_sitter_glsl::language;

use crate::{linemap::LineMap, IncludeLine, Version};

const GET_VERSION: &str = r#"
    (translation_unit
        (preproc_call 
            (preproc_directive) @version_str
            (preproc_arg) @version_num))
    (#match? @version_str "\#version")
"#;

pub const GET_INCLUDES: &str = r#"
    (preproc_include 
        (string_literal) @include)
"#;

pub struct Sourcefile {
    pub source: String,
    pub path: NormalizedPathBuf,
    root: NormalizedPathBuf,
    // linemap: OnceCell<LineMap>,
    // tree: OnceCell<Tree>,
}

impl Sourcefile {
    pub fn new<P, R>(source: String, path: P, root: R) -> Self
    where
        P: Into<NormalizedPathBuf>,
        R: Into<NormalizedPathBuf>,
    {
        Self {
            source,
            path: path.into(),
            root: root.into(),
            // linemap: OnceCell::new(),
            // tree: OnceCell::new(),
        }
    }

    pub fn linemap(&self) -> LineMap {
        // self.linemap.get_or_init(|| LineMap::new(&self.source))
        LineMap::new(&self.source)
    }

    pub fn version(&self) -> Result<Version> {
        let query = Query::new(language(), GET_VERSION)?;
        let mut query_cursor = QueryCursor::new();

        let tree = self.tree();
        let version_num_match = query_cursor
            .captures(&query, tree.root_node(), self.source.as_bytes())
            .next()
            .unwrap()
            .0
            .captures[1];

        Ok(
            match version_num_match
                .node
                .utf8_text(self.source.as_bytes())?
                .trim()
                .split(' ')
                .next()
            {
                Some("110") => Version::Glsl110,
                Some("120") => Version::Glsl120,
                Some("130") => Version::Glsl130,
                Some("140") => Version::Glsl140,
                Some("150") => Version::Glsl150,
                Some("330") => Version::Glsl330,
                Some("400") => Version::Glsl400,
                Some("410") => Version::Glsl410,
                Some("420") => Version::Glsl420,
                Some("430") => Version::Glsl430,
                Some("440") => Version::Glsl440,
                Some("450") => Version::Glsl450,
                Some("460") => Version::Glsl460,
                _ => Version::Glsl110,
            },
        )
    }

    pub fn includes(&self) -> Result<Vec<(NormalizedPathBuf, IncludeLine)>> {
        let query = Query::new(language(), GET_INCLUDES)?;
        let mut query_cursor = QueryCursor::new();

        let mut includes = Vec::new();

        for (m, _) in query_cursor.captures(&query, self.tree().root_node(), self.source.as_bytes()) {
            if m.captures.is_empty() {
                continue;
            }

            let include = m.captures[0];
            let include_str = {
                let mut string = include.node.utf8_text(self.source.as_bytes()).unwrap();
                string = &string[1..string.len() - 1];
                if string.starts_with('/') {
                    self.root.join("shaders").join(string.strip_prefix('/').unwrap())
                } else {
                    self.path.parent().unwrap().join(string)
                }
            };

            includes.push((include_str, IncludeLine(include.node.start_position().row)));
        }

        Ok(includes)
    }

    pub fn includes_of_path<'a>(&'a self, child: &'a NormalizedPathBuf) -> Result<impl Iterator<Item = IncludeLine> + '_> {
        Ok(self.includes()?.into_iter().filter(move |(p, _)| p == child).map(|(_, l)| l))
    }

    fn tree(&self) -> Tree {
        // self.tree.get_or_init(|| {
            let mut parser = Parser::new();
            parser.set_language(language()).unwrap();
            parser.parse(&self.source, None).unwrap()
        // })
    }
}

#[cfg(test)]
mod test {
    use crate::{IncludeLine, Sourcefile, Version};
    use anyhow::Result;
    use trim_margin::MarginTrimmable;

    #[test]
    fn test_versions() {
        const SOURCE: &str = r#"
        #version 150 core

        void main() {}
        "#;

        let source = Sourcefile::new(SOURCE.to_string(), "/asdf", "/");
        assert_eq!(source.version().unwrap(), Version::Glsl150);
    }

    #[test]
    fn test_includes() -> Result<()> {
        let source = r#"
            |#version 330
            |
            |#include "path/to/banana.fsh"
            |       #include "/path/to/badbanana.gsh"
        "#
        .trim_margin()
        .unwrap();

        let source = Sourcefile::new(source, "/myshader/shaders/world0/asdf.fsh", "/myshader");
        assert_eq!(
            source.includes()?,
            vec![
                ("/myshader/shaders/world0/path/to/banana.fsh".into(), IncludeLine(2)),
                ("/myshader/shaders/path/to/badbanana.gsh".into(), IncludeLine(3))
            ]
        );
        Ok(())
    }

    #[test]
    fn test_single_includes() -> Result<()> {
        let source = r#"
            |#version 330
            |
            |#include "path/to/banana.fsh"
            |       #include "/path/to/badbanana.gsh"
        "#
        .trim_margin()
        .unwrap();

        let source = Sourcefile::new(source, "/myshader/shaders/world0/asdf.fsh", "/myshader");
        assert_eq!(
            source.includes_of_path(&"/myshader/shaders/world0/path/to/banana.fsh".into())?.collect::<Vec<_>>(),
            vec![IncludeLine(2)]
        );
        Ok(())
    }
}
