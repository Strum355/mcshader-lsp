use filesystem::NormalizedPathBuf;
use graph::FilialTuple;
use include_merger::MergeViewBuilder;
use serde_json::Value;

use anyhow::Result;
use sourcefile::{SourceMapper, Sourcefile};

pub async fn run(path: &NormalizedPathBuf, sources: &[FilialTuple<&Sourcefile>]) -> Result<Option<Value>> {
    let mut source_mapper = SourceMapper::new(sources.len());

    let view = MergeViewBuilder::new(path, sources, &mut source_mapper).build();

    Ok(Some(serde_json::value::Value::String(view.to_string())))
}
