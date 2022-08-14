#![feature(once_cell)]
mod opengl_context;
mod opengl_context_facade;

use mockall::automock;
use opengl_context::*;
pub use opengl_context_facade::*;

pub mod diagnostics_parser;

use std::fmt::Debug;

#[automock]
pub trait ShaderValidator {
    fn validate(&self, tree_type: TreeType, source: &str) -> Option<String>;
    fn vendor(&self) -> String;
}

#[derive(Debug, Clone, Copy)]
pub enum GPUVendor {
    NVIDIA, AMD, OTHER // and thats it folks
}

impl From<&str> for GPUVendor {
    fn from(s: &str) -> Self {
        match s {
            "NVIDIA Corporation" => Self::NVIDIA,
            "AMD" | "ATI Technologies" | "ATI Technologies Inc." => Self::AMD,
            _ => Self::OTHER
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TreeType {
    Fragment,
    Vertex,
    Geometry,
    Compute,
}

impl From<&str> for TreeType {
    fn from(ext: &str) -> Self {
        if ext == "fsh" {
            TreeType::Fragment
        } else if ext == "vsh" {
            TreeType::Vertex
        } else if ext == "gsh" {
            TreeType::Geometry
        } else if ext == "csh" {
            TreeType::Compute
        } else {
            unreachable!();
        }
    }
}
