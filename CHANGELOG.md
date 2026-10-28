# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.9] - 2026-10-28

### Changed

- API 0.5.0, breaking: `SiliconRevision` knows ES0548's revisions Z and Y besides A, in a shape that grows by adding
  at the end (`Unknown` first, then A, Z, Y; a host older than the firmware reads a newer revision as `Newer`, with
  the code in `rev_id`). A revision a later errata sheet adds is then a compatible change. `g0b1re variant` and
  Python's `variant()` name Z and Y. Hosts built for API 0.4 need rebuilding for this firmware. (API-60)
- wire_weaver pinned to 668483bd (EVO-10: the evolution check knows the enum catch-all); snapshot 0.5.0 saved with it. (API-60)

## [0.1.8] - 2026-10-25

### Changed

- Upgraded from embedded_bedrock's firmware template da9b351: `bedrock_fw.json` records the suite's answers (host,
  board preset, probe, pins, LED mode, clock sync), with which the generator writes the firmware, the board map,
  the logo, CI and hooks as they are here. The firmware README says `just setup` and the right backup-register size,
  AGENTS.md describes the root workspace, the board is named "Nucleo-G0B1RE" throughout, the Python crate's lib has no
  Rust test binary.
- `api_snapshots/` has the released API 0.4.0.

## [0.1.7] - 2026-10-24

### Added

- LD4 breathes by default (PWM on TIM2_CH1, embedded_bedrock's `bedrock_led`); `led_on` / `led_off` hold it solid on
  or off until the next reset, `settings led-at-boot on` starts it solid on.
- `leds` in the API (`ww_indication::Indication`): mode (normal, essential, off), brightness (lowest to highest),
  identify (blinks LD4 for a while). `g0b1re leds show | mode | brightness | identify`; Python `leds_mode`,
  `brightness`, `identify()`.
- `clock` in the API (`ww_timebase::Clock`): the host steers the board's shared clock onto its UTC by itself when it
  connects, and the breath follows shared time, so boards on PCs with NTP breathe in phase. `g0b1re clock [--watch N]`
  waits for the lock and prints the residual against the PC's UTC; Python `wait_clock_locked()`, `clock_check()` and
  `examples/clock.py` (`just py-example clock`). README "LED and clock" has the residual measured over RTT.
- `api_snapshots/` has the released API 0.3.0, so the check judges changes against it.

### Changed

- API 0.4.0 (additions only: hosts built for 0.3 keep working).
- embassy's time driver is TIM4, named explicitly, so TIM2 stays free for LD4's PWM.

## [0.1.6] - 2026-10-24

### Added

- `pins` in the API (0.3.0): the firmware serves `ww_pin_diag::PinDiag` on every pin the board map marks free (50
  pins, named as in `docs/board/nucleo_g0b1re-pins.md`) and PC13 (B1) to read. A pin stays analog until a host sets
  or reads it and goes back when the host releases the pins, disconnects or goes silent; the SWD pins, the virtual
  COM port, the LED, the clock pins and reset are not in it. `just test` checks the list against the pin table.
- `just pins`: every free pin live on the board's headers (`ww pins tui` over RTT with the board map as its preset):
  drive, pull and read them. `docs/board/nucleo_g0b1re-pins-test.toml` is the pin test of a bare board (`ww pins test
  --plan`). README: Pins, with a jumper test.

## [0.1.5] - 2026-10-24

### Added

- The device suite under `device` in the API (the `Device` sub-trait other WireWeaver devices serve): `firmware`,
  `last_reset`, `board` (NUCLEO-G0B1RE, revision, serial = unique id), `uid` (the 96-bit unique id), `label`,
  `counters` (the `cnt` counters, RAM and backup registers, with a reset per buffer), `log` (the defmt log kept in RAM
  across a reset, served by `ww_defmt_logger`; `just run` still shows it live), `variant` (device id, silicon
  revision and flash size from DBGMCU and the flash size register), `settings` (`led_at_boot`, `reset`), `boot`
  (boot state, reboot into the firmware or the STM32 ROM bootloader), `dfu.node` (the update node, profile
  `InPlace`). `led_on` / `led_off` stay.
- Boot-loop rescue: after four boots in a row that each ended within 10 s the firmware resets into the STM32 ROM
  bootloader, and the next firmware reports it as `Recovery` in `device.dfu.node`.
- `g0b1re firmware`, `board`, `uid`, `variant`, `counters [--reset ram|bkp]`, `log [-f]`, `settings show |
  led-at-boot on|off | reset`, `boot state | reboot [--updater]`, `dfu`, `report` (everything in one text for a bug
  report); `g0b1re info` shows the board and the chip too. Counters and the log are decoded with the firmware's ELF,
  found by the build id the board reports; `--elf` names one, `--force` takes it with another build id.
- `just run` and `just flash` put the ELF they flash into the ELF directory (`cnt elf add`), so `g0b1re` decodes
  counters and the log from anywhere.
- Python: `board()`, `uid()`, `variant()`, `counters()`, `reset_counters("ram"|"bkp")`, `log()`, `led_at_boot`,
  `reset_settings()`, `boot_state()`, `reboot(updater=False)`, `dfu_node()`, `report()`; the example
  `examples/device.py` (`just py-example device`).
- README: the Device section, a step per part of the contract, and how a Nucleo reaches the ROM bootloader.

### Changed

- API 0.2.0, a breaking change: `firmware`, `last_reset` and `label` moved under `device`. Hosts built for 0.1 do not
  talk to this firmware; rebuild them from this API.
- `g0b1re label` without a name prints `(none)` for an empty label; Python's `last_reset()` is plain lines, like
  `g0b1re last-reset`.

### Fixed

- The firmware's cargo runner and every recipe that uses the probe (`just run`, `just cnt`, `just flash`, `just
  attach`) take only the Nucleo-64's own ST-LINK/V2-1 (`--probe 0483:374b`), never another probe on the PC: with
  only someone else's probe attached, `cargo run` flashed this firmware onto that board. With two Nucleos,
  `G0B1RE_PROBE=SERIAL` (in the environment or in `.env`) picks one.

## [0.1.4] - 2026-10-24

### Added

- API evolution check: `nucleo_g0b1re_api/api_snapshots/` holds the snapshot of the released 0.1.0, and a test of
  the API crate (WireWeaver's `wire_weaver_evolution`, strict) fails when the API changes without the version bump
  the change needs, naming the version to use. `just api-check` runs it, `just test` and CI too; `just api-save`
  records a release; `.githooks/pre-commit` refuses a commit that touches the API crate while the check fails. `just
  hooks` (also the last step of `just setup`) turns it on after the hooks already in `.git/hooks/` and leaves
  `core.hooksPath` alone. `just rename` saves the renamed crate's first snapshot. README: Changing the API.
- CI uses `actions/checkout@v5` (v4 runs on the deprecated Node 20).

## [0.1.3] - 2026-10-24

### Added

- `just setup` (`scripts/setup.py`, run with uv): sets up a fresh Arch, Ubuntu or Windows PC for the project (rustup
  with the firmware's target, probe-rs, flip-link, cnt, ww, boardmap); it prints its plan first, naming the
  installer scripts it fetches and runs; steps that need root are printed, `--root` runs them; `just setup --check`
  reports what is missing, also as a CI job. On Windows `just`
  runs the recipes with cmd, so Git's sh need not be on PATH, and `just install` uses cargo install.

## [0.1.2] - 2026-10-23

### Added

- `cnt run` as an alternative cargo runner (TPL-9): a commented `runner = "cnt run --chip STM32G0B1RE"` under the
  probe-rs one in `firmware/.cargo/config.toml`, and a `just cnt` recipe; both flash, reset and show the defmt log
  plus the live event counters as they change, without switching the default runner. `cnt` bumped to 0.5.

## [0.1.1] - 2026-10-23

### Added

- Logo at the top of the README (light and dark), drawn by a Typst template in `docs/logo/` with the board name, the
  chip label and the subtitle as parameters, plus a 1280 x 640 social preview; `just logo`.

## [0.1.0] - 2026-10-23

### Added

- First version of the template: firmware for the Nucleo-G0B1RE generated by embedded_bedrock's firmware template
  (embassy, defmt, WireWeaver over RTT, settings store with the board's label, `cnt` counters with backup registers,
  crash capture), the `nucleo_g0b1re_api` API crate, the `nucleo_g0b1re_client` library, the `g0b1re` command line
  with live shell completion, the `nucleo_g0b1re` Python module, `just` recipes, CI and `just rename`.
- Board map (`docs/board/`): top and bottom views with every header pin coloured by what it is for, solder bridges
  with their defaults, the power tree and a pin table, drawn by boardmap from `nucleo_g0b1re.toml`; `just board`.
- README and `docs/board/nucleo_g0b1re.toml` document the USER button B1 (PC13, pulled up and read `1` released,
  `0` pressed, SB20), the LEDs (LD4 user on PA5 through a MOSFET, also `TIM2_CH1_ETR` for a PWM breathe; LD1 COM
  and LD3 PWR, driven by the ST-LINK, not the target MCU), and how the CN4 jumpers pick the ST-LINK's SWD target
  (on-board STM32G0B1RE, or external through CN11); facts from UM2324 Rev 5, the MB1360-G0B1RE-C02 schematic and
  the STM32G0B1xx datasheet.
