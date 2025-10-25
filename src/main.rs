capnp::generated_code!(pub mod message_capnp);

pub mod analysis;
pub mod client;
pub mod server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        match &args[1][..] {
            "client" => client::run().await?,
            "server" => server::run().await?,
            _ => {
                println!("usage: {} [client | server] ADDRESS", args[0]);
            }
        }
    } else {
        println!("usage: {} [client | server] ADDRESS", args[0]);
    }

    Ok(())
}
