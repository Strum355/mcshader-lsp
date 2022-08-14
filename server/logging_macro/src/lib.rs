use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemFn};

#[proc_macro_attribute]
pub fn scope(_args: TokenStream, function: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(function as ItemFn);
    let function_name = function.sig.ident.to_string();
    let stmts = function.block.stmts;

    function.block = Box::new(parse_quote!({
        use logging::{slog_o, FnValue, Level, scope, logger};
        use std::thread::current;

        let _guard = logging::set_level(Level::Trace);
        scope(&logger().new(slog_o!("test_name" => #function_name, "thread_num" => FnValue(|_| format!("{:?}", current().id())))), || {
            #(#stmts)*
        });
    }));

    TokenStream::from(quote!(#function))
}

#[proc_macro_attribute]
pub fn with_trace_id(_args: TokenStream, function: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(function as ItemFn);
    let stmts = function.block.stmts;

    function.block = Box::new(parse_quote!({
        use logging::{slog_o, scope, logger, new_trace_id};

        scope(&logger().new(slog_o!("trace" => new_trace_id())), || {
            #(#stmts)*
        })
    }));

    TokenStream::from(quote!(#function))
}
