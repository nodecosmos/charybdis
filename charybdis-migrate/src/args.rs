use clap::Parser;

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
}