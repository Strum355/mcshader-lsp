use std::path::PathBuf;

use slog_scope::trace;
use anyhow::Result;
use path_slash::PathBufExt;
use url::Url;

pub trait FromUrl {
    fn from_url(u: Url) -> Self;
}

pub trait FromJson {
    fn from_json(v: &serde_json::value::Value) -> Result<Self>
    where
        Self: Sized;
}

impl FromUrl for PathBuf {
    #[cfg(target_family = "windows")]
    fn from_url(u: Url) -> Self {
        let path = percent_encoding::percent_decode_str(u.path().strip_prefix('/').unwrap())
            .decode_utf8()
            .unwrap();
        
        trace!("converted win path from url"; "old" => u.as_str(), "new" => path.to_string());

        PathBuf::from_slash(path)
    }

    #[cfg(target_family = "unix")]
    fn from_url(u: Url) -> Self {
        let path = percent_encoding::percent_decode_str(u.path()).decode_utf8().unwrap();
        
        trace!("converted unix path from url"; "old" => u.as_str(), "new" => path.to_string());

        PathBuf::from_slash(path)
    }
}

impl FromJson for PathBuf {
    #[cfg(target_family = "windows")]
    fn from_json(v: &serde_json::value::Value) -> Result<Self>
    where
        Self: Sized,
    {
        if !v.is_string() {
            return Err(anyhow::format_err!("cannot convert {:?} to PathBuf", v));
        }
        let path = v.to_string();
        let path = percent_encoding::percent_decode_str(path.trim_start_matches('"').trim_end_matches('"').strip_prefix('/').unwrap())
            .decode_utf8()?;

        trace!("converted win path from json"; "old" => v.to_string(), "new" => path.to_string());

        Ok(PathBuf::from_slash(path))
    }

    #[cfg(target_family = "unix")]
    fn from_json(v: &serde_json::value::Value) -> Result<Self>
    where
        Self: Sized,
    {
        if !v.is_string() {
            return Err(anyhow::format_err!("cannot convert {:?} to PathBuf", v));
        }
        let path = v.to_string();
        let path = percent_encoding::percent_decode_str(path.trim_start_matches('"').trim_end_matches('"')).decode_utf8()?;

        trace!("converted unix path from json"; "old" => v.to_string(), "new" => path.to_string());

        Ok(PathBuf::from_slash(path))
    }
}
