//! NeoTrix Web UI — standalone HTTP server with embedded SPA frontend.
//!
//! Usage: neotrix-web [--port PORT]
//! Default port: 3456

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "neotrix-web", about = "NeoTrix Web UI — HTTP server + embedded frontend")]
struct Args {
    #[arg(long, default_value_t = 3456, help = "Port to listen on")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    neotrix::neotrix::nt_io_web::server::start_server(args.port).await;
}
