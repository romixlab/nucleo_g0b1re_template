//! WireWeaver API of the nucleo_g0b1re firmware (`../nucleo_g0b1re_api`), served by `../firmware/src/ww.rs`.
//!
//! The crate name and version identify the API on the wire (`NUCLEO_G_0_B_1_RE_API_FULL_GID`), so bump the version with every
//! change and follow the WireWeaver evolution rules to stay compatible with deployed devices and hosts:
//! https://github.com/vhrdtech/wire_weaver/blob/master/docs/evolution/rules.md
//! Data types used in the API go here too, with `#[derive_shrink_wrap]`.
#![no_std]

use wire_weaver::prelude::*;

/// nucleo_g0b1re device
#[ww_api_root]
pub trait NucleoG0b1reApi {
    /// Turn the LED on
    fn led_on();
    /// Turn the LED off
    fn led_off();

    /// Which firmware runs and its build id (the device contract)
    ww_impl!(firmware: ww_firmware_info::FirmwareInfo);
    /// Why the device last reset, with the crash record after a fault or panic
    ww_impl!(last_reset: ww_firmware_info::LastReset);
    /// Name the user gives this unit, kept in flash
    ww_impl!(label: ww_user_info::UserInfo);
}
