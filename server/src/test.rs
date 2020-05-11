use super::*;
use std::io;
use std::io::Result;
use tempdir::TempDir;
use std::fs;

use fs_extra;

use jsonrpc_common::*;
use jsonrpc_response::*;

struct StdoutNewline {
    s: io::Stdout
}

impl io::Write for StdoutNewline {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let res = self.s.write(buf);
        if buf[buf.len()-1] == "}".as_bytes()[0] {
            #[allow(unused_variables)]
            let res = self.s.write("\n\n".as_bytes());
            
        }
        res
    }

    fn flush(&mut self) -> Result<()> {
        self.s.flush()
    }
}

fn new_temp_server() -> MinecraftShaderLanguageServer {
    let endpoint = LSPEndpoint::create_lsp_output_with_output_stream(|| StdoutNewline{s: io::stdout()});

    MinecraftShaderLanguageServer{
        endpoint,
        graph: Rc::new(RefCell::new(graph::CachedStableGraph::new())),
        config: Configuration::default(),
        wait: WaitGroup::new(),
        root: None,
        command_provider: None,
    }
}

#[test]
fn test_empty_initialize() {
    let mut server = new_temp_server();

    let tmp_dir = TempDir::new("mcshader").unwrap();
    let tmp_path = tmp_dir.path().as_os_str().to_str().unwrap();
    let tmp_uri = format!("{}{}", "file://", tmp_path);

    let initialize_params = InitializeParams{
        process_id: None,
        root_path: None,
        root_uri: Some(Url::parse(tmp_uri.as_str()).unwrap()),
        client_info: None,
        initialization_options: None,
        capabilities: ClientCapabilities{workspace: None, text_document: None, experimental: None, window: None},
        trace: None,
        workspace_folders: None,
    };

    let on_response = |resp: Option<Response>| {
        assert!(resp.is_some());
        let respu = resp.unwrap();
        match respu.result_or_error {
            ResponseResult::Result(_) => {},
            ResponseResult::Error(e) => { panic!(format!("expected ResponseResult::Result(..), got {:?}", e)) }
        }
    };
    
    let completable = MethodCompletable::new(ResponseCompletable::new(Some(Id::Number(1)), Box::new(on_response)));
    server.initialize(initialize_params, completable);

    assert_eq!(server.root, Some(String::from(tmp_path)));

    assert_eq!(server.graph.borrow().graph.edge_count(), 0);
    assert_eq!(server.graph.borrow().graph.node_count(), 0);

    assert_eq!(format!("{:?}", server.wait), "WaitGroup { count: 1 }");

    server.endpoint.request_shutdown();
    //std::thread::sleep(std::time::Duration::from_secs(1));
}

#[test]
fn test_01_initialize() {
    let mut server = new_temp_server();

    let tmp_dir = TempDir::new("mcshader").unwrap();
    fs::create_dir(tmp_dir.path().join("shaders")).unwrap();

    let opts = &fs_extra::dir::CopyOptions::new();
    let files = fs::read_dir("./testdata/01").unwrap().map(|e| String::from(e.unwrap().path().as_os_str().to_str().unwrap())).collect::<Vec<String>>();
    fs_extra::copy_items(&files, tmp_dir.path().join("shaders"), opts).unwrap();

    let tmp_path = tmp_dir.path().as_os_str().to_str().unwrap();
    let tmp_uri = format!("{}{}", "file://", tmp_path);

    let initialize_params = InitializeParams{
        process_id: None,
        root_path: None,
        root_uri: Some(Url::parse(tmp_uri.as_str()).unwrap()),
        client_info: None,
        initialization_options: None,
        capabilities: ClientCapabilities{workspace: None, text_document: None, experimental: None, window: None},
        trace: None,
        workspace_folders: None,
    };

    let on_response = |resp: Option<Response>| {
        assert!(resp.is_some());
        let respu = resp.unwrap();
        match respu.result_or_error {
            ResponseResult::Result(_) => {},
            ResponseResult::Error(e) => { panic!(format!("expected ResponseResult::Result(..), got {:?}", e)) }
        }
    };
    
    let completable = MethodCompletable::new(ResponseCompletable::new(Some(Id::Number(1)), Box::new(on_response)));
    server.initialize(initialize_params, completable);

    assert_eq!(server.graph.borrow().graph.edge_count(), 1);
    let edge = server.graph.borrow().graph.edge_indices().next().unwrap();
    let (node1, node2) = server.graph.borrow().graph.edge_endpoints(edge).unwrap();
    assert_eq!(server.graph.borrow().graph[node1].as_str(), tmp_dir.path().join("shaders").join("final.fsh").as_os_str().to_str().unwrap());

    server.endpoint.request_shutdown();
}