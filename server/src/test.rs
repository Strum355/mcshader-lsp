use super::*;
use std::io;
use std::io::Result;
use tempdir::TempDir;
use std::fs;

use fs_extra::{dir, copy_items};

use jsonrpc_common::*;
use jsonrpc_response::*;

struct StdoutNewline {
    s: io::Stdout
}

impl io::Write for StdoutNewline {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let res = self.s.write(buf);
        if buf[buf.len()-1] == b"}"[0] {
            #[allow(unused_variables)]
            let res = self.s.write(b"\n\n");
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

fn copy_files(files: &str, dest: &TempDir) {
    let opts = &dir::CopyOptions::new();
    let files = fs::read_dir(files).unwrap().map(|e| String::from(e.unwrap().path().as_os_str().to_str().unwrap())).collect::<Vec<String>>();
    copy_items(&files, dest.path().join("shaders"), opts).unwrap();
}

#[allow(deprecated)]
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

#[allow(deprecated)]
#[test]
fn test_01_initialize() {
    let mut server = new_temp_server();

    let tmp_dir = TempDir::new("mcshader").unwrap();
    fs::create_dir(tmp_dir.path().join("shaders")).unwrap();

    copy_files("./testdata/01", &tmp_dir);

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

    // Assert there is one edge between two nodes
    assert_eq!(server.graph.borrow().graph.edge_count(), 1);
    
    let edge = server.graph.borrow().graph.edge_indices().next().unwrap();
    let (node1, node2) = server.graph.borrow().graph.edge_endpoints(edge).unwrap();
    
    // Assert the values of the two nodes in the tree
    assert_eq!(server.graph.borrow().graph[node1].as_str(), tmp_dir.path().join("shaders").join("final.fsh").as_os_str().to_str().unwrap());
    assert_eq!(server.graph.borrow().graph[node2].as_str(), tmp_dir.path().join("shaders").join("common.glsl").as_os_str().to_str().unwrap());

    server.endpoint.request_shutdown();
}

#[test]
fn test_graph_two_connected_nodes() {
    let mut graph = graph::CachedStableGraph::new();

    let idx1 = graph.add_node("sample");
    let idx2 = graph.add_node("banana");
    graph.add_edge(idx1, idx2, 3, 10, 50);

    let children = graph.child_node_names(idx1);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], "banana");

    let children = graph.child_node_indexes(idx1);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], idx2);

    let parents = graph.parent_node_names(idx1);
    assert_eq!(parents.len(), 0);
    
    let parents = graph.parent_node_names(idx2);
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0], "sample");

    let parents = graph.parent_node_indexes(idx2);
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0], idx1);

    let ancestors = graph.collect_root_ancestors(idx2);
    assert_eq!(ancestors.len(), 1);
    assert_eq!(ancestors[0], idx1);

    let ancestors = graph.collect_root_ancestors(idx1);
    assert_eq!(ancestors.len(), 0);

    graph.remove_node("sample");
    assert_eq!(graph.graph.node_count(), 1);
    assert!(graph.find_node("sample").is_none());
    let neighbors = graph.child_node_names(idx2);
    assert_eq!(neighbors.len(), 0);
}

#[test]
fn test_collect_root_ancestors() {
    let mut graph = graph::CachedStableGraph::new();

    let idx0 = graph.add_node("0");
    let idx1 = graph.add_node("1");
    let idx2 = graph.add_node("2");

    graph.add_edge(idx0, idx1, 2, 0, 0);
    graph.add_edge(idx1, idx2, 3, 0, 0);
    graph.add_edge(idx2, idx0, 5, 0, 0);

    let roots = graph.collect_root_ancestors(idx0);
    assert_eq!(roots, vec![idx2]);
}

#[test]
fn test_graph_dfs() {
    {
        let mut graph = graph::CachedStableGraph::new();

        let idx0 = graph.add_node("0");
        let idx1 = graph.add_node("1");
        let idx2 = graph.add_node("2");
        let idx3 = graph.add_node("3");
        let idx4 = graph.add_node("4");
        let idx5 = graph.add_node("5");
        let idx6 = graph.add_node("6");
        let idx7 = graph.add_node("7");

        graph.add_edge(idx0, idx1, 2, 0, 0);
        graph.add_edge(idx0, idx2, 3, 0, 0);
        graph.add_edge(idx1, idx3, 5, 0, 0);
        graph.add_edge(idx1, idx4, 6, 0, 0);
        graph.add_edge(idx2, idx4, 5, 0, 0);
        graph.add_edge(idx2, idx5, 4, 0, 0);
        graph.add_edge(idx3, idx6, 4, 0, 0);
        graph.add_edge(idx4, idx6, 4, 0, 0);
        graph.add_edge(idx6, idx7, 4, 0, 0);

        let mut dfs = dfs::Dfs::new(&graph, idx0);

        let mut collection = Vec::new();

        while let Some(i) = dfs.next() {
            assert_eq!(i.is_ok(), true);
            collection.push(i.unwrap());
        }

        //          0
        //        /  \
        //      1     2
        //     / \   / \
        //    3    4    5
        //     \ /  
        //      6 - 7
        let expected = vec![idx0, idx1, idx3, idx6, idx7, idx4, idx6, idx7, idx2, idx5, idx4, idx6, idx7];
        println!("{:?}\n{:?}", expected, collection);
        
        collection.reverse();
        for i in expected {
            assert_eq!(i, collection.pop().unwrap());
        }

        assert!(!is_cyclic_directed(&graph.graph));
    }
}

#[test]
fn test_graph_dfs_cycle() {
    {
        let mut graph = graph::CachedStableGraph::new();

        let idx0 = graph.add_node("0");
        let idx1 = graph.add_node("1");
        let idx2 = graph.add_node("2");
        let idx3 = graph.add_node("3");
        let idx4 = graph.add_node("4");
        let idx5 = graph.add_node("5");
        let idx6 = graph.add_node("6");
        let idx7 = graph.add_node("7");

        graph.add_edge(idx0, idx1, 2, 0, 0);
        graph.add_edge(idx0, idx2, 3, 0, 0);
        graph.add_edge(idx1, idx3, 5, 0, 0);
        graph.add_edge(idx1, idx4, 6, 0, 0);
        graph.add_edge(idx2, idx4, 5, 0, 0);
        graph.add_edge(idx2, idx5, 4, 0, 0);
        graph.add_edge(idx3, idx6, 4, 0, 0);
        graph.add_edge(idx4, idx6, 4, 0, 0);
        graph.add_edge(idx6, idx7, 4, 0, 0);
        graph.add_edge(idx7, idx4, 4, 0, 0);

        let mut dfs = dfs::Dfs::new(&graph, idx0);

        for _ in 0..5 {
            if let Some(i) = dfs.next() {
                assert_eq!(i.is_ok(), true);
            }
        }
        
        //          0
        //        /  \
        //      1     2
        //     / \   / \
        //    3    4    5
        //     \ /  \
        //      6 - 7

        assert!(is_cyclic_directed(&graph.graph));

        let next = dfs.next().unwrap();
        assert_eq!(next.is_err(), true);
    }

    {
        let mut graph = graph::CachedStableGraph::new();

        let idx0 = graph.add_node("0");
        let idx1 = graph.add_node("1");

        graph.add_edge(idx0, idx1, 2, 0, 0);
        graph.add_edge(idx1, idx0, 2, 0, 0);

        let mut dfs = dfs::Dfs::new(&graph, idx1);

        println!("{:?}", dfs.next());
        println!("{:?}", dfs.next());
        println!("{:?}", dfs.next());
    }
}