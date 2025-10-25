use crate::message_capnp::message_stream;
use capnp_rpc::{RpcSystem, rpc_twoparty_capnp, twoparty};
use futures::AsyncReadExt;
use std::io::{self, Write};
use std::net::ToSocketAddrs;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} client HOST:PORT", args[0]);
        println!("This will start an interactive conversation with the model.");
        return Ok(());
    }

    let addr = args[2]
        .to_socket_addrs()?
        .next()
        .expect("could not parse address");

    tokio::task::LocalSet::new()
        .run_until(async move {
            let stream = tokio::net::TcpStream::connect(&addr).await?;
            stream.set_nodelay(true)?;
            let (reader, writer) =
                tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
            let rpc_network = Box::new(twoparty::VatNetwork::new(
                futures::io::BufReader::new(reader),
                futures::io::BufWriter::new(writer),
                rpc_twoparty_capnp::Side::Client,
                Default::default(),
            ));
            let mut rpc_system = RpcSystem::new(rpc_network, None);
            let stream: message_stream::Client =
                rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

            tokio::task::spawn_local(rpc_system);

            println!("Connected to inference server at {}", addr);

            println!("Enter your message:");
            let mut user_message = String::new();
            io::stdin().read_line(&mut user_message)?;

            // Create request
            let mut request = stream.read_request();

            // Subsequent requests are empty to continue the stream
            request
                .get()
                .init_request()
                .set_content(user_message.trim());

            let reply = request.send().promise.await?;
            let chunk = reply.get()?.get_chunk()?;

            let message = std::str::from_utf8(chunk)?;
            println!("{}", message);
            io::stdout().flush()?;

            Ok(())
        })
        .await
}
