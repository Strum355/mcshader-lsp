use super::*;
use std::fs;
use std::io;
use std::io::Result;

use hamcrest2::prelude::*;
use pretty_assertions::assert_eq;

use slog::o;
use slog::Logger;
use tempdir::TempDir;

use petgraph::algo::is_cyclic_directed;

use fs_extra::{copy_items, dir};

use jsonrpc_common::*;
use jsonrpc_response::*;

struct StdoutNewline {
    s: Box<dyn io::Write>,
}

impl io::Write for StdoutNewline {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let res = self.s.write(buf);
        if buf[buf.len() - 1] == b"}"[0] {
            #[allow(unused_variables)]
            let res = self.s.write(b"\n\n");
        }
        res
    }

    fn flush(&mut self) -> Result<()> {
        self.s.flush()
    }
}

fn new_temp_server(opengl_context: Option<Box<dyn opengl::ShaderValidator>>) -> MinecraftShaderLanguageServer {
    let endpoint = LSPEndpoint::create_lsp_output_with_output_stream(|| StdoutNewline { s: Box::new(io::sink()) });

    let context = opengl_context.unwrap_or_else(|| Box::new(opengl::MockShaderValidator::new()));
    let logger = Logger::root(slog::Discard, o!());
    let guard = slog_scope::set_global_logger(logger);

    MinecraftShaderLanguageServer {
        endpoint,
        graph: Rc::new(RefCell::new(graph::CachedStableGraph::new())),
        root: "".into(),
        command_provider: None,
        opengl_context: context.into(),
        log_guard: Some(guard),
        tree_sitter: Rc::new(RefCell::new(Parser::new())),
    }
}

fn copy_files(files: &str, dest: &TempDir) {
    let opts = &dir::CopyOptions::new();
    let files = fs::read_dir(files)
        .unwrap()
        .map(|e| String::from(e.unwrap().path().to_str().unwrap()))
        .collect::<Vec<String>>();
    copy_items(&files, dest.path().join("shaders"), opts).unwrap();
}

fn copy_to_and_set_root(test_path: &str, server: &mut MinecraftShaderLanguageServer) -> (Rc<TempDir>, PathBuf) {
    let (_tmp_dir, tmp_path) = copy_to_tmp_dir(test_path);

    server.root = tmp_path.clone(); //format!("{}{}", "file://", tmp_path);

    (_tmp_dir, tmp_path)
}

fn copy_to_tmp_dir(test_path: &str) -> (Rc<TempDir>, PathBuf) {
    let tmp_dir = Rc::new(TempDir::new("mcshader").unwrap());
    fs::create_dir(tmp_dir.path().join("shaders")).unwrap();

    copy_files(test_path, &tmp_dir);

    let tmp_clone = tmp_dir.clone();
    let tmp_path = tmp_clone.path().to_str().unwrap();

    (tmp_dir, tmp_path.into())
}

#[allow(deprecated)]
#[test]
fn test_empty_initialize() {
    let mut server = new_temp_server(None);

    let tmp_dir = TempDir::new("mcshader").unwrap();
    let tmp_path = tmp_dir.path();

    let initialize_params = InitializeParams {
        process_id: None,
        root_path: None,
        root_uri: Some(Url::from_directory_path(tmp_path).unwrap()),
        client_info: None,
        initialization_options: None,
        capabilities: ClientCapabilities {
            workspace: None,
            text_document: None,
            experimental: None,
            window: None,
            general: Option::None,
        },
        trace: None,
        workspace_folders: None,
        locale: Option::None,
    };

    let on_response = |resp: Option<Response>| {
        assert!(resp.is_some());
        let respu = resp.unwrap();
        match respu.result_or_error {
            ResponseResult::Result(_) => {}
            ResponseResult::Error(e) => {
                panic!("expected ResponseResult::Result(..), got {:?}", e)
            }
        }
    };

    let completable = MethodCompletable::new(ResponseCompletable::new(Some(Id::Number(1)), Box::new(on_response)));
    server.initialize(initialize_params, completable);

    assert_eq!(server.root, tmp_path);

    assert_eq!(server.graph.borrow().graph.edge_count(), 0);
    assert_eq!(server.graph.borrow().graph.node_count(), 0);

    server.endpoint.request_shutdown();
}

#[allow(deprecated)]
#[test]
fn test_01_initialize() {
    let mut server = new_temp_server(None);

    let (_tmp_dir, tmp_path) = copy_to_tmp_dir("./testdata/01");

    let initialize_params = InitializeParams {
        process_id: None,
        root_path: None,
        root_uri: Some(Url::from_directory_path(tmp_path.clone()).unwrap()),
        client_info: None,
        initialization_options: None,
        capabilities: ClientCapabilities {
            workspace: None,
            text_document: None,
            experimental: None,
            window: None,
            general: Option::None,
        },
        trace: None,
        workspace_folders: None,
        locale: Option::None,
    };

    let on_response = |resp: Option<Response>| {
        assert!(resp.is_some());
        let respu = resp.unwrap();
        match respu.result_or_error {
            ResponseResult::Result(_) => {}
            ResponseResult::Error(e) => {
                panic!("expected ResponseResult::Result(..), got {:?}", e)
            }
        }
    };

    let completable = MethodCompletable::new(ResponseCompletable::new(Some(Id::Number(1)), Box::new(on_response)));
    server.initialize(initialize_params, completable);
    server.endpoint.request_shutdown();

    // Assert there is one edge between two nodes
    assert_eq!(server.graph.borrow().graph.edge_count(), 1);

    let edge = server.graph.borrow().graph.edge_indices().next().unwrap();
    let (node1, node2) = server.graph.borrow().graph.edge_endpoints(edge).unwrap();

    // Assert the values of the two nodes in the tree
    assert_eq!(
        server.graph.borrow().graph[node1],
        //format!("{:?}/{}/{}", tmp_path, "shaders", "final.fsh")
        tmp_path.join("shaders").join("final.fsh").to_str().unwrap().to_string()
    );
    assert_eq!(
        server.graph.borrow().graph[node2],
        //format!("{:?}/{}/{}", tmp_path, "shaders", "common.glsl")
        tmp_path.join("shaders").join("common.glsl").to_str().unwrap().to_string()
    );

    assert_eq!(server.graph.borrow().graph.edge_weight(edge).unwrap().line, 2);
}

#[test]
fn test_05_initialize() {
    let mut server = new_temp_server(None);

    let (_tmp_dir, tmp_path) = copy_to_tmp_dir("./testdata/05");

    let initialize_params = InitializeParams {
        process_id: None,
        root_path: None,
        root_uri: Some(Url::from_directory_path(tmp_path.clone()).unwrap()),
        client_info: None,
        initialization_options: None,
        capabilities: ClientCapabilities {
            workspace: None,
            text_document: None,
            experimental: None,
            window: None,
            general: Option::None,
        },
        trace: None,
        workspace_folders: None,
        locale: Option::None,
    };

    let on_response = |resp: Option<Response>| {
        assert!(resp.is_some());
        let respu = resp.unwrap();
        match respu.result_or_error {
            ResponseResult::Result(_) => {}
            ResponseResult::Error(e) => {
                panic!("expected ResponseResult::Result(..), got {:?}", e)
            }
        }
    };

    let completable = MethodCompletable::new(ResponseCompletable::new(Some(Id::Number(1)), Box::new(on_response)));
    server.initialize(initialize_params, completable);
    server.endpoint.request_shutdown();

    // Assert there is one edge between two nodes
    assert_eq!(server.graph.borrow().graph.edge_count(), 3);

    assert_eq!(server.graph.borrow().graph.node_count(), 4);

    let pairs: HashSet<(PathBuf, PathBuf)> = vec![
        (
            tmp_path.join("shaders").join("final.fsh").to_str().unwrap().to_string().into(),
            tmp_path.join("shaders").join("common.glsl").to_str().unwrap().to_string().into(),
        ),
        (
            tmp_path.join("shaders").join("final.fsh").to_str().unwrap().to_string().into(),
            tmp_path
                .join("shaders")
                .join("test")
                .join("banana.glsl")
                .to_str()
                .unwrap()
                .to_string()
                .into(),
        ),
        (
            tmp_path
                .join("shaders")
                .join("test")
                .join("banana.glsl")
                .to_str()
                .unwrap()
                .to_string()
                .into(),
            tmp_path
                .join("shaders")
                .join("test")
                .join("burger.glsl")
                .to_str()
                .unwrap()
                .to_string()
                .into(),
        ),
    ]
    .into_iter()
    .collect();

    for edge in server.graph.borrow().graph.edge_indices() {
        let endpoints = server.graph.borrow().graph.edge_endpoints(edge).unwrap();
        let first = server.graph.borrow().get_node(endpoints.0);
        let second = server.graph.borrow().get_node(endpoints.1);
        let contains = pairs.contains(&(first.clone(), second.clone()));
        assert!(contains, "doesn't contain ({:?}, {:?})", first, second);
    }
}

#[test]
fn test_graph_two_connected_nodes() {
    let mut graph = graph::CachedStableGraph::new();

    let idx1 = graph.add_node(&PathBuf::from("sample"));
    let idx2 = graph.add_node(&PathBuf::from("banana"));
    graph.add_edge(idx1, idx2, IncludePosition { line: 3, start: 0, end: 0 });

    let children = graph.child_node_names(idx1);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], Into::<PathBuf>::into("banana".to_string()));

    let children = graph.child_node_indexes(idx1);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], idx2);

    let parents = graph.parent_node_names(idx1);
    assert_eq!(parents.len(), 0);

    let parents = graph.parent_node_names(idx2);
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0], Into::<PathBuf>::into("sample".to_string()));

    let parents = graph.parent_node_indexes(idx2);
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0], idx1);

    let ancestors = graph.collect_root_ancestors(idx2);
    assert_eq!(ancestors.len(), 1);
    assert_eq!(ancestors[0], idx1);

    let ancestors = graph.collect_root_ancestors(idx1);
    assert_eq!(ancestors.len(), 0);

    graph.remove_node(&PathBuf::from("sample"));
    assert_eq!(graph.graph.node_count(), 1);
    assert!(graph.find_node(&PathBuf::from("sample")).is_none());
    let neighbors = graph.child_node_names(idx2);
    assert_eq!(neighbors.len(), 0);
}

#[test]
fn test_collect_root_ancestors() {
    {
        let mut graph = graph::CachedStableGraph::new();

        let idx0 = graph.add_node(&PathBuf::from("0"));
        let idx1 = graph.add_node(&PathBuf::from("1"));
        let idx2 = graph.add_node(&PathBuf::from("2"));
        let idx3 = graph.add_node(&PathBuf::from("3"));

        graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
        graph.add_edge(idx1, idx2, IncludePosition { line: 3, start: 0, end: 0 });
        graph.add_edge(idx3, idx1, IncludePosition { line: 4, start: 0, end: 0 });

        //       0  3
        //       |/
        //       1
        //       |
        //       2

        let roots = graph.collect_root_ancestors(idx2);
        assert_eq!(roots, vec![idx3, idx0]);

        let roots = graph.collect_root_ancestors(idx1);
        assert_eq!(roots, vec![idx3, idx0]);

        let roots = graph.collect_root_ancestors(idx0);
        assert_eq!(roots, vec![]);

        let roots = graph.collect_root_ancestors(idx3);
        assert_eq!(roots, vec![]);
    }
    {
        let mut graph = graph::CachedStableGraph::new();

        let idx0 = graph.add_node(&PathBuf::from("0"));
        let idx1 = graph.add_node(&PathBuf::from("1"));
        let idx2 = graph.add_node(&PathBuf::from("2"));
        let idx3 = graph.add_node(&PathBuf::from("3"));

        graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
        graph.add_edge(idx0, idx2, IncludePosition { line: 3, start: 0, end: 0 });
        graph.add_edge(idx1, idx3, IncludePosition { line: 5, start: 0, end: 0 });

        //       0
        //      / \
        //     1   2
        //    /
        //   3

        let roots = graph.collect_root_ancestors(idx3);
        assert_eq!(roots, vec![idx0]);

        let roots = graph.collect_root_ancestors(idx2);
        assert_eq!(roots, vec![idx0]);

        let roots = graph.collect_root_ancestors(idx1);
        assert_eq!(roots, vec![idx0]);

        let roots = graph.collect_root_ancestors(idx0);
        assert_eq!(roots, vec![]);
    }
    {
        let mut graph = graph::CachedStableGraph::new();

        let idx0 = graph.add_node(&PathBuf::from("0"));
        let idx1 = graph.add_node(&PathBuf::from("1"));
        let idx2 = graph.add_node(&PathBuf::from("2"));
        let idx3 = graph.add_node(&PathBuf::from("3"));

        graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
        graph.add_edge(idx2, idx3, IncludePosition { line: 3, start: 0, end: 0 });
        graph.add_edge(idx1, idx3, IncludePosition { line: 5, start: 0, end: 0 });

        //       0
        //       |
        //       1
        //       \
        //     2  \
        //      \ /
        //       3

        let roots = graph.collect_root_ancestors(idx3);
        assert_eq!(roots, vec![idx0, idx2]);

        let roots = graph.collect_root_ancestors(idx2);
        assert_eq!(roots, vec![]);

        let roots = graph.collect_root_ancestors(idx1);
        assert_eq!(roots, vec![idx0]);

        let roots = graph.collect_root_ancestors(idx0);
        assert_eq!(roots, vec![]);
    }
    {
        let mut graph = graph::CachedStableGraph::new();

        let idx0 = graph.add_node(&PathBuf::from("0"));
        let idx1 = graph.add_node(&PathBuf::from("1"));
        let idx2 = graph.add_node(&PathBuf::from("2"));
        let idx3 = graph.add_node(&PathBuf::from("3"));

        graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
        graph.add_edge(idx1, idx2, IncludePosition { line: 4, start: 0, end: 0 });
        graph.add_edge(idx1, idx3, IncludePosition { line: 6, start: 0, end: 0 });

        //       0
        //       |
        //       1
        //      / \
        //     2   3

        let roots = graph.collect_root_ancestors(idx3);
        assert_eq!(roots, vec![idx0]);

        let roots = graph.collect_root_ancestors(idx2);
        assert_eq!(roots, vec![idx0]);

        let roots = graph.collect_root_ancestors(idx1);
        assert_eq!(roots, vec![idx0]);

        let roots = graph.collect_root_ancestors(idx0);
        assert_eq!(roots, vec![]);
    }
}

#[test]
fn test_graph_dfs() {
    {
        let mut graph = graph::CachedStableGraph::new();

        let idx0 = graph.add_node(&PathBuf::from("0"));
        let idx1 = graph.add_node(&PathBuf::from("1"));
        let idx2 = graph.add_node(&PathBuf::from("2"));
        let idx3 = graph.add_node(&PathBuf::from("3"));

        graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
        graph.add_edge(idx0, idx2, IncludePosition { line: 3, start: 0, end: 0 });
        graph.add_edge(idx1, idx3, IncludePosition { line: 5, start: 0, end: 0 });

        let dfs = dfs::Dfs::new(&graph, idx0);

        let mut collection = Vec::new();

        for i in dfs {
            assert_that!(&i, ok());
            collection.push(i.unwrap());
        }

        let nodes: Vec<NodeIndex> = collection.iter().map(|n| n.0).collect();
        let parents: Vec<Option<NodeIndex>> = collection.iter().map(|n| n.1).collect();
        //          0
        //        /  \
        //      1     2
        //     /
        //    3
        let expected_nodes = vec![idx0, idx1, idx3, idx2];

        assert_eq!(expected_nodes, nodes);

        let expected_parents = vec![None, Some(idx0), Some(idx1), Some(idx0)];

        assert_eq!(expected_parents, parents);

        assert!(!is_cyclic_directed(&graph.graph));
    }
    {
        let mut graph = graph::CachedStableGraph::new();

        let idx0 = graph.add_node(&PathBuf::from("0"));
        let idx1 = graph.add_node(&PathBuf::from("1"));
        let idx2 = graph.add_node(&PathBuf::from("2"));
        let idx3 = graph.add_node(&PathBuf::from("3"));
        let idx4 = graph.add_node(&PathBuf::from("4"));
        let idx5 = graph.add_node(&PathBuf::from("5"));
        let idx6 = graph.add_node(&PathBuf::from("6"));
        let idx7 = graph.add_node(&PathBuf::from("7"));

        graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
        graph.add_edge(idx0, idx2, IncludePosition { line: 3, start: 0, end: 0 });
        graph.add_edge(idx1, idx3, IncludePosition { line: 5, start: 0, end: 0 });
        graph.add_edge(idx1, idx4, IncludePosition { line: 6, start: 0, end: 0 });
        graph.add_edge(idx2, idx4, IncludePosition { line: 5, start: 0, end: 0 });
        graph.add_edge(idx2, idx5, IncludePosition { line: 4, start: 0, end: 0 });
        graph.add_edge(idx3, idx6, IncludePosition { line: 4, start: 0, end: 0 });
        graph.add_edge(idx4, idx6, IncludePosition { line: 4, start: 0, end: 0 });
        graph.add_edge(idx6, idx7, IncludePosition { line: 4, start: 0, end: 0 });

        let dfs = dfs::Dfs::new(&graph, idx0);

        let mut collection = Vec::new();

        for i in dfs {
            assert_that!(&i, ok());
            collection.push(i.unwrap());
        }

        let nodes: Vec<NodeIndex> = collection.iter().map(|n| n.0).collect();
        let parents: Vec<Option<NodeIndex>> = collection.iter().map(|n| n.1).collect();
        //          0
        //        /  \
        //      1     2
        //     / \   / \
        //    3    4    5
        //     \ /
        //      6 - 7
        let expected_nodes = vec![idx0, idx1, idx3, idx6, idx7, idx4, idx6, idx7, idx2, idx5, idx4, idx6, idx7];

        assert_eq!(expected_nodes, nodes);

        let expected_parents = vec![
            None,
            Some(idx0),
            Some(idx1),
            Some(idx3),
            Some(idx6),
            Some(idx1),
            Some(idx4),
            Some(idx6),
            Some(idx0),
            Some(idx2),
            Some(idx2),
            Some(idx4),
            Some(idx6),
        ];

        assert_eq!(expected_parents, parents);

        assert!(!is_cyclic_directed(&graph.graph));
    }
}

#[test]
fn test_graph_dfs_cycle() {
    {
        let mut graph = graph::CachedStableGraph::new();

        let idx0 = graph.add_node(&PathBuf::from("0"));
        let idx1 = graph.add_node(&PathBuf::from("1"));
        let idx2 = graph.add_node(&PathBuf::from("2"));
        let idx3 = graph.add_node(&PathBuf::from("3"));
        let idx4 = graph.add_node(&PathBuf::from("4"));
        let idx5 = graph.add_node(&PathBuf::from("5"));
        let idx6 = graph.add_node(&PathBuf::from("6"));
        let idx7 = graph.add_node(&PathBuf::from("7"));

        graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
        graph.add_edge(idx0, idx2, IncludePosition { line: 3, start: 0, end: 0 });
        graph.add_edge(idx1, idx3, IncludePosition { line: 5, start: 0, end: 0 });
        graph.add_edge(idx1, idx4, IncludePosition { line: 6, start: 0, end: 0 });
        graph.add_edge(idx2, idx4, IncludePosition { line: 5, start: 0, end: 0 });
        graph.add_edge(idx2, idx5, IncludePosition { line: 4, start: 0, end: 0 });
        graph.add_edge(idx3, idx6, IncludePosition { line: 4, start: 0, end: 0 });
        graph.add_edge(idx4, idx6, IncludePosition { line: 4, start: 0, end: 0 });
        graph.add_edge(idx6, idx7, IncludePosition { line: 4, start: 0, end: 0 });
        graph.add_edge(idx7, idx4, IncludePosition { line: 4, start: 0, end: 0 });

        let mut dfs = dfs::Dfs::new(&graph, idx0);

        for _ in 0..5 {
            if let Some(i) = dfs.next() {
                assert_that!(&i, ok());
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
        assert_that!(next, err());
    }
    {
        let mut graph = graph::CachedStableGraph::new();

        let idx0 = graph.add_node(&PathBuf::from("0"));
        let idx1 = graph.add_node(&PathBuf::from("1"));

        graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
        graph.add_edge(idx1, idx0, IncludePosition { line: 2, start: 0, end: 0 });

        let mut dfs = dfs::Dfs::new(&graph, idx1);

        println!("{:?}", dfs.next());
        println!("{:?}", dfs.next());
        println!("{:?}", dfs.next());
    }
}

#[test]
fn test_generate_merge_list_01() {
    let mut server = new_temp_server(None);

    let (_tmp_dir, tmp_path) = copy_to_and_set_root("./testdata/01", &mut server);
    server.endpoint.request_shutdown();

    let final_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{:?}/shaders/final.fsh", tmp_path).try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("final.fsh"));
    let common_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{:?}/shaders/common.glsl", tmp_path).try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("common.glsl"));

    server
        .graph
        .borrow_mut()
        .add_edge(final_idx, common_idx, IncludePosition { line: 2, start: 0, end: 0 });

    let nodes = server.get_dfs_for_node(final_idx).unwrap();
    let sources = server.load_sources(&nodes).unwrap();

    let graph_borrow = server.graph.borrow();
    let result = merge_views::generate_merge_list(&nodes, &sources, &graph_borrow);

    let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

    let mut truth = fs::read_to_string(merge_file).unwrap();
    truth = truth.replacen(
        "!!",
        &tmp_path.join("shaders").join("common.glsl").to_str().unwrap().replace('\\', "\\\\"),
        1,
    );
    truth = truth.replace(
        "!!",
        &tmp_path.join("shaders").join("final.fsh").to_str().unwrap().replace('\\', "\\\\"),
    );

    assert_eq!(result, truth);
}

#[test]
fn test_generate_merge_list_02() {
    let mut server = new_temp_server(None);

    let (_tmp_dir, tmp_path) = copy_to_and_set_root("./testdata/02", &mut server);
    server.endpoint.request_shutdown();

    let final_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/{}", tmp_path, "final.fsh").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("final.fsh"));
    let test_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/utils/{}", tmp_path, "test.glsl").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("utils").join("test.glsl"));
    let burger_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/utils/{}", tmp_path, "burger.glsl").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("utils").join("burger.glsl"));
    let sample_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/utils/{}", tmp_path, "sample.glsl").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("utils").join("sample.glsl"));

    server
        .graph
        .borrow_mut()
        .add_edge(final_idx, sample_idx, IncludePosition { line: 2, start: 0, end: 0 });
    server
        .graph
        .borrow_mut()
        .add_edge(sample_idx, burger_idx, IncludePosition { line: 4, start: 0, end: 0 });
    server
        .graph
        .borrow_mut()
        .add_edge(sample_idx, test_idx, IncludePosition { line: 6, start: 0, end: 0 });

    let nodes = server.get_dfs_for_node(final_idx).unwrap();
    let sources = server.load_sources(&nodes).unwrap();

    let graph_borrow = server.graph.borrow();
    let result = merge_views::generate_merge_list(&nodes, &sources, &graph_borrow);

    let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

    let mut truth = fs::read_to_string(merge_file).unwrap();

    for file in &["sample.glsl", "burger.glsl", "sample.glsl", "test.glsl", "sample.glsl"] {
        let path = tmp_path.clone();
        truth = truth.replacen(
            "!!",
            &path
                .join("shaders")
                .join("utils")
                .join(file)
                .to_str()
                .unwrap()
                .replace('\\', "\\\\"),
            1,
        );
    }
    truth = truth.replacen(
        "!!",
        &tmp_path.join("shaders").join("final.fsh").to_str().unwrap().replace('\\', "\\\\"),
        1,
    );

    assert_eq!(result, truth);
}

#[test]
fn test_generate_merge_list_03() {
    let mut server = new_temp_server(None);

    let (_tmp_dir, tmp_path) = copy_to_and_set_root("./testdata/03", &mut server);
    server.endpoint.request_shutdown();

    let final_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/{}", tmp_path, "final.fsh").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("final.fsh"));
    let test_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/utils/{}", tmp_path, "test.glsl").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("utils").join("test.glsl"));
    let burger_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/utils/{}", tmp_path, "burger.glsl").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("utils").join("burger.glsl"));
    let sample_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/utils/{}", tmp_path, "sample.glsl").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("utils").join("sample.glsl"));

    server
        .graph
        .borrow_mut()
        .add_edge(final_idx, sample_idx, IncludePosition { line: 2, start: 0, end: 0 });
    server
        .graph
        .borrow_mut()
        .add_edge(sample_idx, burger_idx, IncludePosition { line: 4, start: 0, end: 0 });
    server
        .graph
        .borrow_mut()
        .add_edge(sample_idx, test_idx, IncludePosition { line: 6, start: 0, end: 0 });

    let nodes = server.get_dfs_for_node(final_idx).unwrap();
    let sources = server.load_sources(&nodes).unwrap();

    let graph_borrow = server.graph.borrow();
    let result = merge_views::generate_merge_list(&nodes, &sources, &graph_borrow);

    let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

    let mut truth = fs::read_to_string(merge_file).unwrap();

    for file in &["sample.glsl", "burger.glsl", "sample.glsl", "test.glsl", "sample.glsl"] {
        let path = tmp_path.clone();
        truth = truth.replacen(
            "!!",
            &path
                .join("shaders")
                .join("utils")
                .join(file)
                .to_str()
                .unwrap()
                .replace('\\', "\\\\"),
            1,
        );
    }
    truth = truth.replacen(
        "!!",
        &tmp_path.join("shaders").join("final.fsh").to_str().unwrap().replace('\\', "\\\\"),
        1,
    );

    assert_eq!(result, truth);
}

#[test]
fn test_generate_merge_list_04() {
    let mut server = new_temp_server(None);

    let (_tmp_dir, tmp_path) = copy_to_and_set_root("./testdata/04", &mut server);
    server.endpoint.request_shutdown();

    let final_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/{}", tmp_path, "final.fsh").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("final.fsh"));
    let utilities_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/utils/{}", tmp_path, "utilities.glsl").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("utils").join("utilities.glsl"));
    let stuff1_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/utils/{}", tmp_path, "stuff1.glsl").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("utils").join("stuff1.glsl"));
    let stuff2_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/utils/{}", tmp_path, "stuff2.glsl").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("utils").join("stuff2.glsl"));
    let matrices_idx = server
        .graph
        .borrow_mut()
        //.add_node(&format!("{}/shaders/lib/{}", tmp_path, "matrices.glsl").try_into().unwrap());
        .add_node(&tmp_path.join("shaders").join("lib").join("matrices.glsl"));

    server
        .graph
        .borrow_mut()
        .add_edge(final_idx, utilities_idx, IncludePosition { line: 2, start: 0, end: 0 });
    server
        .graph
        .borrow_mut()
        .add_edge(utilities_idx, stuff1_idx, IncludePosition { line: 0, start: 0, end: 0 });
    server
        .graph
        .borrow_mut()
        .add_edge(utilities_idx, stuff2_idx, IncludePosition { line: 1, start: 0, end: 0 });
    server
        .graph
        .borrow_mut()
        .add_edge(final_idx, matrices_idx, IncludePosition { line: 3, start: 0, end: 0 });

    let nodes = server.get_dfs_for_node(final_idx).unwrap();
    let sources = server.load_sources(&nodes).unwrap();

    let graph_borrow = server.graph.borrow();
    let result = merge_views::generate_merge_list(&nodes, &sources, &graph_borrow);

    let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

    let mut truth = fs::read_to_string(merge_file).unwrap();

    for file in &[
        PathBuf::new().join("utils").join("utilities.glsl").to_str().unwrap(),
        PathBuf::new().join("utils").join("stuff1.glsl").to_str().unwrap(),
        PathBuf::new().join("utils").join("utilities.glsl").to_str().unwrap(),
        PathBuf::new().join("utils").join("stuff2.glsl").to_str().unwrap(),
        PathBuf::new().join("utils").join("utilities.glsl").to_str().unwrap(),
        PathBuf::new().join("final.fsh").to_str().unwrap(),
        PathBuf::new().join("lib").join("matrices.glsl").to_str().unwrap(),
        PathBuf::new().join("final.fsh").to_str().unwrap(),
    ] {
        let path = tmp_path.clone();
        //path.f
        truth = truth.replacen("!!", &path.join("shaders").join(file).to_str().unwrap().replace('\\', "\\\\"), 1);
    }

    assert_eq!(result, truth);
}

#[test]
fn test_nvidia_diagnostics() {
    let mut mockgl = opengl::MockShaderValidator::new();
    mockgl.expect_vendor().returning(|| "NVIDIA".into());
    let server = new_temp_server(Some(Box::new(mockgl)));

    let output = "/home/noah/.minecraft/shaderpacks/test/shaders/final.fsh(9) : error C0000: syntax error, unexpected '}', expecting ',' or ';' at token \"}\"";

    let results = diagnostics_parser::parse_diagnostics_output(
        output.to_string(),
        &PathBuf::from_str("/home/noah/.minecraft/shaderpacks/test").unwrap(),
        server.opengl_context.as_ref(),
    );

    assert_eq!(results.len(), 1);
    let first = results.into_iter().next().unwrap();
    assert_eq!(
        first.0,
        url::Url::from_file_path("/home/noah/.minecraft/shaderpacks/test/shaders/final.fsh").unwrap()
    );
    server.endpoint.request_shutdown();
}

#[test]
fn test_amd_diagnostics() {
    let mut mockgl = opengl::MockShaderValidator::new();
    mockgl.expect_vendor().returning(|| "ATI Technologies".into());
    let server = new_temp_server(Some(Box::new(mockgl)));

    let output = "ERROR: 0:1: '' : syntax error: #line
ERROR: 0:10: '' : syntax error: #line
ERROR: 0:15: 'varying' : syntax error: syntax error
";

    let results = diagnostics_parser::parse_diagnostics_output(
        output.to_string(),
        &PathBuf::from_str("/home/test").unwrap(),
        server.opengl_context.as_ref(),
    );
    assert_eq!(results.len(), 1);
    let first = results.into_iter().next().unwrap();
    assert_eq!(first.1.len(), 3);
    server.endpoint.request_shutdown();
}
