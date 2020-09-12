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
        sha256 = "043164d8ba5c4c3035fec9bbee8647c0261d788f3474306f93bb65901cae0e86",
        strip_prefix = "aho-corasick-0.7.13",
        build_file = Label("//server/cargo/remote:aho-corasick-0.7.13.BUILD"),
    )

    _new_http_archive(
        name = "server__anyhow__1_0_32",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/anyhow/anyhow-1.0.32.crate",
        type = "tar.gz",
        sha256 = "6b602bfe940d21c130f3895acd65221e8a61270debe89d628b9cb4e3ccb8569b",
        strip_prefix = "anyhow-1.0.32",
        build_file = Label("//server/cargo/remote:anyhow-1.0.32.BUILD"),
    )

    _new_http_archive(
        name = "server__autocfg__1_0_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/autocfg/autocfg-1.0.1.crate",
        type = "tar.gz",
        sha256 = "cdb031dd78e28731d87d56cc8ffef4a8f36ca26c38fe2de700543e627f8a464a",
        strip_prefix = "autocfg-1.0.1",
        build_file = Label("//server/cargo/remote:autocfg-1.0.1.BUILD"),
    )

    _new_http_archive(
        name = "server__base64__0_12_3",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/base64/base64-0.12.3.crate",
        type = "tar.gz",
        sha256 = "3441f0f7b02788e948e47f457ca01f1d7e6d92c693bc132c22b087d3141c03ff",
        strip_prefix = "base64-0.12.3",
        build_file = Label("//server/cargo/remote:base64-0.12.3.BUILD"),
    )

    _new_http_archive(
        name = "server__bit_set__0_5_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/bit-set/bit-set-0.5.2.crate",
        type = "tar.gz",
        sha256 = "6e11e16035ea35e4e5997b393eacbf6f63983188f7a2ad25bfb13465f5ad59de",
        strip_prefix = "bit-set-0.5.2",
        build_file = Label("//server/cargo/remote:bit-set-0.5.2.BUILD"),
    )

    _new_http_archive(
        name = "server__bit_vec__0_6_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/bit-vec/bit-vec-0.6.2.crate",
        type = "tar.gz",
        sha256 = "5f0dc55f2d8a1a85650ac47858bb001b4c0dd73d79e3c455a842925e68d29cd3",
        strip_prefix = "bit-vec-0.6.2",
        build_file = Label("//server/cargo/remote:bit-vec-0.6.2.BUILD"),
    )

    _new_http_archive(
        name = "server__bitflags__1_2_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/bitflags/bitflags-1.2.1.crate",
        type = "tar.gz",
        sha256 = "cf1de2fe8c75bc145a2f577add951f8134889b4795d47466a54a5c846d691693",
        strip_prefix = "bitflags-1.2.1",
        build_file = Label("//server/cargo/remote:bitflags-1.2.1.BUILD"),
    )

    _new_http_archive(
        name = "server__cfg_if__0_1_10",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/cfg-if/cfg-if-0.1.10.crate",
        type = "tar.gz",
        sha256 = "4785bdd1c96b2a846b2bd7cc02e86b6b3dbf14e7e53446c4f54c92a361040822",
        strip_prefix = "cfg-if-0.1.10",
        build_file = Label("//server/cargo/remote:cfg-if-0.1.10.BUILD"),
    )

    _new_http_archive(
        name = "server__chan__0_1_23",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/chan/chan-0.1.23.crate",
        type = "tar.gz",
        sha256 = "d14956a3dae065ffaa0d92ece848ab4ced88d32361e7fdfbfd653a5c454a1ed8",
        strip_prefix = "chan-0.1.23",
        build_file = Label("//server/cargo/remote:chan-0.1.23.BUILD"),
    )

    _new_http_archive(
        name = "server__fixedbitset__0_2_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/fixedbitset/fixedbitset-0.2.0.crate",
        type = "tar.gz",
        sha256 = "37ab347416e802de484e4d03c7316c48f1ecb56574dfd4a46a80f173ce1de04d",
        strip_prefix = "fixedbitset-0.2.0",
        build_file = Label("//server/cargo/remote:fixedbitset-0.2.0.BUILD"),
    )

    _new_http_archive(
        name = "server__fs_extra__1_2_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/fs_extra/fs_extra-1.2.0.crate",
        type = "tar.gz",
        sha256 = "2022715d62ab30faffd124d40b76f4134a550a87792276512b18d63272333394",
        strip_prefix = "fs_extra-1.2.0",
        build_file = Label("//server/cargo/remote:fs_extra-1.2.0.BUILD"),
    )

    _new_http_archive(
        name = "server__fuchsia_cprng__0_1_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/fuchsia-cprng/fuchsia-cprng-0.1.1.crate",
        type = "tar.gz",
        sha256 = "a06f77d526c1a601b7c4cdd98f54b5eaabffc14d5f2f0296febdc7f357c6d3ba",
        strip_prefix = "fuchsia-cprng-0.1.1",
        build_file = Label("//server/cargo/remote:fuchsia-cprng-0.1.1.BUILD"),
    )

    _new_http_archive(
        name = "server__futures__0_1_29",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/futures/futures-0.1.29.crate",
        type = "tar.gz",
        sha256 = "1b980f2816d6ee8673b6517b52cb0e808a180efc92e5c19d02cdda79066703ef",
        strip_prefix = "futures-0.1.29",
        build_file = Label("//server/cargo/remote:futures-0.1.29.BUILD"),
    )

    _new_http_archive(
        name = "server__hamcrest2__0_3_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/hamcrest2/hamcrest2-0.3.0.crate",
        type = "tar.gz",
        sha256 = "49f837c62de05dc9cc71ff6486cd85de8856a330395ae338a04bfcefe5e91075",
        strip_prefix = "hamcrest2-0.3.0",
        build_file = Label("//server/cargo/remote:hamcrest2-0.3.0.BUILD"),
    )

    _new_http_archive(
        name = "server__idna__0_2_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/idna/idna-0.2.0.crate",
        type = "tar.gz",
        sha256 = "02e2673c30ee86b5b96a9cb52ad15718aa1f966f5ab9ad54a8b95d5ca33120a9",
        strip_prefix = "idna-0.2.0",
        build_file = Label("//server/cargo/remote:idna-0.2.0.BUILD"),
    )

    _new_http_archive(
        name = "server__indexmap__1_0_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/indexmap/indexmap-1.0.2.crate",
        type = "tar.gz",
        sha256 = "7e81a7c05f79578dbc15793d8b619db9ba32b4577003ef3af1a91c416798c58d",
        strip_prefix = "indexmap-1.0.2",
        build_file = Label("//server/cargo/remote:indexmap-1.0.2.BUILD"),
    )

    _new_http_archive(
        name = "server__itoa__0_4_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/itoa/itoa-0.4.6.crate",
        type = "tar.gz",
        sha256 = "dc6f3ad7b9d11a0c00842ff8de1b60ee58661048eb8049ed33c73594f359d7e6",
        strip_prefix = "itoa-0.4.6",
        build_file = Label("//server/cargo/remote:itoa-0.4.6.BUILD"),
    )

    _new_http_archive(
        name = "server__lazy_static__1_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/lazy_static/lazy_static-1.4.0.crate",
        type = "tar.gz",
        sha256 = "e2abad23fbc42b3700f2f279844dc832adb2b2eb069b2df918f455c4e18cc646",
        strip_prefix = "lazy_static-1.4.0",
        build_file = Label("//server/cargo/remote:lazy_static-1.4.0.BUILD"),
    )

    _new_http_archive(
        name = "server__libc__0_2_77",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/libc/libc-0.2.77.crate",
        type = "tar.gz",
        sha256 = "f2f96b10ec2560088a8e76961b00d47107b3a625fecb76dedb29ee7ccbf98235",
        strip_prefix = "libc-0.2.77",
        build_file = Label("//server/cargo/remote:libc-0.2.77.BUILD"),
    )

    _new_http_archive(
        name = "server__log__0_4_11",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/log/log-0.4.11.crate",
        type = "tar.gz",
        sha256 = "4fabed175da42fed1fa0746b0ea71f412aa9d35e76e95e59b192c64b9dc2bf8b",
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
        sha256 = "7ffc5c5338469d4d3ea17d269fa8ea3512ad247247c30bd2df69e68309ed0a08",
        strip_prefix = "matches-0.1.8",
        build_file = Label("//server/cargo/remote:matches-0.1.8.BUILD"),
    )

    _new_http_archive(
        name = "server__memchr__2_3_3",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/memchr/memchr-2.3.3.crate",
        type = "tar.gz",
        sha256 = "3728d817d99e5ac407411fa471ff9800a778d88a24685968b36824eaf4bee400",
        strip_prefix = "memchr-2.3.3",
        build_file = Label("//server/cargo/remote:memchr-2.3.3.BUILD"),
    )

    _new_http_archive(
        name = "server__num__0_2_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num/num-0.2.1.crate",
        type = "tar.gz",
        sha256 = "b8536030f9fea7127f841b45bb6243b27255787fb4eb83958aa1ef9d2fdc0c36",
        strip_prefix = "num-0.2.1",
        build_file = Label("//server/cargo/remote:num-0.2.1.BUILD"),
    )

    _new_http_archive(
        name = "server__num_bigint__0_2_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-bigint/num-bigint-0.2.6.crate",
        type = "tar.gz",
        sha256 = "090c7f9998ee0ff65aa5b723e4009f7b217707f1fb5ea551329cc4d6231fb304",
        strip_prefix = "num-bigint-0.2.6",
        build_file = Label("//server/cargo/remote:num-bigint-0.2.6.BUILD"),
    )

    _new_http_archive(
        name = "server__num_complex__0_2_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-complex/num-complex-0.2.4.crate",
        type = "tar.gz",
        sha256 = "b6b19411a9719e753aff12e5187b74d60d3dc449ec3f4dc21e3989c3f554bc95",
        strip_prefix = "num-complex-0.2.4",
        build_file = Label("//server/cargo/remote:num-complex-0.2.4.BUILD"),
    )

    _new_http_archive(
        name = "server__num_integer__0_1_43",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-integer/num-integer-0.1.43.crate",
        type = "tar.gz",
        sha256 = "8d59457e662d541ba17869cf51cf177c0b5f0cbf476c66bdc90bf1edac4f875b",
        strip_prefix = "num-integer-0.1.43",
        build_file = Label("//server/cargo/remote:num-integer-0.1.43.BUILD"),
    )

    _new_http_archive(
        name = "server__num_iter__0_1_41",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-iter/num-iter-0.1.41.crate",
        type = "tar.gz",
        sha256 = "7a6e6b7c748f995c4c29c5f5ae0248536e04a5739927c74ec0fa564805094b9f",
        strip_prefix = "num-iter-0.1.41",
        build_file = Label("//server/cargo/remote:num-iter-0.1.41.BUILD"),
    )

    _new_http_archive(
        name = "server__num_rational__0_2_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-rational/num-rational-0.2.4.crate",
        type = "tar.gz",
        sha256 = "5c000134b5dbf44adc5cb772486d335293351644b801551abe8f75c84cfa4aef",
        strip_prefix = "num-rational-0.2.4",
        build_file = Label("//server/cargo/remote:num-rational-0.2.4.BUILD"),
    )

    _new_http_archive(
        name = "server__num_traits__0_2_12",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/num-traits/num-traits-0.2.12.crate",
        type = "tar.gz",
        sha256 = "ac267bcc07f48ee5f8935ab0d24f316fb722d7a1292e2913f0cc196b29ffd611",
        strip_prefix = "num-traits-0.2.12",
        build_file = Label("//server/cargo/remote:num-traits-0.2.12.BUILD"),
    )

    _new_http_archive(
        name = "server__percent_encoding__2_1_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/percent-encoding/percent-encoding-2.1.0.crate",
        type = "tar.gz",
        sha256 = "d4fd5641d01c8f18a23da7b6fe29298ff4b55afcccdf78973b24cf3175fee32e",
        strip_prefix = "percent-encoding-2.1.0",
        build_file = Label("//server/cargo/remote:percent-encoding-2.1.0.BUILD"),
    )

    _new_http_archive(
        name = "server__petgraph__0_5_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/petgraph/petgraph-0.5.1.crate",
        type = "tar.gz",
        sha256 = "467d164a6de56270bd7c4d070df81d07beace25012d5103ced4e9ff08d6afdb7",
        strip_prefix = "petgraph-0.5.1",
        build_file = Label("//server/cargo/remote:petgraph-0.5.1.BUILD"),
    )

    _new_http_archive(
        name = "server__proc_macro2__1_0_21",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/proc-macro2/proc-macro2-1.0.21.crate",
        type = "tar.gz",
        sha256 = "36e28516df94f3dd551a587da5357459d9b36d945a7c37c3557928c1c2ff2a2c",
        strip_prefix = "proc-macro2-1.0.21",
        build_file = Label("//server/cargo/remote:proc-macro2-1.0.21.BUILD"),
    )

    _new_http_archive(
        name = "server__quote__1_0_7",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/quote/quote-1.0.7.crate",
        type = "tar.gz",
        sha256 = "aa563d17ecb180e500da1cfd2b028310ac758de548efdd203e18f283af693f37",
        strip_prefix = "quote-1.0.7",
        build_file = Label("//server/cargo/remote:quote-1.0.7.BUILD"),
    )

    _new_http_archive(
        name = "server__rand__0_3_23",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rand/rand-0.3.23.crate",
        type = "tar.gz",
        sha256 = "64ac302d8f83c0c1974bf758f6b041c6c8ada916fbb44a609158ca8b064cc76c",
        strip_prefix = "rand-0.3.23",
        build_file = Label("//server/cargo/remote:rand-0.3.23.BUILD"),
    )

    _new_http_archive(
        name = "server__rand__0_4_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rand/rand-0.4.6.crate",
        type = "tar.gz",
        sha256 = "552840b97013b1a26992c11eac34bdd778e464601a4c2054b5f0bff7c6761293",
        strip_prefix = "rand-0.4.6",
        build_file = Label("//server/cargo/remote:rand-0.4.6.BUILD"),
    )

    _new_http_archive(
        name = "server__rand_core__0_3_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rand_core/rand_core-0.3.1.crate",
        type = "tar.gz",
        sha256 = "7a6fdeb83b075e8266dcc8762c22776f6877a63111121f5f8c7411e5be7eed4b",
        strip_prefix = "rand_core-0.3.1",
        build_file = Label("//server/cargo/remote:rand_core-0.3.1.BUILD"),
    )

    _new_http_archive(
        name = "server__rand_core__0_4_2",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rand_core/rand_core-0.4.2.crate",
        type = "tar.gz",
        sha256 = "9c33a3c44ca05fa6f1807d8e6743f3824e8509beca625669633be0acbdf509dc",
        strip_prefix = "rand_core-0.4.2",
        build_file = Label("//server/cargo/remote:rand_core-0.4.2.BUILD"),
    )

    _new_http_archive(
        name = "server__rdrand__0_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/rdrand/rdrand-0.4.0.crate",
        type = "tar.gz",
        sha256 = "678054eb77286b51581ba43620cc911abf02758c91f93f479767aed0f90458b2",
        strip_prefix = "rdrand-0.4.0",
        build_file = Label("//server/cargo/remote:rdrand-0.4.0.BUILD"),
    )

    _new_http_archive(
        name = "server__regex__1_3_9",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/regex/regex-1.3.9.crate",
        type = "tar.gz",
        sha256 = "9c3780fcf44b193bc4d09f36d2a3c87b251da4a046c87795a0d35f4f927ad8e6",
        strip_prefix = "regex-1.3.9",
        build_file = Label("//server/cargo/remote:regex-1.3.9.BUILD"),
    )

    _new_http_archive(
        name = "server__regex_syntax__0_6_18",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/regex-syntax/regex-syntax-0.6.18.crate",
        type = "tar.gz",
        sha256 = "26412eb97c6b088a6997e05f69403a802a92d520de2f8e63c2b65f9e0f47c4e8",
        strip_prefix = "regex-syntax-0.6.18",
        build_file = Label("//server/cargo/remote:regex-syntax-0.6.18.BUILD"),
    )

    _new_http_archive(
        name = "server__remove_dir_all__0_5_3",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/remove_dir_all/remove_dir_all-0.5.3.crate",
        type = "tar.gz",
        sha256 = "3acd125665422973a33ac9d3dd2df85edad0f4ae9b00dafb1a05e43a9f5ef8e7",
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
        sha256 = "7cfffa8a89d8758be2dd5605c5fc62bce055af2491ebf3ce953d4d31512c59fd",
        strip_prefix = "rustdt_util-0.2.3",
        build_file = Label("//server/cargo/remote:rustdt_util-0.2.3.BUILD"),
    )

    _new_http_archive(
        name = "server__ryu__1_0_5",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/ryu/ryu-1.0.5.crate",
        type = "tar.gz",
        sha256 = "71d301d4193d031abdd79ff7e3dd721168a9572ef3fe51a1517aba235bd8f86e",
        strip_prefix = "ryu-1.0.5",
        build_file = Label("//server/cargo/remote:ryu-1.0.5.BUILD"),
    )

    _new_http_archive(
        name = "server__same_file__1_0_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/same-file/same-file-1.0.6.crate",
        type = "tar.gz",
        sha256 = "93fc1dc3aaa9bfed95e02e6eadabb4baf7e3078b0bd1b4d7b6b0b68378900502",
        strip_prefix = "same-file-1.0.6",
        build_file = Label("//server/cargo/remote:same-file-1.0.6.BUILD"),
    )

    _new_http_archive(
        name = "server__serde__1_0_116",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde/serde-1.0.116.crate",
        type = "tar.gz",
        sha256 = "96fe57af81d28386a513cbc6858332abc6117cfdb5999647c6444b8f43a370a5",
        strip_prefix = "serde-1.0.116",
        build_file = Label("//server/cargo/remote:serde-1.0.116.BUILD"),
    )

    _new_http_archive(
        name = "server__serde_derive__1_0_116",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde_derive/serde_derive-1.0.116.crate",
        type = "tar.gz",
        sha256 = "f630a6370fd8e457873b4bd2ffdae75408bc291ba72be773772a4c2a065d9ae8",
        strip_prefix = "serde_derive-1.0.116",
        build_file = Label("//server/cargo/remote:serde_derive-1.0.116.BUILD"),
    )

    _new_http_archive(
        name = "server__serde_json__1_0_57",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde_json/serde_json-1.0.57.crate",
        type = "tar.gz",
        sha256 = "164eacbdb13512ec2745fb09d51fd5b22b0d65ed294a1dcf7285a360c80a675c",
        strip_prefix = "serde_json-1.0.57",
        build_file = Label("//server/cargo/remote:serde_json-1.0.57.BUILD"),
    )

    _new_http_archive(
        name = "server__serde_repr__0_1_6",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/serde_repr/serde_repr-0.1.6.crate",
        type = "tar.gz",
        sha256 = "2dc6b7951b17b051f3210b063f12cc17320e2fe30ae05b0fe2a3abb068551c76",
        strip_prefix = "serde_repr-0.1.6",
        build_file = Label("//server/cargo/remote:serde_repr-0.1.6.BUILD"),
    )

    _new_http_archive(
        name = "server__syn__1_0_40",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/syn/syn-1.0.40.crate",
        type = "tar.gz",
        sha256 = "963f7d3cc59b59b9325165add223142bbf1df27655d07789f109896d353d8350",
        strip_prefix = "syn-1.0.40",
        build_file = Label("//server/cargo/remote:syn-1.0.40.BUILD"),
    )

    _new_http_archive(
        name = "server__tempdir__0_3_7",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/tempdir/tempdir-0.3.7.crate",
        type = "tar.gz",
        sha256 = "15f2b5fb00ccdf689e0149d1b1b3c03fead81c2b37735d812fa8bddbbf41b6d8",
        strip_prefix = "tempdir-0.3.7",
        build_file = Label("//server/cargo/remote:tempdir-0.3.7.BUILD"),
    )

    _new_http_archive(
        name = "server__thiserror__1_0_20",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/thiserror/thiserror-1.0.20.crate",
        type = "tar.gz",
        sha256 = "7dfdd070ccd8ccb78f4ad66bf1982dc37f620ef696c6b5028fe2ed83dd3d0d08",
        strip_prefix = "thiserror-1.0.20",
        build_file = Label("//server/cargo/remote:thiserror-1.0.20.BUILD"),
    )

    _new_http_archive(
        name = "server__thiserror_impl__1_0_20",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/thiserror-impl/thiserror-impl-1.0.20.crate",
        type = "tar.gz",
        sha256 = "bd80fc12f73063ac132ac92aceea36734f04a1d93c1240c6944e23a3b8841793",
        strip_prefix = "thiserror-impl-1.0.20",
        build_file = Label("//server/cargo/remote:thiserror-impl-1.0.20.BUILD"),
    )

    _new_http_archive(
        name = "server__thread_local__1_0_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/thread_local/thread_local-1.0.1.crate",
        type = "tar.gz",
        sha256 = "d40c6d1b69745a6ec6fb1ca717914848da4b44ae29d9b3080cbee91d72a69b14",
        strip_prefix = "thread_local-1.0.1",
        build_file = Label("//server/cargo/remote:thread_local-1.0.1.BUILD"),
    )

    _new_http_archive(
        name = "server__tinyvec__0_3_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/tinyvec/tinyvec-0.3.4.crate",
        type = "tar.gz",
        sha256 = "238ce071d267c5710f9d31451efec16c5ee22de34df17cc05e56cbc92e967117",
        strip_prefix = "tinyvec-0.3.4",
        build_file = Label("//server/cargo/remote:tinyvec-0.3.4.BUILD"),
    )

    _new_http_archive(
        name = "server__unicode_bidi__0_3_4",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/unicode-bidi/unicode-bidi-0.3.4.crate",
        type = "tar.gz",
        sha256 = "49f2bd0c6468a8230e1db229cff8029217cf623c767ea5d60bfbd42729ea54d5",
        strip_prefix = "unicode-bidi-0.3.4",
        build_file = Label("//server/cargo/remote:unicode-bidi-0.3.4.BUILD"),
    )

    _new_http_archive(
        name = "server__unicode_normalization__0_1_13",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/unicode-normalization/unicode-normalization-0.1.13.crate",
        type = "tar.gz",
        sha256 = "6fb19cf769fa8c6a80a162df694621ebeb4dafb606470b2b2fce0be40a98a977",
        strip_prefix = "unicode-normalization-0.1.13",
        build_file = Label("//server/cargo/remote:unicode-normalization-0.1.13.BUILD"),
    )

    _new_http_archive(
        name = "server__unicode_xid__0_2_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/unicode-xid/unicode-xid-0.2.1.crate",
        type = "tar.gz",
        sha256 = "f7fe0bb3479651439c9112f72b6c505038574c9fbb575ed1bf3b797fa39dd564",
        strip_prefix = "unicode-xid-0.2.1",
        build_file = Label("//server/cargo/remote:unicode-xid-0.2.1.BUILD"),
    )

    _new_http_archive(
        name = "server__url__2_1_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/url/url-2.1.1.crate",
        type = "tar.gz",
        sha256 = "829d4a8476c35c9bf0bbce5a3b23f4106f79728039b726d292bb93bc106787cb",
        strip_prefix = "url-2.1.1",
        build_file = Label("//server/cargo/remote:url-2.1.1.BUILD"),
    )

    _new_http_archive(
        name = "server__walkdir__2_3_1",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/walkdir/walkdir-2.3.1.crate",
        type = "tar.gz",
        sha256 = "777182bc735b6424e1a57516d35ed72cb8019d85c8c9bf536dccb3445c1a2f7d",
        strip_prefix = "walkdir-2.3.1",
        build_file = Label("//server/cargo/remote:walkdir-2.3.1.BUILD"),
    )

    _new_http_archive(
        name = "server__winapi__0_3_9",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi/winapi-0.3.9.crate",
        type = "tar.gz",
        sha256 = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419",
        strip_prefix = "winapi-0.3.9",
        build_file = Label("//server/cargo/remote:winapi-0.3.9.BUILD"),
    )

    _new_http_archive(
        name = "server__winapi_i686_pc_windows_gnu__0_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi-i686-pc-windows-gnu/winapi-i686-pc-windows-gnu-0.4.0.crate",
        type = "tar.gz",
        sha256 = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6",
        strip_prefix = "winapi-i686-pc-windows-gnu-0.4.0",
        build_file = Label("//server/cargo/remote:winapi-i686-pc-windows-gnu-0.4.0.BUILD"),
    )

    _new_http_archive(
        name = "server__winapi_util__0_1_5",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi-util/winapi-util-0.1.5.crate",
        type = "tar.gz",
        sha256 = "70ec6ce85bb158151cae5e5c87f95a8e97d2c0c4b001223f33a334e3ce5de178",
        strip_prefix = "winapi-util-0.1.5",
        build_file = Label("//server/cargo/remote:winapi-util-0.1.5.BUILD"),
    )

    _new_http_archive(
        name = "server__winapi_x86_64_pc_windows_gnu__0_4_0",
        url = "https://crates-io.s3-us-west-1.amazonaws.com/crates/winapi-x86_64-pc-windows-gnu/winapi-x86_64-pc-windows-gnu-0.4.0.crate",
        type = "tar.gz",
        sha256 = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f",
        strip_prefix = "winapi-x86_64-pc-windows-gnu-0.4.0",
        build_file = Label("//server/cargo/remote:winapi-x86_64-pc-windows-gnu-0.4.0.BUILD"),
    )

