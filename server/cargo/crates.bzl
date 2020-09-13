"""
@generated
cargo-raze crate workspace functions

DO NOT EDIT! Replaced on runs of cargo-raze
"""
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")

def _new_http_archive(name, **kwargs):
    if not native.existing_rule(name):
        http_archive(name=name, **kwargs)

def _new_git_repository(name, **kwargs):
    if not native.existing_rule(name):
        new_git_repository(name=name, **kwargs)

def server_fetch_remote_crates():

    _new_http_archive(
        name = "server__aho_corasick__0_7_13",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/aho-corasick/aho-corasick-0.7.13.crate",
        type = "tar.gz",
        strip_prefix = "aho-corasick-0.7.13",
        build_file = Label("//server/cargo/remote:aho-corasick-0.7.13.BUILD"),
    )

    _new_http_archive(
        name = "server__anyhow__1_0_32",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/anyhow/anyhow-1.0.32.crate",
        type = "tar.gz",
        strip_prefix = "anyhow-1.0.32",
        build_file = Label("//server/cargo/remote:anyhow-1.0.32.BUILD"),
    )

    _new_http_archive(
        name = "server__autocfg__1_0_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/autocfg/autocfg-1.0.1.crate",
        type = "tar.gz",
        strip_prefix = "autocfg-1.0.1",
        build_file = Label("//server/cargo/remote:autocfg-1.0.1.BUILD"),
    )

    _new_http_archive(
        name = "server__base64__0_12_3",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/base64/base64-0.12.3.crate",
        type = "tar.gz",
        strip_prefix = "base64-0.12.3",
        build_file = Label("//server/cargo/remote:base64-0.12.3.BUILD"),
    )

    _new_http_archive(
        name = "server__bit_set__0_5_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/bit-set/bit-set-0.5.2.crate",
        type = "tar.gz",
        strip_prefix = "bit-set-0.5.2",
        build_file = Label("//server/cargo/remote:bit-set-0.5.2.BUILD"),
    )

    _new_http_archive(
        name = "server__bit_vec__0_6_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/bit-vec/bit-vec-0.6.2.crate",
        type = "tar.gz",
        strip_prefix = "bit-vec-0.6.2",
        build_file = Label("//server/cargo/remote:bit-vec-0.6.2.BUILD"),
    )

    _new_http_archive(
        name = "server__bitflags__1_2_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/bitflags/bitflags-1.2.1.crate",
        type = "tar.gz",
        strip_prefix = "bitflags-1.2.1",
        build_file = Label("//server/cargo/remote:bitflags-1.2.1.BUILD"),
    )

    _new_http_archive(
        name = "server__cfg_if__0_1_10",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/cfg-if/cfg-if-0.1.10.crate",
        type = "tar.gz",
        strip_prefix = "cfg-if-0.1.10",
        build_file = Label("//server/cargo/remote:cfg-if-0.1.10.BUILD"),
    )

    _new_http_archive(
        name = "server__chan__0_1_23",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/chan/chan-0.1.23.crate",
        type = "tar.gz",
        strip_prefix = "chan-0.1.23",
        build_file = Label("//server/cargo/remote:chan-0.1.23.BUILD"),
    )

    _new_http_archive(
        name = "server__fixedbitset__0_2_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/fixedbitset/fixedbitset-0.2.0.crate",
        type = "tar.gz",
        strip_prefix = "fixedbitset-0.2.0",
        build_file = Label("//server/cargo/remote:fixedbitset-0.2.0.BUILD"),
    )

    _new_http_archive(
        name = "server__fs_extra__1_2_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/fs_extra/fs_extra-1.2.0.crate",
        type = "tar.gz",
        strip_prefix = "fs_extra-1.2.0",
        build_file = Label("//server/cargo/remote:fs_extra-1.2.0.BUILD"),
    )

    _new_http_archive(
        name = "server__fuchsia_cprng__0_1_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/fuchsia-cprng/fuchsia-cprng-0.1.1.crate",
        type = "tar.gz",
        strip_prefix = "fuchsia-cprng-0.1.1",
        build_file = Label("//server/cargo/remote:fuchsia-cprng-0.1.1.BUILD"),
    )

    _new_http_archive(
        name = "server__futures__0_1_29",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/futures/futures-0.1.29.crate",
        type = "tar.gz",
        strip_prefix = "futures-0.1.29",
        build_file = Label("//server/cargo/remote:futures-0.1.29.BUILD"),
    )

    _new_http_archive(
        name = "server__hamcrest2__0_3_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/hamcrest2/hamcrest2-0.3.0.crate",
        type = "tar.gz",
        strip_prefix = "hamcrest2-0.3.0",
        build_file = Label("//server/cargo/remote:hamcrest2-0.3.0.BUILD"),
    )

    _new_http_archive(
        name = "server__hashbrown__0_9_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/hashbrown/hashbrown-0.9.0.crate",
        type = "tar.gz",
        strip_prefix = "hashbrown-0.9.0",
        build_file = Label("//server/cargo/remote:hashbrown-0.9.0.BUILD"),
    )

    _new_http_archive(
        name = "server__idna__0_2_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/idna/idna-0.2.0.crate",
        type = "tar.gz",
        strip_prefix = "idna-0.2.0",
        build_file = Label("//server/cargo/remote:idna-0.2.0.BUILD"),
    )

    _new_http_archive(
        name = "server__indexmap__1_6_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/indexmap/indexmap-1.6.0.crate",
        type = "tar.gz",
        strip_prefix = "indexmap-1.6.0",
        build_file = Label("//server/cargo/remote:indexmap-1.6.0.BUILD"),
    )

    _new_http_archive(
        name = "server__itoa__0_4_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/itoa/itoa-0.4.6.crate",
        type = "tar.gz",
        strip_prefix = "itoa-0.4.6",
        build_file = Label("//server/cargo/remote:itoa-0.4.6.BUILD"),
    )

    _new_http_archive(
        name = "server__lazy_static__1_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/lazy_static/lazy_static-1.4.0.crate",
        type = "tar.gz",
        strip_prefix = "lazy_static-1.4.0",
        build_file = Label("//server/cargo/remote:lazy_static-1.4.0.BUILD"),
    )

    _new_http_archive(
        name = "server__libc__0_2_77",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/libc/libc-0.2.77.crate",
        type = "tar.gz",
        strip_prefix = "libc-0.2.77",
        build_file = Label("//server/cargo/remote:libc-0.2.77.BUILD"),
    )

    _new_http_archive(
        name = "server__log__0_4_11",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/log/log-0.4.11.crate",
        type = "tar.gz",
        strip_prefix = "log-0.4.11",
        build_file = Label("//server/cargo/remote:log-0.4.11.BUILD"),
    )

    _new_git_repository(
        name = "server__lsp_types__0_80_0",
        remote = "https://github.com/gluon-lang/lsp-types",
        commit = "881cbf1708ee67c71230a1629986ee3d39da27e9",
        build_file = Label("//server/cargo/remote:lsp-types-0.80.0.BUILD"),
        init_submodules = True,
    )

    _new_http_archive(
        name = "server__matches__0_1_8",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/matches/matches-0.1.8.crate",
        type = "tar.gz",
        strip_prefix = "matches-0.1.8",
        build_file = Label("//server/cargo/remote:matches-0.1.8.BUILD"),
    )

    _new_http_archive(
        name = "server__memchr__2_3_3",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/memchr/memchr-2.3.3.crate",
        type = "tar.gz",
        strip_prefix = "memchr-2.3.3",
        build_file = Label("//server/cargo/remote:memchr-2.3.3.BUILD"),
    )

    _new_http_archive(
        name = "server__num__0_2_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num/num-0.2.1.crate",
        type = "tar.gz",
        strip_prefix = "num-0.2.1",
        build_file = Label("//server/cargo/remote:num-0.2.1.BUILD"),
    )

    _new_http_archive(
        name = "server__num_bigint__0_2_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-bigint/num-bigint-0.2.6.crate",
        type = "tar.gz",
        strip_prefix = "num-bigint-0.2.6",
        build_file = Label("//server/cargo/remote:num-bigint-0.2.6.BUILD"),
    )

    _new_http_archive(
        name = "server__num_complex__0_2_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-complex/num-complex-0.2.4.crate",
        type = "tar.gz",
        strip_prefix = "num-complex-0.2.4",
        build_file = Label("//server/cargo/remote:num-complex-0.2.4.BUILD"),
    )

    _new_http_archive(
        name = "server__num_integer__0_1_43",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-integer/num-integer-0.1.43.crate",
        type = "tar.gz",
        strip_prefix = "num-integer-0.1.43",
        build_file = Label("//server/cargo/remote:num-integer-0.1.43.BUILD"),
    )

    _new_http_archive(
        name = "server__num_iter__0_1_41",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-iter/num-iter-0.1.41.crate",
        type = "tar.gz",
        strip_prefix = "num-iter-0.1.41",
        build_file = Label("//server/cargo/remote:num-iter-0.1.41.BUILD"),
    )

    _new_http_archive(
        name = "server__num_rational__0_2_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-rational/num-rational-0.2.4.crate",
        type = "tar.gz",
        strip_prefix = "num-rational-0.2.4",
        build_file = Label("//server/cargo/remote:num-rational-0.2.4.BUILD"),
    )

    _new_http_archive(
        name = "server__num_traits__0_2_12",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-traits/num-traits-0.2.12.crate",
        type = "tar.gz",
        strip_prefix = "num-traits-0.2.12",
        build_file = Label("//server/cargo/remote:num-traits-0.2.12.BUILD"),
    )

    _new_http_archive(
        name = "server__percent_encoding__2_1_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/percent-encoding/percent-encoding-2.1.0.crate",
        type = "tar.gz",
        strip_prefix = "percent-encoding-2.1.0",
        build_file = Label("//server/cargo/remote:percent-encoding-2.1.0.BUILD"),
    )

    _new_http_archive(
        name = "server__petgraph__0_5_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/petgraph/petgraph-0.5.1.crate",
        type = "tar.gz",
        strip_prefix = "petgraph-0.5.1",
        build_file = Label("//server/cargo/remote:petgraph-0.5.1.BUILD"),
    )

    _new_http_archive(
        name = "server__proc_macro2__1_0_21",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/proc-macro2/proc-macro2-1.0.21.crate",
        type = "tar.gz",
        strip_prefix = "proc-macro2-1.0.21",
        build_file = Label("//server/cargo/remote:proc-macro2-1.0.21.BUILD"),
    )

    _new_http_archive(
        name = "server__quote__1_0_7",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/quote/quote-1.0.7.crate",
        type = "tar.gz",
        strip_prefix = "quote-1.0.7",
        build_file = Label("//server/cargo/remote:quote-1.0.7.BUILD"),
    )

    _new_http_archive(
        name = "server__rand__0_3_23",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rand/rand-0.3.23.crate",
        type = "tar.gz",
        strip_prefix = "rand-0.3.23",
        build_file = Label("//server/cargo/remote:rand-0.3.23.BUILD"),
    )

    _new_http_archive(
        name = "server__rand__0_4_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rand/rand-0.4.6.crate",
        type = "tar.gz",
        strip_prefix = "rand-0.4.6",
        build_file = Label("//server/cargo/remote:rand-0.4.6.BUILD"),
    )

    _new_http_archive(
        name = "server__rand_core__0_3_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rand_core/rand_core-0.3.1.crate",
        type = "tar.gz",
        strip_prefix = "rand_core-0.3.1",
        build_file = Label("//server/cargo/remote:rand_core-0.3.1.BUILD"),
    )

    _new_http_archive(
        name = "server__rand_core__0_4_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rand_core/rand_core-0.4.2.crate",
        type = "tar.gz",
        strip_prefix = "rand_core-0.4.2",
        build_file = Label("//server/cargo/remote:rand_core-0.4.2.BUILD"),
    )

    _new_http_archive(
        name = "server__rdrand__0_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rdrand/rdrand-0.4.0.crate",
        type = "tar.gz",
        strip_prefix = "rdrand-0.4.0",
        build_file = Label("//server/cargo/remote:rdrand-0.4.0.BUILD"),
    )

    _new_http_archive(
        name = "server__regex__1_3_9",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/regex/regex-1.3.9.crate",
        type = "tar.gz",
        strip_prefix = "regex-1.3.9",
        build_file = Label("//server/cargo/remote:regex-1.3.9.BUILD"),
    )

    _new_http_archive(
        name = "server__regex_syntax__0_6_18",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/regex-syntax/regex-syntax-0.6.18.crate",
        type = "tar.gz",
        strip_prefix = "regex-syntax-0.6.18",
        build_file = Label("//server/cargo/remote:regex-syntax-0.6.18.BUILD"),
    )

    _new_http_archive(
        name = "server__remove_dir_all__0_5_3",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/remove_dir_all/remove_dir_all-0.5.3.crate",
        type = "tar.gz",
        strip_prefix = "remove_dir_all-0.5.3",
        build_file = Label("//server/cargo/remote:remove_dir_all-0.5.3.BUILD"),
    )

    _new_git_repository(
        name = "server__rust_lsp__0_6_0",
        remote = "https://github.com/Strum355/RustLSP",
        commit = "629507c387b479d5bdeb0a4eed9ef9aff34801ce",
        build_file = Label("//server/cargo/remote:rust_lsp-0.6.0.BUILD"),
        init_submodules = True,
    )

    _new_git_repository(
        name = "server__rustdt_json_rpc__0_3_0",
        remote = "https://github.com/Strum355/rustdt-json_rpc",
        commit = "e2394e9ca2be737de7fd70d4736bc667759b624b",
        build_file = Label("//server/cargo/remote:rustdt-json_rpc-0.3.0.BUILD"),
        init_submodules = True,
    )

    _new_http_archive(
        name = "server__rustdt_util__0_2_3",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rustdt_util/rustdt_util-0.2.3.crate",
        type = "tar.gz",
        strip_prefix = "rustdt_util-0.2.3",
        build_file = Label("//server/cargo/remote:rustdt_util-0.2.3.BUILD"),
    )

    _new_http_archive(
        name = "server__ryu__1_0_5",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/ryu/ryu-1.0.5.crate",
        type = "tar.gz",
        strip_prefix = "ryu-1.0.5",
        build_file = Label("//server/cargo/remote:ryu-1.0.5.BUILD"),
    )

    _new_http_archive(
        name = "server__same_file__1_0_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/same-file/same-file-1.0.6.crate",
        type = "tar.gz",
        strip_prefix = "same-file-1.0.6",
        build_file = Label("//server/cargo/remote:same-file-1.0.6.BUILD"),
    )

    _new_http_archive(
        name = "server__serde__1_0_116",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde/serde-1.0.116.crate",
        type = "tar.gz",
        strip_prefix = "serde-1.0.116",
        build_file = Label("//server/cargo/remote:serde-1.0.116.BUILD"),
    )

    _new_http_archive(
        name = "server__serde_derive__1_0_116",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde_derive/serde_derive-1.0.116.crate",
        type = "tar.gz",
        strip_prefix = "serde_derive-1.0.116",
        build_file = Label("//server/cargo/remote:serde_derive-1.0.116.BUILD"),
    )

    _new_http_archive(
        name = "server__serde_json__1_0_57",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde_json/serde_json-1.0.57.crate",
        type = "tar.gz",
        strip_prefix = "serde_json-1.0.57",
        build_file = Label("//server/cargo/remote:serde_json-1.0.57.BUILD"),
    )

    _new_http_archive(
        name = "server__serde_repr__0_1_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde_repr/serde_repr-0.1.6.crate",
        type = "tar.gz",
        strip_prefix = "serde_repr-0.1.6",
        build_file = Label("//server/cargo/remote:serde_repr-0.1.6.BUILD"),
    )

    _new_http_archive(
        name = "server__syn__1_0_40",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/syn/syn-1.0.40.crate",
        type = "tar.gz",
        strip_prefix = "syn-1.0.40",
        build_file = Label("//server/cargo/remote:syn-1.0.40.BUILD"),
    )

    _new_http_archive(
        name = "server__tempdir__0_3_7",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/tempdir/tempdir-0.3.7.crate",
        type = "tar.gz",
        strip_prefix = "tempdir-0.3.7",
        build_file = Label("//server/cargo/remote:tempdir-0.3.7.BUILD"),
    )

    _new_http_archive(
        name = "server__thiserror__1_0_20",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/thiserror/thiserror-1.0.20.crate",
        type = "tar.gz",
        strip_prefix = "thiserror-1.0.20",
        build_file = Label("//server/cargo/remote:thiserror-1.0.20.BUILD"),
    )

    _new_http_archive(
        name = "server__thiserror_impl__1_0_20",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/thiserror-impl/thiserror-impl-1.0.20.crate",
        type = "tar.gz",
        strip_prefix = "thiserror-impl-1.0.20",
        build_file = Label("//server/cargo/remote:thiserror-impl-1.0.20.BUILD"),
    )

    _new_http_archive(
        name = "server__thread_local__1_0_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/thread_local/thread_local-1.0.1.crate",
        type = "tar.gz",
        strip_prefix = "thread_local-1.0.1",
        build_file = Label("//server/cargo/remote:thread_local-1.0.1.BUILD"),
    )

    _new_http_archive(
        name = "server__tinyvec__0_3_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/tinyvec/tinyvec-0.3.4.crate",
        type = "tar.gz",
        strip_prefix = "tinyvec-0.3.4",
        build_file = Label("//server/cargo/remote:tinyvec-0.3.4.BUILD"),
    )

    _new_http_archive(
        name = "server__unicode_bidi__0_3_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/unicode-bidi/unicode-bidi-0.3.4.crate",
        type = "tar.gz",
        strip_prefix = "unicode-bidi-0.3.4",
        build_file = Label("//server/cargo/remote:unicode-bidi-0.3.4.BUILD"),
    )

    _new_http_archive(
        name = "server__unicode_normalization__0_1_13",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/unicode-normalization/unicode-normalization-0.1.13.crate",
        type = "tar.gz",
        strip_prefix = "unicode-normalization-0.1.13",
        build_file = Label("//server/cargo/remote:unicode-normalization-0.1.13.BUILD"),
    )

    _new_http_archive(
        name = "server__unicode_xid__0_2_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/unicode-xid/unicode-xid-0.2.1.crate",
        type = "tar.gz",
        strip_prefix = "unicode-xid-0.2.1",
        build_file = Label("//server/cargo/remote:unicode-xid-0.2.1.BUILD"),
    )

    _new_http_archive(
        name = "server__url__2_1_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/url/url-2.1.1.crate",
        type = "tar.gz",
        strip_prefix = "url-2.1.1",
        build_file = Label("//server/cargo/remote:url-2.1.1.BUILD"),
    )

    _new_http_archive(
        name = "server__walkdir__2_3_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/walkdir/walkdir-2.3.1.crate",
        type = "tar.gz",
        strip_prefix = "walkdir-2.3.1",
        build_file = Label("//server/cargo/remote:walkdir-2.3.1.BUILD"),
    )

    _new_http_archive(
        name = "server__winapi__0_3_9",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi/winapi-0.3.9.crate",
        type = "tar.gz",
        strip_prefix = "winapi-0.3.9",
        build_file = Label("//server/cargo/remote:winapi-0.3.9.BUILD"),
    )

    _new_http_archive(
        name = "server__winapi_i686_pc_windows_gnu__0_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi-i686-pc-windows-gnu/winapi-i686-pc-windows-gnu-0.4.0.crate",
        type = "tar.gz",
        strip_prefix = "winapi-i686-pc-windows-gnu-0.4.0",
        build_file = Label("//server/cargo/remote:winapi-i686-pc-windows-gnu-0.4.0.BUILD"),
    )

    _new_http_archive(
        name = "server__winapi_util__0_1_5",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi-util/winapi-util-0.1.5.crate",
        type = "tar.gz",
        strip_prefix = "winapi-util-0.1.5",
        build_file = Label("//server/cargo/remote:winapi-util-0.1.5.BUILD"),
    )

    _new_http_archive(
        name = "server__winapi_x86_64_pc_windows_gnu__0_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi-x86_64-pc-windows-gnu/winapi-x86_64-pc-windows-gnu-0.4.0.crate",
        type = "tar.gz",
        strip_prefix = "winapi-x86_64-pc-windows-gnu-0.4.0",
        build_file = Label("//server/cargo/remote:winapi-x86_64-pc-windows-gnu-0.4.0.BUILD"),
    )

