#![feature(once_cell)]

mod linemap;
mod source_file;
mod source_mapper;
use std::fmt::{Debug, Display, Formatter};

pub use linemap::*;
use logging::Value;
pub use source_file::*;
pub use source_mapper::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IncludeLine(usize);

impl From<IncludeLine> for usize {
    fn from(n: IncludeLine) -> Self {
        n.0
    }
}

impl From<usize> for IncludeLine {
    fn from(n: usize) -> Self {
        IncludeLine(n)
    }
}

impl std::ops::Add<usize> for IncludeLine {
    type Output = IncludeLine;

    fn add(self, rhs: usize) -> Self::Output {
        IncludeLine(self.0 + rhs)
    }
}

impl PartialEq<usize> for IncludeLine {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl Value for IncludeLine {
    fn serialize(&self, record: &logging::Record, key: logging::Key, serializer: &mut dyn logging::Serializer) -> logging::Result {
        self.0.serialize(record, key, serializer)
    }
}

impl Debug for IncludeLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{line: {}}}", self.0)
    }
}

impl Display for IncludeLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{{line: {}}}", self.0)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Version {
    Glsl110 = 110,
    Glsl120 = 120,
    Glsl130 = 130,
    Glsl140 = 140,
    Glsl150 = 150,
    Glsl330 = 330,
    Glsl400 = 400,
    Glsl410 = 410,
    Glsl420 = 420,
    Glsl430 = 430,
    Glsl440 = 440,
    Glsl450 = 450,
    Glsl460 = 460,
}
