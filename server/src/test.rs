use super::*;
use std::io;
use std::io::Result;

use jsonrpc_common::*;
use jsonrpc_response::*;
use jsonrpc_response::*;

use serde_json::json;

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
        graph: Rc::new(RefCell::new(Graph::default())),
        config: Configuration::default(),
        wait: WaitGroup::new(),
        root: None,
        command_provider: None,
    }
}

#[test]
fn test_initialize() {
    let mut server = new_temp_server();

    let initialize_params = InitializeParams{
        process_id: None,
        root_path: Some(String::from("/tmp/vscodemcshader")),
        root_uri: None,
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

    assert_eq!(server.root, Some(String::from("/tmp/vscodemcshader")));

    assert_eq!(server.graph.borrow().edge_count(), 0);
    assert_eq!(server.graph.borrow().node_count(), 0);

    assert_eq!(format!("{:?}", server.wait), "WaitGroup { count: 1 }");

    server.endpoint.request_shutdown();
    //std::thread::sleep(std::time::Duration::from_secs(1));
}