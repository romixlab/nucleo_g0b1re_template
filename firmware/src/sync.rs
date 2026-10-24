//! The shared clock: a host's steer points discipline the local clock (embassy-time on TIM4) onto UTC, so LD4
//! (src/led.rs) breathes in phase with other boards. Served as `clock` (`ww_timebase::Clock`) by src/ww/clock.rs; this
//! module only holds the static, as src/led.rs holds the indication.
//!
//! No sync line and no oscillator backend: the 64 MHz comes from the HSI16 through the PLL, the shared clock is
//! corrected in software only (bedrock_sync slews it).

/// The board's shared clock, read by the LED task every tick and steered from `clock.steer` (src/ww/clock.rs).
pub static SYNC: bedrock_sync::Sync = bedrock_sync::Sync::new(CONFIG);

/// bedrock_sync's defaults with a lock tolerance for RTT: the host polls RTT through the probe, so round trips are
/// 6–13 ms and the error bound the host sends with each steer point (half the shortest round trip plus twice the
/// fit's scatter) is about 4 ms, more than the default 2 ms (sized for a full-speed USB board). 10 ms keeps a
/// slower probe or a busy PC locked; a 3 s breath looks in phase far beyond that.
const CONFIG: bedrock_sync::Config = bedrock_sync::Config {
    lock_within_us: 10_000,
    ..bedrock_sync::Config::DEFAULT
};
