use anyhow::{anyhow, Result};
#[cfg(not(test))]
use std::fs;
use std::{ops::Deref, path::Path};
#[cfg(test)]
use {
    logging::{logger, FutureExt},
    tokio::fs,
};

/// Denotes a string with any `\r\n` replaced with `\n`
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LFString(String);

impl LFString {
    pub async fn read<P: AsRef<Path>>(path: P) -> Result<LFString> {
        #[cfg(test)]
        let read_result = fs::read_to_string(&path).with_logger(logger()).await;
        #[cfg(not(test))]
        let read_result = fs::read_to_string(&path);

        let source = read_result.map_err(|e| anyhow!("error reading {:?}: {}", path.as_ref(), e))?;
        Ok(LFString(source.replace("\r\n", "\n")))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        LFString(String::with_capacity(capacity))
    }

    pub fn from_unchecked(string: String) -> Self {
        LFString(string)
    }
}

impl Deref for LFString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
