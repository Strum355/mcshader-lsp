use filesystem::NormalizedPathBuf;
use logging::debug;
use regex::Regex;
use std::collections::HashMap;
use tower_lsp::lsp_types::*;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};
use url::Url;

use crate::GPUVendor;
use sourcefile::{SourceMapper, SourceNum, Sourcefile, Version, ROOT_SOURCE_NUM};

pub struct DiagnosticsParser {
    line_regex: Regex,
    line_offset: u32,
}

impl DiagnosticsParser {
    pub fn new(gpu_vendor: GPUVendor, doc_glsl_version: Version) -> Self {
        DiagnosticsParser {
            line_regex: DiagnosticsParser::get_line_regex(gpu_vendor),
            line_offset: DiagnosticsParser::get_line_offset(gpu_vendor, doc_glsl_version),
        }
    }

    fn get_line_regex(gpu_vendor: GPUVendor) -> Regex {
        match gpu_vendor {
            GPUVendor::NVIDIA => {
                Regex::new(r#"^(?P<filepath>\d+)\((?P<linenum>\d+)\) : (?P<severity>error|warning) [A-C]\d+: (?P<output>.+)"#)
            }
            _ => Regex::new(
                r#"^(?P<severity>ERROR|WARNING): (?P<filepath>[^?<>*|"\n]+):(?P<linenum>\d+): (?:'.*' :|[a-z]+\(#\d+\)) +(?P<output>.+)$"#,
            ),
        }
        .unwrap()
    }

    /// for certain NVIDIA GLSL versions, we need to offset the diagnostic number by -1 as those versions (incorrectly/inconsistently) state that:
    /// "After processing this directive (including its new-line), the implementation will behave as if it is compiling at line number line+1".
    /// So to get the correct behaviour (first line), with source strings being 0-based, we need to -1.
    fn get_line_offset(gpu_vendor: GPUVendor, doc_glsl_version: Version) -> u32 {
        match (gpu_vendor, doc_glsl_version) {
            (GPUVendor::NVIDIA, Version::Glsl110)
            | (GPUVendor::NVIDIA, Version::Glsl120)
            | (GPUVendor::NVIDIA, Version::Glsl130)
            | (GPUVendor::NVIDIA, Version::Glsl140)
            | (GPUVendor::NVIDIA, Version::Glsl150) => 1,
            _ => 0,
        }
    }

    pub fn parse_diagnostics_output(
        &self, output: String, uri: &NormalizedPathBuf, source_mapper: &SourceMapper<NormalizedPathBuf>,
        sources: &HashMap<&NormalizedPathBuf, &Sourcefile>,
    ) -> HashMap<Url, Vec<Diagnostic>> {
        let output_lines = output.split('\n').collect::<Vec<&str>>();
        let mut diagnostics: HashMap<Url, Vec<Diagnostic>> = HashMap::with_capacity(output_lines.len());

        debug!("diagnostics regex selected"; "regex" => self.line_regex .as_str());

        for line in output_lines {
            let diagnostic_capture = match self.line_regex.captures(line) {
                Some(d) => d,
                None => continue,
            };

            debug!("found match for output line"; "line" => line, "capture" => format!("{:?}", diagnostic_capture));

            let msg = diagnostic_capture.name("output").unwrap().as_str();

            let source_num: SourceNum = match diagnostic_capture.name("filepath") {
                Some(o) => o.as_str().parse::<usize>().unwrap().into(),
                None => 0.into(),
            };

            let origin = match source_num {
                ROOT_SOURCE_NUM => uri,
                _ => source_mapper.get_node(source_num),
            };

            let line = match diagnostic_capture.name("linenum") {
                Some(c) => {
                    c.as_str().parse::<u32>().unwrap_or(0) - self.line_offset
                }
                None => 0,
            };

            let severity = match diagnostic_capture.name("severity") {
                Some(c) => match c.as_str().to_lowercase().as_str() {
                    "error" => DiagnosticSeverity::ERROR,
                    "warning" => DiagnosticSeverity::WARNING,
                    _ => DiagnosticSeverity::INFORMATION,
                },
                _ => DiagnosticSeverity::INFORMATION,
            };

            let source = sources[origin];
            let (start, end) = source.linemap().line_range_for_position(Position::new(line, 0));
            let line_text = &source.source[start..end.unwrap_or(source.source.len() - 1)];

            let diagnostic = Diagnostic {
                range: Range::new(
                    Position::new(line, (line_text.len() - line_text.trim_start().len()) as u32),
                    Position::new(line, line_text.len() as u32),
                ),
                severity: Some(severity),
                source: Some("mcglsl".to_string()),
                message: msg.trim().into(),
                ..Default::default()
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
    use std::collections::HashMap;

    use filesystem::NormalizedPathBuf;
    use sourcefile::{SourceMapper, Sourcefile};
    use trim_margin::MarginTrimmable;
    use url::Url;

    use crate::diagnostics_parser::DiagnosticsParser;

    #[test]
    #[logging_macro::scope]
    fn test_nvidia_diagnostics_glsl150() {
        logging::scope(&logging::logger().new(slog_o!("driver" => "nvidia")), || {
            let output = "0(1) : error C0000: syntax error, unexpected '}', expecting ',' or ';' at token \"}\"";

            #[cfg(target_family = "unix")]
            let path: NormalizedPathBuf = "/home/noah/.minecraft/shaderpacks/test/shaders/final.fsh".into();
            #[cfg(target_family = "windows")]
            let path: NormalizedPathBuf = "c:\\home\\noah\\.minecraft\\shaderpacks\\test\\shaders\\final.fsh".into();

            let mut source_mapper = SourceMapper::new(0);
            source_mapper.get_num(&path);

            let parser = DiagnosticsParser::new(crate::GPUVendor::NVIDIA, sourcefile::Version::Glsl150);

            let source = Sourcefile::new(
                "sample text".to_string(),
                path.clone(),
                path.parent().and_then(|p| p.parent()).unwrap(),
            );
            let sources = HashMap::from_iter(vec![(&path, &source)]);

            let results = parser.parse_diagnostics_output(output.to_string(), &path, &source_mapper, &sources);

            assert_eq!(results.len(), 1);
            let first = results.into_iter().next().unwrap();
            assert_eq!(first.0, Url::from_file_path(path).unwrap());
        });
    }

    #[test]
    #[logging_macro::scope]
    fn test_nvidia_diagnostics_glsl330() {
        logging::scope(&logging::logger().new(slog_o!("driver" => "nvidia")), || {
            let output = "0(0) : error C0000: syntax error, unexpected '}', expecting ',' or ';' at token \"}\"";

            #[cfg(target_family = "unix")]
            let path: NormalizedPathBuf = "/home/noah/.minecraft/shaderpacks/test/shaders/final.fsh".into();
            #[cfg(target_family = "windows")]
            let path: NormalizedPathBuf = "c:\\home\\noah\\.minecraft\\shaderpacks\\test\\shaders\\final.fsh".into();

            let mut source_mapper = SourceMapper::new(0);
            source_mapper.get_num(&path);

            let parser = DiagnosticsParser::new(crate::GPUVendor::NVIDIA, sourcefile::Version::Glsl330);

            let source = Sourcefile::new(
                "sample text".to_string(),
                path.clone(),
                path.parent().and_then(|p| p.parent()).unwrap(),
            );
            let sources = HashMap::from_iter(vec![(&path, &source)]);

            let results = parser.parse_diagnostics_output(output.to_string(), &path, &source_mapper, &sources);

            assert_eq!(results.len(), 1);
            let first = results.into_iter().next().unwrap();
            assert_eq!(first.0, Url::from_file_path(path).unwrap());
        });
    }

    #[test]
    #[logging_macro::scope]
    fn test_amd_diagnostics() {
        logging::scope(&logging::logger().new(slog_o!("driver" => "amd")), || {
            let output = r#"
                |ERROR: 0:0: '' : syntax error: #line
                |ERROR: 0:1: '' : syntax error: #line
                |ERROR: 0:2: 'varying' : syntax error: syntax error
            "#
            .trim_margin()
            .unwrap();

            #[cfg(target_family = "unix")]
            let path: NormalizedPathBuf = "/home/noah/.minecraft/shaderpacks/test/shaders/final.fsh".into();
            #[cfg(target_family = "windows")]
            let path: NormalizedPathBuf = "c:\\home\\noah\\.minecraft\\shaderpacks\\test\\shaders\\final.fsh".into();

            let mut source_mapper = SourceMapper::new(0);
            source_mapper.get_num(&path);

            let parser = DiagnosticsParser::new(crate::GPUVendor::AMD, sourcefile::Version::Glsl150);

            let source = Sourcefile::new(
                "|int main() {
                |   hello_world();
                |}"
                .to_string()
                .trim_margin()
                .unwrap(),
                path.clone(),
                path.parent().and_then(|p| p.parent()).unwrap(),
            );
            let sources = HashMap::from_iter(vec![(&path, &source)]);

            let results = parser.parse_diagnostics_output(output, &path, &source_mapper, &sources);

            assert_eq!(results.len(), 1);
            let first = results.into_iter().next().unwrap();
            assert_eq!(first.1.len(), 3);
        });
    }
}
