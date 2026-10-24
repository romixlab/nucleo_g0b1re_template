//! `device`: the device contract (`nucleo_g0b1re_api::Device`): board, ids, variant, counters, log, settings, boot
//! and the update node. Counters and the log are decoded with the firmware's ELF, found by the build id the board
//! reports (`target/`, `WW_ELF_PATH`, the ELF directory `ww elf add` fills) or given as `elf`.

use std::path::{Path, PathBuf};

use defmt_decoder::{DecodeError, StreamDecoder, Table};
use wire_weaver::prelude::UVlq64;
use wire_weaver_client::{StreamCursor, StreamEvent};

use crate::{Error, Nucleo, Result, call, call_fallible, read, write};

pub use nucleo_g0b1re_api::{SiliconRevision, Variant};
pub use ww_board_info::BoardSummaryOwned;
pub use ww_counters::CounterBuffer;
pub use ww_dfu::{BootState, BootTarget, NodeInfoOwned};
pub use ww_report::counters::{CounterValue, CountersReport};
pub use ww_report::log::LogReport;
pub use ww_uid::UniqueIdOwned;

impl Nucleo {
    /// Board name, revision and serial number (the chip's unique id in hex).
    pub async fn board(&self) -> Result<BoardSummaryOwned> {
        read("BoardInfo::BOARD", self.raw.device().board().read_BOARD()).await
    }

    /// The board's unique ids: the STM32 96-bit unique id.
    pub async fn uid(&self) -> Result<Vec<UniqueIdOwned>> {
        let ids = read("Uid::IDS", self.raw.device().uid().read_IDS()).await?;
        Ok(ids.into_inner())
    }

    /// The chip on the board: device id, silicon revision, flash size.
    pub async fn variant(&self) -> Result<Variant> {
        read("Device::variant", self.raw.device().read_variant()).await
    }

    /// Every `cnt` counter of the firmware, RAM and backup registers, decoded with the firmware's ELF (`elf`, or
    /// the one of the board's build id). `force`: use `elf` although its build id differs.
    pub async fn counters(&self, elf: Option<&Path>, force: bool) -> Result<CountersReport> {
        let info = read("Counters::INFO", self.raw.device().counters().read_INFO()).await?;
        let elf = resolve_elf(elf, info.build_id.as_slice(), force)?;
        ww_report::counters::read(&self.contract.counters, info, &elf)
            .await
            .map_err(|e| Error::Decode(format!("counters: {e:#}")))
    }

    /// Zeroes the counters of one buffer.
    pub async fn reset_counters(&self, buffer: CounterBuffer) -> Result<()> {
        call_fallible(
            "Counters::reset",
            self.raw.device().counters().reset(buffer),
        )
        .await
    }

    /// The defmt log the board keeps (since boot, and from before a crash), decoded with the firmware's ELF. Leaves
    /// the board's store as it is: running it again shows the same lines and the new ones.
    pub async fn log(&self, elf: Option<&Path>, force: bool) -> Result<LogReport> {
        let table = self.defmt_table(elf, force).await?;
        ww_report::log::read(&self.contract.defmt, &table)
            .await
            .map_err(|e| Error::Decode(format!("log: {e:#}")))
    }

    /// Like [Nucleo::log], then every new line as it is logged, handed to `on_line`, until the link drops.
    pub async fn follow_log(
        &self,
        elf: Option<&Path>,
        force: bool,
        mut on_line: impl FnMut(&str),
    ) -> Result<()> {
        let table = self.defmt_table(elf, force).await?;
        let device = self.raw.device();
        let log = device.log();
        let status = read("LogDefmt::frames_status", log.frames_status()).await?;
        let Some(session) = status.session else {
            return Err(Error::Decode("the board has no log session".to_string()));
        };
        let cursor = StreamCursor {
            session,
            pos: UVlq64(0),
        };
        let mut frames = log
            .frames_resume(cursor)
            .call()
            .await
            .map_err(|source| Error::Link {
                item: "LogDefmt::frames",
                source,
            })?;
        let mut decoder = table.new_stream_decoder();
        loop {
            let event = frames
                .recv_event()
                .await
                .map_err(|e| Error::Decode(format!("log stream: {e}")))?;
            match event {
                StreamEvent::Item { value, .. } => {
                    for line in decode(value.bytes.as_slice(), &mut *decoder) {
                        on_line(&line);
                    }
                }
                StreamEvent::Gap { units, cause, .. } => {
                    on_line(&format!("--- {units} log messages lost ({cause:?}) ---"))
                }
                StreamEvent::Closed | StreamEvent::Disconnected => return Ok(()),
                _ => {}
            }
        }
    }

    /// The defmt table of the firmware's ELF (`elf`, or the one of the board's build id).
    async fn defmt_table(&self, elf: Option<&Path>, force: bool) -> Result<Table> {
        let info = read("LogDefmt::DEFMT", self.raw.device().log().read_DEFMT()).await?;
        let elf = resolve_elf(elf, info.build_id.as_slice(), force)?;
        let bytes =
            std::fs::read(&elf).map_err(|e| Error::Elf(format!("{}: {e}", elf.display())))?;
        match Table::parse(&bytes) {
            Ok(Some(table)) => Ok(table),
            Ok(None) => Err(Error::Elf(format!("{} has no defmt table", elf.display()))),
            Err(e) => Err(Error::Elf(format!("{}: {e:#}", elf.display()))),
        }
    }

    /// The LED after boot: on or off.
    pub async fn led_at_boot(&self) -> Result<bool> {
        read(
            "Settings::led_at_boot",
            self.raw.device().settings().read_led_at_boot(),
        )
        .await
    }

    /// Sets the LED after boot; it is in flash when the call returns.
    pub async fn set_led_at_boot(&self, on: bool) -> Result<()> {
        write(
            "Settings::led_at_boot",
            self.raw.device().settings().write_led_at_boot(on),
            |e| Error::Device {
                item: "Settings::led_at_boot",
                message: format!("{e:?}"),
            },
        )
        .await
    }

    /// Every setting back to its default, the label too; in flash when the call returns.
    pub async fn reset_settings(&self) -> Result<()> {
        call_fallible("Settings::reset", self.raw.device().settings().reset()).await
    }

    /// The boot state: always `Confirmed`, the firmware is the only image.
    pub async fn boot_state(&self) -> Result<BootState> {
        call(
            "BootControl::boot_state",
            self.raw.device().boot().boot_state(),
        )
        .await
    }

    /// Resets the board into the firmware (`App`) or into the STM32 ROM bootloader (`Updater`). The board answers
    /// first, the reset follows: the connection is gone afterwards.
    pub async fn reboot(&self, into: BootTarget) -> Result<()> {
        call_fallible("BootControl::reboot", self.raw.device().boot().reboot(into)).await
    }

    /// Who the update node is: product, profile (`InPlace`), boot state, last result (`Recovery` after the ROM
    /// bootloader took over from a boot loop).
    pub async fn dfu_node(&self) -> Result<NodeInfoOwned> {
        call("UpdateNode::node", self.raw.device().dfu().node().node()).await
    }

    /// The whole device contract in one text, as `ww report` prints it: firmware, board, ids, label, last reset
    /// with the crash's addresses named, counters and the stored log, decoded with the ELF.
    pub async fn report(&self, elf: Option<PathBuf>, force: bool) -> String {
        let options = ww_report::report::ReportOptions {
            elf,
            force,
            defmt_table: None,
            tool: format!("g0b1re {}", env!("CARGO_PKG_VERSION")),
        };
        let report = ww_report::report::collect_report(&self.contract, &options).await;
        ww_report::report::render_report(&report)
    }
}

/// The ELF for `build_id` (or `elf` itself, checked against it unless `force`).
fn resolve_elf(elf: Option<&Path>, build_id: &[u8], force: bool) -> Result<PathBuf> {
    if build_id.is_empty() && elf.is_none() {
        return Err(Error::Elf(
            "the board reports no build id, so its ELF can't be found: pass the ELF".to_string(),
        ));
    }
    let resolved = cnt_core::elf::resolve(elf, Some(build_id), force)
        .map_err(|e| Error::Elf(format!("{e:#}")))?;
    Ok(resolved.path)
}

/// The lines `bytes` completes.
fn decode(bytes: &[u8], decoder: &mut dyn StreamDecoder) -> Vec<String> {
    decoder.received(bytes);
    let mut lines = vec![];
    loop {
        match decoder.decode() {
            Ok(frame) => lines.push(frame.display(false).to_string()),
            Err(DecodeError::UnexpectedEof) => break,
            Err(DecodeError::Malformed) => {
                lines.push("--- malformed frame (wrong ELF?) ---".to_string());
                break;
            }
        }
    }
    lines
}
