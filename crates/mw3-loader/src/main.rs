//! CLI: `teknogods.ini`, game argv helpers, optional update check.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use mw3_loader::{
    client_connect_args, client_lan_args, dedicated_args, fetch_mw3_update_check,
    parse_update_response, sp_args, sp_coop_host_args, TeknogodsIni, DEFAULT_MW3_UPDATE_URL,
};

#[derive(Parser, Debug)]
#[command(
    name = "mw3-loader",
    version,
    about = "TeknoMW3 launcher helpers (ini, argv, update-check)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Dump parsed INI to stdout (`section\tkey=value`)
    IniDump {
        #[arg(default_value = "teknogods.ini")]
        path: PathBuf,
    },
    /// Print one key (empty if missing)
    IniGet {
        #[arg(default_value = "teknogods.ini")]
        path: PathBuf,
        section: String,
        key: String,
    },
    /// Set a key and write the file (merges with existing content)
    IniSet {
        #[arg(default_value = "teknogods.ini")]
        path: PathBuf,
        section: String,
        key: String,
        value: String,
    },
    /// Print game argv for dedicated server (`iw5mp_server.exe`)
    ArgsDedicated {
        port: String,
        #[arg(long)]
        usekeys: bool,
    },
    /// Print argv for MP LAN (`iw5mp.exe`)
    ArgsClientLan {
        #[arg(long)]
        usekeys: bool,
    },
    /// Print argv for MP direct connect (`+server host:port`)
    ArgsClientConnect { host: String, port: String },
    /// Print argv for SP coop host (`+server ip:0`, `iw5sp.exe`)
    ArgsSpCoop { ip: String },
    /// Print argv for plain SP (empty)
    ArgsSp,
    /// Fetch update-check URL and print `version` and `url`
    UpdateCheck {
        #[arg(long, default_value = DEFAULT_MW3_UPDATE_URL)]
        url: String,
        #[arg(long, default_value_t = 15000)]
        timeout_ms: u64,
    },
    /// Parse update-check body from stdin (no network)
    UpdateParse,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    match cli.command {
        Command::IniDump { path } => {
            let ini = TeknogodsIni::load(&path)?;
            for (sec, map) in ini.sections() {
                for (k, v) in map {
                    println!("{sec}\t{k}={v}");
                }
            }
        }
        Command::IniGet { path, section, key } => {
            let ini = TeknogodsIni::load(&path)?;
            println!("{}", ini.get_or_empty(&section, &key));
        }
        Command::IniSet {
            path,
            section,
            key,
            value,
        } => {
            let mut ini = if path.exists() {
                TeknogodsIni::load(&path)?
            } else {
                TeknogodsIni::new()
            };
            ini.set(&section, &key, &value);
            ini.save(&path)?;
        }
        Command::ArgsDedicated { port, usekeys } => {
            println!("{}", dedicated_args(&port, usekeys));
        }
        Command::ArgsClientLan { usekeys } => {
            println!("{}", client_lan_args(usekeys));
        }
        Command::ArgsClientConnect { host, port } => {
            println!("{}", client_connect_args(&host, &port));
        }
        Command::ArgsSpCoop { ip } => {
            println!("{}", sp_coop_host_args(&ip));
        }
        Command::ArgsSp => {
            print!("{}", sp_args());
        }
        Command::UpdateCheck { url, timeout_ms } => {
            let u = fetch_mw3_update_check(&url, Duration::from_millis(timeout_ms))?;
            println!("version={}", u.version_label);
            println!("url={}", u.open_url);
        }
        Command::UpdateParse => {
            let mut body = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut body)?;
            let u = parse_update_response(&body)?;
            println!("version={}", u.version_label);
            println!("url={}", u.open_url);
        }
    }
    Ok(())
}
