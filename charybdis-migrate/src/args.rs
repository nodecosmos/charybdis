use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Scylla Host
    #[arg(long)]
    pub host: String,

    /// Keyspace
    #[arg(short, long)]
    pub keyspace: String,

    #[arg(short, long, default_value = None)]
    pub user: Option<String>,

    #[arg(short, long, default_value = None)]
    pub password: Option<String>,

    #[arg(short, long, default_value_t = 30)]
    pub timeout: u64,

    /// Drop and recreate columns in case of type change
    #[arg(short, long, default_value_t = false)]
    pub drop_and_replace: bool,

    /// Prints alter table options queries
    #[arg(long, default_value_t = false)]
    pub verbose: bool,

    /// Path to the CA file if using TLS
    #[arg(long, default_value = None)]
    pub ca: Option<String>,

    /// Client certificate file if required_client_auth is set to true
    #[arg(long, default_value = None)]
    pub cert: Option<String>,

    /// Client private key file if required_client_auth is set to true
    #[arg(long, default_value = None)]
    pub key: Option<String>,

    #[arg(skip = get_current_dir())]
    pub current_dir: String,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            host: String::new(),
            keyspace: String::new(),
            user: None,
            password: None,
            timeout: 30,
            drop_and_replace: false,
            verbose: false,
            ca: None,
            cert: None,
            key: None,
            current_dir: get_current_dir(),
        }
    }
}

pub(crate) fn get_current_dir() -> String {
    let path = env::current_dir().expect("Failed to find project root: Could not get current directory");

    path.to_str()
        .expect("Failed to find project root: Could not convert path to string")
        .to_string()
}
