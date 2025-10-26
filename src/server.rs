use std::net::ToSocketAddrs;

use capnp_rpc::{RpcSystem, pry, rpc_twoparty_capnp, twoparty};
use tokio::net::TcpStream;

use core_affinity;
use tokio::sync::mpsc;

use crate::analysis;
use crate::message_capnp::message_stream;
use capnp::capability::Promise;

use futures::AsyncReadExt;
use tokio_util::compat::TokioAsyncReadCompatExt;

struct StreamImpl {}

impl message_stream::Server for StreamImpl {
    fn read(
        &mut self,
        params: message_stream::ReadParams,
        mut results: message_stream::ReadResults,
    ) -> Promise<(), ::capnp::Error> {
        // Get the request from the parameters
        let request = pry!(params.get());
        // Get the content from the request
        let content = pry!(pry!(request.get_request()).get_content());

        if !content.is_empty() {
            // Process the content
            let content_str = pry!(
                content
                    .to_str()
                    .map_err(|e| ::capnp::Error::failed(format!("UTF-8 error: {}", e)))
            );
            println!("Received content: {}", content_str);

            let reversed_content = analysis::reverse(content_str);

            // to send something back, we set the chunk for the result parameter
            results.get().set_chunk(reversed_content.as_bytes());
        }

        Promise::ok(())
    }
}

fn worker(id: usize, mut rx: mpsc::Receiver<TcpStream>) {
    // Each worker gets its own single-threaded runtime and LocalSet.
    // We need to create a LocalSet because cap'n proto doesn't support Send between threads.
    // Hence, we need to create a LocalSet to ensure that the LocalSet is not Send between threads.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let local = tokio::task::LocalSet::new();

    // Get the data from the local stream (created thread)
    local.block_on(&rt, async move {
        while let Some(stream) = rx.recv().await {
            // Get the address of the peer
            let peer_addr = stream
                .peer_addr()
                .map(|a| a.to_string())
                .unwrap_or_else(|_| "unknown".to_string());

            tokio::task::spawn_local(async move {
                println!(
                    "[Worker {}] received a new connection from {}",
                    id, peer_addr
                );
                if let Some(core_ids) = core_affinity::get_core_ids() {
                    if !core_ids.is_empty() {
                        println!(
                            "[Worker {}] `spawn_local` is running on core {:?}",
                            id, core_ids
                        );
                    }
                }

                let stream_impl = StreamImpl {};

                // here we create the client that handles the output of the stream
                let client: message_stream::Client = capnp_rpc::new_client(stream_impl);

                let (reader, writer) = stream.compat().split();
                let network = twoparty::VatNetwork::new(
                    futures::io::BufReader::new(reader),
                    futures::io::BufWriter::new(writer),
                    rpc_twoparty_capnp::Side::Server,
                    Default::default(),
                );

                let rpc_system = RpcSystem::new(Box::new(network), Some(client.client));

                println!("[Worker {}] RPC system started for peer: {}", id, peer_addr);

                if let Err(e) = rpc_system.await {
                    eprintln!(
                        "[Worker {}] RPC system error for peer {}: {}",
                        id, peer_addr, e
                    );
                } else {
                    println!(
                        "[Worker {}] RPC system for peer {} finished cleanly.",
                        id, peer_addr
                    );
                }
            });
        }
    });
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let addr = String::from("127.0.0.1:5000")
        .to_socket_addrs()?
        .next()
        .expect("could not parse address");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let core_ids = core_affinity::get_core_ids().unwrap();
    let num_workers = core_ids.len();
    let mut senders = Vec::new();

    for (i, core_id) in core_ids.into_iter().enumerate() {
        let (tx, rx) = mpsc::channel(100);
        senders.push(tx);
        // We cannot create multiple Tokio runtimes on the same OS thread - each needs its own dedicated thread.
        // spinning up OS threads for each worker
        std::thread::spawn(move || {
            // Pin this thread to a single CPU core.
            if core_affinity::set_for_current(core_id) {
                worker(i, rx);
            }
        });
    }

    println!(
        "Listening on {} with {} worker threads...",
        addr, num_workers
    );

    let mut next_worker = 0;
    // The main server loop
    // assign incoming connections in a round-robin fashion to the worker threads.
    loop {
        // Accept incoming connections
        let (stream, _) = listener.accept().await?;
        // Set no delay to true disables the Nagle algorithm.
        // This means that segments are always sent as soon as possible, even if there is only a small amount of data.
        // When not set, data is buffered until there is a sufficient amount to send out, thereby avoiding the frequent sending of small packets.
        stream.set_nodelay(true)?;

        // Dispatch the new connection to the next worker in a round-robin fashion.
        // Here we get the channel for the next worker.
        let sender = &senders[next_worker];
        println!("Dispatching new connection to worker {}", next_worker);
        // we stream the message from the connection to the worker thread using the worker's channel.
        if let Err(e) = sender.send(stream).await {
            eprintln!("Failed to dispatch connection to worker: {}", e);
        }

        next_worker = (next_worker + 1) % num_workers;
    }
}
