use std::{
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc,
    },
    thread,
};

use crate::{Context, ShaderValidator};

enum ClientMessage {
    Validate { tree_type: crate::TreeType, source: Arc<str> },
    Vendor,
}

enum ServerMessage {
    Validate(Option<String>),
    Vendor(String),
}

///
pub struct ContextFacade {
    start_chan: SyncSender<()>,
    client_tx: SyncSender<ClientMessage>,
    server_rx: Receiver<ServerMessage>,
}

impl Default for ContextFacade {
    fn default() -> Self {
        let (client_tx, client_rx) = mpsc::sync_channel::<ClientMessage>(1);
        let (server_tx, server_rx) = mpsc::sync_channel::<ServerMessage>(1);
        let (start_chan, start_chan_recv) = mpsc::sync_channel::<()>(1);
        thread::spawn(move || {
            start_chan_recv.recv().unwrap();

            let opengl_ctx = Context::default();
            loop {
                match client_rx.recv() {
                    Ok(msg) => {
                        if let ClientMessage::Validate { tree_type, source } = msg {
                            server_tx
                                .send(ServerMessage::Validate(opengl_ctx.validate(tree_type, &source)))
                                .unwrap();
                        } else {
                            server_tx.send(ServerMessage::Vendor(opengl_ctx.vendor())).unwrap();
                        }
                    }
                    Err(_) => return,
                }
                start_chan_recv.recv().unwrap();
            }
        });

        ContextFacade {
            start_chan,
            client_tx,
            server_rx,
        }
    }
}

impl ShaderValidator for ContextFacade {
    fn validate(&self, tree_type: crate::TreeType, source: &str) -> Option<String> {
        self.start_chan.send(()).unwrap();
        match self.client_tx.send(ClientMessage::Validate {
            tree_type,
            source: Arc::from(source),
        }) {
            Ok(_) => match self.server_rx.recv().unwrap() {
                ServerMessage::Validate(output) => output,
                ServerMessage::Vendor(_) => panic!("expected validate reply, got vendor"),
            },
            Err(e) => panic!("error sending vendor message: {:?}", e),
        }
    }

    fn vendor(&self) -> String {
        self.start_chan.send(()).unwrap();
        match self.client_tx.send(ClientMessage::Vendor) {
            Ok(_) => match self.server_rx.recv().unwrap() {
                ServerMessage::Validate(_) => panic!("expected vendor reply, got validate"),
                ServerMessage::Vendor(resp) => resp,
            },
            Err(e) => panic!("error sending vendor message: {:?}", e),
        }
    }
}
