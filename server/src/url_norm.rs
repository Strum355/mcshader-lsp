use std::path::PathBuf;

use path_slash::PathBufExt;
use url::Url;
use anyhow::Result;

pub trait FromUrl {
    fn from_url(u: Url) -> Self;
}

pub trait FromJSON {
    fn from_json(v: &serde_json::value::Value) -> Result<Self> where Self: Sized;
}

impl FromUrl for PathBuf {
    #[cfg(target_family = "windows")]
    fn from_url(u: Url) -> Self {
        let path = percent_encoding::percent_decode_str(u.path().strip_prefix("/").unwrap()).decode_utf8().unwrap();
        PathBuf::from_slash(path)
    }

    #[cfg(target_family = "unix")]
    fn from_url(u: Url) -> Self {
        let path = percent_encoding::percent_decode_str(u.path()).decode_utf8().unwrap();
        PathBuf::from_slash(path)
    }
}

impl FromJSON for PathBuf {
    #[cfg(target_family = "windows")]
    fn from_json(v: &serde_json::value::Value) -> Result<Self>
    where Self: Sized {
        if !v.is_string() {
            return Err(anyhow::format_err!("cannot convert {:?} to PathBuf", v));
        }
        let path = v.to_string();
        let path = percent_encoding::percent_decode_str(
            path.trim_start_matches('"').trim_end_matches('"').strip_prefix("/").unwrap()
        ).decode_utf8()?;
        Ok(PathBuf::from_slash(path))
    }

    #[cfg(target_family = "unix")]
    fn from_json(v: &serde_json::value::Value) -> Result<Self>
    where Self: Sized {
        if !v.is_string() {
            return Err(anyhow::format_err!("cannot convert {:?} to PathBuf", v));
        }
        let path = v.to_string();
        let path = percent_encoding::percent_decode_str(
            path.trim_start_matches('"').trim_end_matches('"')
        ).decode_utf8()?;
        Ok(PathBuf::from_slash(path))
    }
}