use corridor::AppConfig;
use corridor::Network;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let config = if args.len() > 1 {
        let network_path = &args[1];
        let network = parse_network(network_path);
        AppConfig::with_network(network)
    } else {
        AppConfig::default()
    };

    corridor::run_app(config);
}

fn parse_network(path: &String) -> corridor::Network {
    let json = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read network file {}: {}", path, e));
    let network: Network = serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("Failed to parse network JSON from {}: {}", path, e));
    network
}
