use structopt::StructOpt;
use crate::types::Arguments;

mod client;
mod utils;
mod server;
mod types;
mod proto;
mod original;

fn main() {
	// Parse command-line arguments to determine whether to run the server or client
	// let args: Vec<String> = std::env::args().collect();

	let opt = Arguments::from_args();
	println!("{}", opt.server);
	if opt.main {
		println!("Starting main...");
		original::main();
	}
	if opt.server {
		println!("Starting server...");
		if let Err(e) = server::run_server() {
			eprintln!("Server error: {}", e);
		}
	} else if opt.client {
		println!("Starting client...");
		if let Err(e) = client::run_client() {
			eprintln!("Client error: {}", e);
		}
	} else {
		eprintln!("Invalid argument. Use -h for more info");
	}
}
