//! Dedicated server launcher CLI. Full launch is **Windows-only**; other hosts support `--print-va` for PE checks.

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "mw3-dedi-launch",
    version,
    about = "Launch iw5mp_server.exe with TeknoMW3.dll patch (Windows)"
)]
struct Cli {
    /// Path to `iw5mp_server.exe`
    #[arg(default_value = "iw5mp_server.exe")]
    exe: PathBuf,

    /// DLL file name written into the process (max 12 chars + NUL; default `TeknoMW3.dll`)
    #[arg(long, default_value = "TeknoMW3.dll")]
    dll: String,

    /// Print PE patch virtual address and exit (no process spawn)
    #[arg(long)]
    print_va: bool,

    /// Game command line (pass through, e.g. `+dedicated 1 +set net_port 27015`)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    game_args: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    if cli.print_va {
        match mw3_dedi_launch::locate_steam_api_dll_va(&cli.exe) {
            Ok(va) => println!("steam_api.dll string VA: 0x{va:X}"),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return;
    }

    #[cfg(windows)]
    {
        use mw3_dedi_launch::win::{launch_suspended_patch_resume, LaunchOptions};
        let args = cli.game_args.join(" ");
        match launch_suspended_patch_resume(LaunchOptions {
            exe_path: &cli.exe,
            game_args: &args,
            dll_name: &cli.dll,
        }) {
            Ok(code) => std::process::exit(code as i32),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(windows))]
    {
        eprintln!("mw3-dedi-launch: process launch is only implemented on Windows.");
        eprintln!("Use --print-va with a path to iw5mp_server.exe to validate PE parsing.");
        std::process::exit(2);
    }
}
