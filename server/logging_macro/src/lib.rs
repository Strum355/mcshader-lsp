use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemFn};

#[proc_macro_attribute]
pub fn log_scope(_args: TokenStream, function: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(function as ItemFn);

    let function_name = function.sig.ident.to_string();

    let stmts = function.block.stmts;

    function.block = Box::new(parse_quote!({
        use slog::{slog_o, FnValue, Level};
        use std::thread::current;

        let _guard = logging::set_logger_with_level(Level::Trace);
        slog_scope::scope(&slog_scope::logger().new(slog_o!("test_name" => #function_name, "thread_num" => FnValue(|_| format!("{:?}", current().id())))), || {
            #(#stmts)*
        });
    }));

    TokenStream::from(quote!(#function))
}
