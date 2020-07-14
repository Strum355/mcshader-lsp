use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::prelude::*;

use serde_json::Value;

use petgraph::dot;

use crate::graph::CachedStableGraph;

pub struct CustomCommandProvider {
    commands: HashMap<String, Box<dyn Invokeable>>
}

impl CustomCommandProvider {
    pub fn new(commands: Vec<(&str, Box<dyn Invokeable>)>) -> CustomCommandProvider {
       CustomCommandProvider{
            commands: commands.into_iter().map(|tup| {
                (String::from(tup.0), tup.1)
            }).collect(),
        }
    }

    pub fn execute(&self, command: &str, args: Vec<Value>) -> Result<(), String> {
        if self.commands.contains_key(command) {
            return self.commands.get(command).unwrap().run_command(args);
        }
        Err(String::from("command doesn't exist"))
    }
}

pub trait Invokeable {
    fn run_command(&self, arguments: Vec<Value>) -> Result<(), String>;
}

pub struct GraphDotCommand {
    pub graph: Rc<RefCell<CachedStableGraph>>
}

impl<'a> Invokeable for GraphDotCommand {
    fn run_command(&self, params: Vec<Value>) -> Result<(), String> {
        let rootpath = params.get(0).unwrap().to_string();
        let rootpath = String::from(rootpath.trim_start_matches('"').trim_end_matches('"'));
        let filepath = rootpath + "/graph.dot";
        eprintln!("generating dot file at {}", filepath);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(filepath)
            .unwrap();

        let mut write_data_closure = || -> Result<(), std::io::Error> {
            let graph = self.graph.as_ref();
            file.seek(std::io::SeekFrom::Start(0))?;
            file.write_all(dot::Dot::new(&(graph.borrow().graph)).to_string().as_bytes())?;
            file.flush()?;
            file.seek(std::io::SeekFrom::Start(0))?;
            Ok(())
        };

        match write_data_closure() {
            Err(err) => Err(format!("Error generating graphviz data: {}", err)),
            _ => Ok(())
        }
    }
}