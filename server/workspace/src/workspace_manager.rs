use std::{ffi::OsStr, path::Path};

use filesystem::NormalizedPathBuf;
use glob::{glob_with, MatchOptions};
use logging::{info, error, FutureExt, logger};
use tst::TSTMap;
use walkdir::WalkDir;

use crate::workspace::Workspace;

pub struct WorkspaceIndex(usize);

#[derive(Default)]
pub struct WorkspaceManager<G, F>
where
    G: opengl::ShaderValidator + Send,
    F: Fn() -> G
{
    search: TSTMap<WorkspaceIndex>,
    workspaces: Vec<Workspace<G>>,
    gl_factory: F
}

impl <G, F> WorkspaceManager<G, F> 
where
    G: opengl::ShaderValidator + Send,
    F: Fn() -> G
{
    pub fn new(gl_factory: F) -> Self {
        WorkspaceManager {
            search: Default::default(),
            workspaces: Default::default(),
            gl_factory
        }
    }

    pub async fn gather_workspaces(&mut self, root: &NormalizedPathBuf) {
        let options = MatchOptions {
            case_sensitive: true,
            ..MatchOptions::default()
        };

        let glob = root.join("**").join("shaders.properties");
        info!("banana"; "glob" => &glob);

        for entry in glob_with(&glob.to_string(), options).unwrap() {
            match entry {
                Ok(path)
                    if path.file_name().and_then(OsStr::to_str) == Some("shaders.properties")
                        && path.parent().and_then(Path::file_name).and_then(OsStr::to_str) == Some("shaders") =>
                {
                    match path.parent().and_then(Path::parent).map(Into::into) {
                        Some(shader_root) => self.add_workspace(&shader_root).with_logger(logger()).await,
                        None => todo!(),
                    }
                }
                Ok(path)
                    if path.file_name().and_then(OsStr::to_str) == Some("shaders.properties")
                        && path
                            .parent()
                            .and_then(Path::file_name)
                            .and_then(OsStr::to_str)
                            .map_or(false, |f| f.starts_with("world"))
                        && path
                            .parent()
                            .and_then(Path::parent)
                            .and_then(Path::file_name)
                            .and_then(OsStr::to_str)
                            == Some("shaders") =>
                {
                    match path.parent().and_then(Path::parent).and_then(Path::parent).map(Into::into) {
                        Some(shader_root) => self.add_workspace(&shader_root).with_logger(logger()).await,
                        None => todo!(),
                    }
                }
                Ok(path) => {
                    let path: NormalizedPathBuf = path.into();
                    error!("shaders.properties found outside ./shaders or ./worldX dir"; "path" => path)
                }
                Err(e) => error!("error iterating glob entries"; "error" => format!("{:?}", e)),
            }
        }

        let glob = root.join("**").join("shaders");
        for entry in glob_with(&glob.to_string(), options).unwrap() {
            match entry {
                Ok(path)
                    if !WalkDir::new(path.clone()).into_iter().any(|p| {
                        p.as_ref()
                            .ok()
                            .map(|p| p.file_name())
                            .and_then(|f| f.to_str())
                            .map_or(false, |f| f == "shaders.properties")
                    }) =>
                {
                    match path.parent().map(Into::into) {
                        Some(shader_root) => self.add_workspace(&shader_root).with_logger(logger()).await,
                        None => todo!(),
                    }
                }
                Ok(path) => {
                    let path: NormalizedPathBuf = path.into();
                    info!("skipping as already existing"; "path" => path)
                }
                Err(e) => error!("error iterating glob entries"; "error" => format!("{:?}", e)),
            }
        }
    }

    async fn add_workspace(&mut self, root: &NormalizedPathBuf) {
        if !self.search.contains_key(&root.to_string()) {
            info!("adding workspace"; "root" => &root);
            let opengl_context = (self.gl_factory)();
            let workspace = Workspace::new(root.clone(), opengl_context);
            workspace.build().with_logger(logger()).await;
            self.workspaces.push(workspace);
            self.search.insert(&root.to_string(), WorkspaceIndex(self.workspaces.len() - 1));
        }
    }

    pub fn find_workspace_for_file(&self, file: &NormalizedPathBuf) -> Option<&Workspace<G>> {
        let file = file.to_string();
        let prefix = self.search.longest_prefix(&file);
        if prefix.is_empty() {
            return None;
        }

        match self.search.get(prefix) {
            Some(idx) => self.workspaces.get(idx.0),
            None => None,
        }
    }

    pub fn workspaces(&self) -> &[Workspace<G>] {
        &self.workspaces
    }
}
