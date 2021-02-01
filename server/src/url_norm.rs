use std::path::PathBuf;

use path_slash::PathBufExt;
use url::Url;

pub trait FromUrl {
    fn from_url(u: Url) -> Self;
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