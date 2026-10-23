//! `g0b1re`: the command line over `nucleo_g0b1re_client`. Argument parsing and printing live here, everything
//! that talks to the board is the library's.
//!
//! Shell completion, with the serials of the probes attached right now: `source <(COMPLETE=bash g0b1re)` in
//! `~/.bashrc`, `COMPLETE=zsh g0b1re`, `COMPLETE=fish g0b1re | source`.

use std::ffi::OsStr;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use anstream::println;
use anstyle::{AnsiColor, Color, Style};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::engine::{ArgValueCandidates, ArgValueCompleter, PathCompleter, ValueCompleter};
use clap_complete::{CompleteEnv, CompletionCandidate};
use indicatif::{ProgressBar, ProgressStyle};
use nucleo_g0b1re_client::{Error, Nucleo, PROBE_RS_CHIP, Probe, api_line, firmware_line};

/// `--version`: `0.1.0 (a1b2c3d-dirty, built 23 Oct 2026 10:20 UTC)`, stamped by `build.rs`.
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("G0B1RE_GIT"),
    ", built ",
    env!("G0B1RE_BUILT"),
    ")"
);

const BOLD: Style = Style::new().bold();
const DIM: Style = Style::new().dimmed();
const GREEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
const YELLOW: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow)));
const RED: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

/// Nucleo G0B1RE over its debug probe: the LED, the firmware, the board's label, the last reset.
#[derive(Parser)]
#[command(
    name = "g0b1re",
    version = VERSION,
    after_help = "Shell completion: source <(COMPLETE=bash g0b1re), COMPLETE=zsh g0b1re, COMPLETE=fish g0b1re | source"
)]
struct Cli {
    /// Serial number of the debug probe, or a part of it (default: the only probe attached)
    #[arg(short, long, env = "G0B1RE_PROBE", global = true, value_name = "SERIAL", add = ArgValueCandidates::new(probe_serials))]
    probe: Option<String>,
    /// probe-rs chip name of the MCU on the board
    #[arg(long, env = "G0B1RE_CHIP", global = true, default_value = PROBE_RS_CHIP)]
    chip: String,
    /// The firmware's ELF, for a fast attach (the RTT control block address is read from it instead of scanning
    /// the RAM, about 1.5 s through an ST-LINK)
    #[arg(long, env = "G0B1RE_ELF", global = true, value_name = "FILE", add = ArgValueCompleter::new(elf_files))]
    elf: Option<PathBuf>,
    /// SWD clock in Hz (default: the probe's)
    #[arg(long, global = true, value_name = "HZ")]
    speed_hz: Option<u32>,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List the debug probes attached to this PC
    Probes,
    /// Firmware, API and last reset of the board
    Info,
    /// The user LED (LD4)
    #[command(subcommand)]
    Led(LedCmd),
    /// Show the board's label, or store a new one (kept in the board's flash)
    Label {
        /// The new label, at most 32 bytes
        name: Option<String>,
    },
    /// Why the board last reset, with the crash record after a fault or panic
    LastReset {
        /// Forget the record and zero the boot counter afterwards
        #[arg(long)]
        clear: bool,
    },
}

#[derive(Subcommand)]
enum LedCmd {
    /// Switch the LED on
    On,
    /// Switch the LED off
    Off,
    /// Blink the LED a few times
    Blink {
        /// How many times
        #[arg(short = 'n', long, default_value_t = 3)]
        count: u32,
        /// One on-off cycle, in milliseconds
        #[arg(long, default_value_t = 500, value_name = "MS")]
        period: u64,
    },
}

/// Serials of the attached probes for `--probe`, the probe's name as help; empty on any error.
fn probe_serials() -> Vec<CompletionCandidate> {
    probe_rs::probe::list::Lister::new()
        .list_all()
        .into_iter()
        .filter_map(|p| {
            let serial = p.serial_number.clone()?;
            Some(CompletionCandidate::new(serial).help(Some(p.identifier.clone().into())))
        })
        .collect()
}

/// Firmware ELF files for `--elf`: directories to walk into, files without an extension (cargo's output) or `.elf`.
fn elf_files(current: &OsStr) -> Vec<CompletionCandidate> {
    PathCompleter::file()
        .filter(|path| {
            path.is_dir()
                || path
                    .extension()
                    .is_none_or(|ext| ext.eq_ignore_ascii_case("elf"))
        })
        .complete(current)
}

impl Cli {
    fn probe(&self) -> Probe {
        Probe {
            chip: self.chip.clone(),
            serial: self.probe.clone(),
            speed_hz: self.speed_hz,
            elf: self.elf.clone(),
        }
    }
}

/// Connects, with a spinner on stderr while it takes (a RAM scan for the RTT control block is slow on an ST-LINK).
async fn connect(cli: &Cli) -> anyhow::Result<Nucleo> {
    let spinner = if std::io::stderr().is_terminal() {
        let s = ProgressBar::new_spinner().with_message(format!("connecting to {}", cli.chip));
        s.set_style(ProgressStyle::with_template("{spinner} {msg}").expect("template"));
        s.enable_steady_tick(Duration::from_millis(80));
        Some(s)
    } else {
        None
    };
    let result = Nucleo::connect(&cli.probe()).await;
    if let Some(s) = spinner {
        s.finish_and_clear();
    }
    Ok(result?)
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match &cli.command {
        Cmd::Probes => {
            let probes = probe_rs::probe::list::Lister::new().list_all();
            if probes.is_empty() {
                println!("{YELLOW}no debug probe attached{YELLOW:#}");
            }
            for p in probes {
                let serial = p.serial_number.as_deref().unwrap_or("-");
                println!(
                    "{BOLD}{serial}{BOLD:#}  {}  {DIM}{:04x}:{:04x}{DIM:#}",
                    p.identifier, p.vendor_id, p.product_id
                );
            }
        }
        Cmd::Info => {
            let board = connect(&cli).await?;
            let info = board.info();
            println!("{BOLD}API{BOLD:#}        {}", api_line(info));
            let fw = board.firmware().await?;
            println!("{BOLD}firmware{BOLD:#}   {}", firmware_line(&fw));
            match board.build_id().await {
                Ok(id) => println!("{BOLD}build id{BOLD:#}   {}", hex(&id)),
                Err(Error::Unimplemented { .. }) => {}
                Err(e) => return Err(e.into()),
            }
            let label = board.label().await?;
            if label.is_empty() {
                println!("{BOLD}label{BOLD:#}      {DIM}(none){DIM:#}");
            } else {
                println!("{BOLD}label{BOLD:#}      {label}");
            }
            let boots = board.boot_count().await?;
            let reset = board.last_reset().await?;
            println!("{BOLD}last reset{BOLD:#} {reset:?}  {DIM}boot #{boots}{DIM:#}");
            board.disconnect().await;
        }
        Cmd::Led(cmd) => {
            let board = connect(&cli).await?;
            match cmd {
                LedCmd::On => board.led_on().await?,
                LedCmd::Off => board.led_off().await?,
                LedCmd::Blink { count, period } => {
                    board.blink(*count, Duration::from_millis(*period)).await?
                }
            }
            board.disconnect().await;
        }
        Cmd::Label { name } => {
            let board = connect(&cli).await?;
            match name {
                Some(name) => {
                    board.set_label(name).await?;
                    println!("{GREEN}label stored{GREEN:#}: {name}");
                }
                None => println!("{}", board.label().await?),
            }
            board.disconnect().await;
        }
        Cmd::LastReset { clear } => {
            let board = connect(&cli).await?;
            let boots = board.boot_count().await?;
            let reset = board.last_reset().await?;
            println!("{reset:#?}");
            println!("{DIM}boots since cleared: {boots}{DIM:#}");
            if *clear {
                board.clear_last_reset().await?;
                println!("{GREEN}cleared{GREEN:#}");
            }
            board.disconnect().await;
        }
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() -> ExitCode {
    CompleteEnv::with_factory(Cli::command).complete();
    let cli = Cli::parse();
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            anstream::eprintln!("{RED}error{RED:#}: {e}");
            return ExitCode::FAILURE;
        }
    };
    match rt.block_on(run(cli)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            anstream::eprintln!("{RED}error{RED:#}: {e:#}");
            ExitCode::FAILURE
        }
    }
}
