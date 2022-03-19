use std::{collections::HashMap, path::Path};

use once_cell::sync::OnceCell;

use regex::Regex;
use rust_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use url::Url;

use crate::{consts, opengl};

static RE_DIAGNOSTIC: OnceCell<Regex> = OnceCell::new();
fn diagnostics_regex<T>(vendor: &T) -> &'static Regex
where
    T: opengl::ShaderValidator + ?Sized,
{
    RE_DIAGNOSTIC.get_or_init(|| match vendor.vendor().as_str() {
        "NVIDIA" => {
            Regex::new(r#"^(?P<filepath>[^?<>*|"]+)\((?P<linenum>\d+)\) : (?P<severity>error|warning) [A-C]\d+: (?P<output>.+)"#).unwrap()
        }
        _ => {
            Regex::new(r#"^(?P<severity>ERROR|WARNING): (?P<filepath>[^?<>*|"\n]+):(?P<linenum>\d+): '[a-z]*' : (?P<output>.+)$"#).unwrap()
        }
    })
}

static LINE_NUM_OFFSET: OnceCell<u32> = OnceCell::new();
fn line_number_offset<T>(vendor: &T) -> &'static u32
where
    T: opengl::ShaderValidator + ?Sized,
{
    LINE_NUM_OFFSET.get_or_init(|| match vendor.vendor().as_str() {
        "ATI Technologies" => 0,
        _ => 2,
    })
}

pub fn parse_diagnostics_output<T>(output: String, uri: &Path, vendor_querier: &T) -> HashMap<Url, Vec<Diagnostic>>
where
    T: opengl::ShaderValidator + ?Sized,
{
    let output_lines = output.split('\n');
    let mut diagnostics: HashMap<Url, Vec<Diagnostic>> = HashMap::with_capacity(output_lines.count());
    let output_lines = output.split('\n');

    for line in output_lines {
        let diagnostic_capture = match diagnostics_regex(vendor_querier).captures(line) {
            Some(d) => d,
            None => continue,
        };

        // info!("match {:?}", diagnostic_capture);

        let msg = diagnostic_capture.name("output").unwrap().as_str();

        let line = match diagnostic_capture.name("linenum") {
            Some(c) => c.as_str().parse::<u32>().unwrap_or(0),
            None => 0,
        } - line_number_offset(vendor_querier);

        // TODO: line matching maybe
        /* let line_text = source_lines[line as usize];
        let leading_whitespace = line_text.len() - line_text.trim_start().len(); */

        let severity = match diagnostic_capture.name("severity") {
            Some(c) => match c.as_str().to_lowercase().as_str() {
                "error" => DiagnosticSeverity::Error,
                "warning" => DiagnosticSeverity::Warning,
                _ => DiagnosticSeverity::Information,
            },
            _ => DiagnosticSeverity::Information,
        };

        let origin = match diagnostic_capture.name("filepath") {
            Some(o) => {
                if o.as_str() == "0" {
                    uri.to_str().unwrap().to_string()
                } else {
                    o.as_str().to_string()
                }
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
