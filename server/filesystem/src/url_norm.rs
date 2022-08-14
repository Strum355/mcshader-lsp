use std::{
    fmt::{Display, Debug},
    path::{Path, PathBuf},
};

use anyhow::Result;
use logging::trace;
use path_slash::PathBufExt;
use serde_json::value::Value;
use url::Url;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NormalizedPathBuf(PathBuf);

impl NormalizedPathBuf {
    pub fn join(&self, path: impl Into<PathBuf>) -> NormalizedPathBuf {
        NormalizedPathBuf(PathBuf::from_slash(self.0.join(path.into()).to_str().unwrap()))
    }

    pub fn parent(&self) -> Option<NormalizedPathBuf> {
        self.0.parent().map(Into::into)
    }

    pub fn extension(&self) -> Option<&str> {
        self.0.extension().and_then(|e| e.to_str())
    }

    pub fn strip_prefix(&self, prefix: &Self) -> NormalizedPathBuf {
        self.0.strip_prefix(prefix.clone().0).unwrap().into()
    }

    pub fn exists(&self) -> bool {
        self.0.exists()
    }
}

impl Debug for NormalizedPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

impl AsRef<Path> for NormalizedPathBuf {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl AsRef<PathBuf> for NormalizedPathBuf {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl From<&NormalizedPathBuf> for PathBuf {
    fn from(p: &NormalizedPathBuf) -> Self {
        PathBuf::from_slash(p.0.to_str().unwrap())
    }
}

impl From<&Path> for NormalizedPathBuf {
    fn from(p: &Path) -> Self {
        // TODO: is this right??
        PathBuf::from_slash(p.to_str().unwrap()).into()
    }
}

impl From<PathBuf> for NormalizedPathBuf {
    fn from(p: PathBuf) -> Self {
        // don't use p.as_path().into(), it'll cause infinite recursion with above impl
        p.to_str().unwrap().into()
    }
}

impl From<&str> for NormalizedPathBuf {
    fn from(s: &str) -> Self {
        // TODO: is this right??
        NormalizedPathBuf(PathBuf::from_slash(s))
    }
}


impl logging::Value for NormalizedPathBuf {
    fn serialize(&self, record: &logging::Record, key: logging::Key, serializer: &mut dyn logging::Serializer) -> logging::Result {
        self.0.to_str().unwrap().serialize(record, key, serializer)
    }
}

impl Display for NormalizedPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl From<Url> for NormalizedPathBuf {
    #[cfg(target_family = "windows")]
    fn from(u: Url) -> Self {
        let path = PathBuf::from_slash(
            percent_encoding::percent_decode_str(u.path().strip_prefix('/').unwrap())
                .decode_utf8()
                .unwrap(),
        );

        trace!("converted win path from url"; "old" => u.as_str(), "new" => path.to_str().unwrap());

        NormalizedPathBuf(path)
    }

    #[cfg(target_family = "unix")]
    fn from(u: Url) -> Self {
        let path = PathBuf::from_slash(percent_encoding::percent_decode_str(u.path()).decode_utf8().unwrap());

        trace!("converted unix path from url"; "old" => u.as_str(), "new" => path.to_str().unwrap());

        NormalizedPathBuf(path)
    }
}

impl TryFrom<&Value> for NormalizedPathBuf {
    type Error = anyhow::Error;

    #[cfg(target_family = "windows")]
    fn try_from(v: &Value) -> Result<Self> {
        if !v.is_string() {
            return Err(anyhow::format_err!("cannot convert {:?} to PathBuf", v));
        }
        let path = v.to_string();
        let path = PathBuf::from_slash(
            percent_encoding::percent_decode_str(path.trim_start_matches('"').trim_end_matches('"').strip_prefix('/').unwrap())
                .decode_utf8()?,
        );

        trace!("converted win path from json"; "old" => v.to_string(), "new" => path.to_str().unwrap());

        Ok(NormalizedPathBuf(path))
    }

    #[cfg(target_family = "unix")]
    fn try_from(v: &serde_json::value::Value) -> Result<Self> {
        if !v.is_string() {
            return Err(anyhow::format_err!("cannot convert {:?} to PathBuf", v));
        }
        let path = v.to_string();
        let path =
            PathBuf::from_slash(percent_encoding::percent_decode_str(path.trim_start_matches('"').trim_end_matches('"')).decode_utf8()?);

        trace!("converted unix path from json"; "old" => v.to_string(), "new" => path.to_str().unwrap());

        Ok(NormalizedPathBuf(path))
    }
}
