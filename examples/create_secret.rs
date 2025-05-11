use unsealer::utils::crypto::encode_payload;

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use sodiumoxide::crypto::box_;
use serde_json::json;

fn main() {
    sodiumoxide::init().unwrap();
       
    // Generate key pairs for manager (sending secret) 
    // and server (running unsealer and afterward the service)
    let (manager_public, manager_private) = box_::gen_keypair();
    let (server_public, server_private) = box_::gen_keypair();

    // This JSON will be encrypted and sent as config
    let config_json = json!({
        "secret": "world"
    });
    
    let encoded = encode_payload(&config_json, &manager_private, &server_public).unwrap();

    // Output encoded config
    println!("CONFIG PARAM:\n{}", encoded);

    // Output base64 keys for server/client use
    println!(
        "\nSERVER PRIVATE KEY (Base64): {}",
        URL_SAFE.encode(server_private)
    );
    println!(
        "SERVER PUBLIC KEY (Base64):  {}",
        URL_SAFE.encode(server_public)
    );
    println!(
        "MANAGER PRIVATE KEY (Base64): {}",
        URL_SAFE.encode(manager_private)
    );
    println!(
        "MANAGER PUBLIC KEY (Base64):  {}",
        URL_SAFE.encode(manager_public)
    );
}
