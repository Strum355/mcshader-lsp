#![feature(option_get_or_insert_default)]
use logging::{logger, FutureExt};
use server::Server;
use tower_lsp::LspService;

mod configuration;
mod navigation;

#[tokio::main]
async fn main() {
    let _guard = logging::set_level(logging::Level::Debug);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Server::new(client, opengl::ContextFacade::default));
    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .with_logger(logger())
        .await;
}
