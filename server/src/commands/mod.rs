use std::{collections::HashMap, path::Path};

use serde_json::Value;

use anyhow::{format_err, Result};

pub mod graph_dot;
pub mod merged_includes;

pub struct CustomCommandProvider {
    commands: HashMap<String, Box<dyn Invokeable>>,
}

impl CustomCommandProvider {
    pub fn new(commands: Vec<(&str, Box<dyn Invokeable>)>) -> CustomCommandProvider {
        CustomCommandProvider {
            commands: commands.into_iter().map(|tup| (tup.0.into(), tup.1)).collect(),
        }
    }

    pub fn execute(&self, command: &str, args: &[Value], root_path: &Path) -> Result<Value> {
        if self.commands.contains_key(command) {
            return self.commands.get(command).unwrap().run_command(root_path, args);
        }
        Err(format_err!("command doesn't exist"))
    }
}

pub trait Invokeable {
    fn run_command(&self, root: &Path, arguments: &[Value]) -> Result<Value>;
}
