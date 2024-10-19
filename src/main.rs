mod client;
mod utils;
mod server;
mod types;
mod proto;

fn main() {
	// Parse command-line arguments to determine whether to run the server or client
	let args: Vec<String> = std::env::args().collect();

	if args.len() > 1 {
		match args[1].as_str() {
			"server" => {
				println!("Starting server...");
				if let Err(e) = server::run_server() {
					eprintln!("Server error: {}", e);
				}
			}
			"client" => {
				println!("Starting client...");
				if let Err(e) = client::run_client() {
					eprintln!("Client error: {}", e);
				}
			}
			_ => {
				eprintln!("Invalid argument. Use 'server' or 'client'.");
			}
		}
	} else {
		eprintln!("Usage: {} [server|client]", args[0]);
	}
}
