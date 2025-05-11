pub mod base64key;

use std::net::SocketAddr;

use clap::Parser;

use crate::cli::base64key::Base64Key;
use sodiumoxide::crypto::box_::{PUBLICKEYBYTES, PublicKey, SECRETKEYBYTES, SecretKey};

pub fn get_default_bind() -> SocketAddr {
    "0.0.0.0:3000".parse().unwrap()
}

pub fn get_version() -> &'static str {
    option_env!("VERGEN_GIT_DESCRIBE").unwrap_or("No Version")
}

pub type SecretKeyValue = Base64Key<SecretKey, SECRETKEYBYTES>;
pub type PublicKeyValue = Base64Key<PublicKey, PUBLICKEYBYTES>;

#[derive(Parser, Clone, Debug)]
#[command(version=get_version())]
pub struct Cli {
    #[clap(short, long, default_value_t = get_default_bind(), env("S_BIND"))]
    pub bind: SocketAddr,

    #[clap(short, long, env = "SERVER_PRIVATE_KEY")]
    pub server_private_key: SecretKeyValue,

    #[clap(short, long, env = "MANAGER_PUBLIC_KEY")]
    pub manager_public_key: PublicKeyValue,

    #[clap(short, long, env = "COMMAND")]
    pub command: String,
}
