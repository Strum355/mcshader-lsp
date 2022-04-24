use std::{collections::HashMap, lazy::OnceCell, path::Path};

use regex::Regex;
use rust_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use slog_scope::debug;
use url::Url;

use crate::{
    consts,
    graph::CachedStableGraph,
    opengl,
    source_mapper::{SourceMapper, SourceNum},
};

pub struct DiagnosticsParser<'a, T: opengl::ShaderValidator + ?Sized> {
    line_offset: OnceCell<u32>,
    line_regex: OnceCell<Regex>,
    vendor_querier: &'a T,
}

impl<'a, T: opengl::ShaderValidator + ?Sized> DiagnosticsParser<'a, T> {
    pub fn new(vendor_querier: &'a T) -> Self {
        DiagnosticsParser {
            line_offset: OnceCell::new(),
            line_regex: OnceCell::new(),
            vendor_querier,
        }
    }

    fn get_line_regex(&self) -> &Regex {
        self.line_regex.get_or_init(|| match self.vendor_querier.vendor().as_str() {
            "NVIDIA Corporation" => {
                Regex::new(r#"^(?P<filepath>\d+)\((?P<linenum>\d+)\) : (?P<severity>error|warning) [A-C]\d+: (?P<output>.+)"#).unwrap()
            }
            _ => Regex::new(r#"^(?P<severity>ERROR|WARNING): (?P<filepath>[^?<>*|"\n]+):(?P<linenum>\d+): (?:'.*' :|[a-z]+\(#\d+\)) +(?P<output>.+)$"#)
                .unwrap(),
        })
    }

    fn get_line_offset(&self) -> u32 {
        *self.line_offset.get_or_init(|| match self.vendor_querier.vendor().as_str() {
            "ATI Technologies" => 0,
            _ => 1,
        })
    }

    pub fn parse_diagnostics_output(
        &self, output: String, uri: &Path, source_mapper: &SourceMapper, graph: &CachedStableGraph,
    ) -> HashMap<Url, Vec<Diagnostic>> {
        let output_lines = output.split('\n').collect::<Vec<&str>>();
        let mut diagnostics: HashMap<Url, Vec<Diagnostic>> = HashMap::with_capacity(output_lines.len());

        debug!("diagnostics regex selected"; "regex" => self.get_line_regex() .as_str());

        for line in output_lines {
            let diagnostic_capture = match self.get_line_regex().captures(line) {
                Some(d) => d,
                None => continue,
            };

            debug!("found match for output line"; "line" => line, "capture" => format!("{:?}", diagnostic_capture));

            let msg = diagnostic_capture.name("output").unwrap().as_str();

            let line = match diagnostic_capture.name("linenum") {
                Some(c) => c.as_str().parse::<u32>().unwrap_or(0),
                None => 0,
            } - self.get_line_offset();

            // TODO: line matching maybe
            /* let line_text = source_lines[line as usize];
            let leading_whitespace = line_text.len() - line_text.trim_start().len(); */

            let severity = match diagnostic_capture.name("severity") {
                Some(c) => match c.as_str().to_lowercase().as_str() {
                    "error" => DiagnosticSeverity::ERROR,
                    "warning" => DiagnosticSeverity::WARNING,
                    _ => DiagnosticSeverity::INFORMATION,
                },
                _ => DiagnosticSeverity::INFORMATION,
            };

            let origin = match diagnostic_capture.name("filepath") {
                Some(o) => {
                    let source_num: SourceNum = o.as_str().parse::<usize>().unwrap().into();
                    let graph_node = source_mapper.get_node(source_num);
                    graph.get_node(graph_node).to_str().unwrap().to_string()
                }
                None => uri.to_str().unwrap().to_string(),
            };

            let diagnostic = Diagnostic {
                range: Range::new(
                    /* Position::new(line, leading_whitespace as u64),
                    Position::new(line, line_text.len() as u64) */
                    Position::new(line, 0),
                    Position::new(line, 1000),
                ),
                code: None,
                severity: Some(severity),
                source: Some(consts::SOURCE.into()),
                message: msg.trim().into(),
                related_information: None,
                tags: None,
                code_description: Option::None,
                data: Option::None,
            };

            let origin_url = Url::from_file_path(origin).unwrap();
            match diagnostics.get_mut(&origin_url) {
                Some(d) => d.push(diagnostic),
                None => {
                    diagnostics.insert(origin_url, vec![diagnostic]);
                }
            };
        }
        diagnostics
    }
}

#[cfg(test)]
mod diagnostics_test {
    use std::path::PathBuf;

    use slog::slog_o;
    use url::Url;

    use crate::{
        diagnostics_parser::DiagnosticsParser, opengl::MockShaderValidator, source_mapper::SourceMapper, test::new_temp_server,
    };

    #[test]
    #[logging_macro::log_scope]
    fn test_nvidia_diagnostics() {
        slog_scope::scope(&slog_scope::logger().new(slog_o!("driver" => "nvidia")), || {
            let mut mockgl = MockShaderValidator::new();
            mockgl.expect_vendor().returning(|| "NVIDIA Corporation".into());
            let server = new_temp_server(Some(Box::new(mockgl)));

            let output = "0(9) : error C0000: syntax error, unexpected '}', expecting ',' or ';' at token \"}\"";

            #[cfg(target_family = "unix")]
            let path: PathBuf = "/home/noah/.minecraft/shaderpacks/test/shaders/final.fsh".into();
            #[cfg(target_family = "windows")]
            let path: PathBuf = "c:\\home\\noah\\.minecraft\\shaderpacks\\test\\shaders\\final.fsh".into();

            let mut source_mapper = SourceMapper::new(0);
            source_mapper.get_num(server.graph.borrow_mut().add_node(&path));

            let parser = DiagnosticsParser::new(server.opengl_context.as_ref());

            let results =
                parser.parse_diagnostics_output(output.to_string(), path.parent().unwrap(), &source_mapper, &server.graph.borrow());

            assert_eq!(results.len(), 1);
            let first = results.into_iter().next().unwrap();
            assert_eq!(first.0, Url::from_file_path(path).unwrap());
            server.endpoint.request_shutdown();
        });
    }

    #[test]
    #[logging_macro::log_scope]
    fn test_amd_diagnostics() {
        slog_scope::scope(&slog_scope::logger().new(slog_o!("driver" => "amd")), || {
            let mut mockgl = MockShaderValidator::new();
            mockgl.expect_vendor().returning(|| "ATI Technologies".into());
            let server = new_temp_server(Some(Box::new(mockgl)));

            let output = "ERROR: 0:1: '' : syntax error: #line
ERROR: 0:10: '' : syntax error: #line
ERROR: 0:15: 'varying' : syntax error: syntax error
";

            #[cfg(target_family = "unix")]
            let path: PathBuf = "/home/noah/.minecraft/shaderpacks/test/shaders/final.fsh".into();
            #[cfg(target_family = "windows")]
            let path: PathBuf = "c:\\home\\noah\\.minecraft\\shaderpacks\\test\\shaders\\final.fsh".into();

            let mut source_mapper = SourceMapper::new(0);
            source_mapper.get_num(server.graph.borrow_mut().add_node(&path));

            let parser = DiagnosticsParser::new(server.opengl_context.as_ref());

            let results =
                parser.parse_diagnostics_output(output.to_string(), path.parent().unwrap(), &source_mapper, &server.graph.borrow());

            assert_eq!(results.len(), 1);
            let first = results.into_iter().next().unwrap();
            assert_eq!(first.1.len(), 3);
            server.endpoint.request_shutdown();
        });
    }
}
