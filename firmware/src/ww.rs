//! WireWeaver server: implements `nucleo_g0b1re_api::NucleoG0b1reApi` (defined in `../nucleo_g0b1re_api/src/lib.rs`) for
//! [ServerState] and serves it over RTT (the debug probe).
//!
//! To add a method, property or stream: declare it in the API trait, then implement it here, `ww_codegen!` below
//! regenerates the dispatch code and the compiler lists what is missing. Extend [ServerState] with the peripherals
//! and data the implementations need. See https://github.com/vhrdtech/wire_weaver (`docs/`, `examples_mcu/`).

use cnt::cnt;
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_stm32::gpio::Output;
use embassy_time::Timer;
use rtt_target::{DownChannel, UpChannel};
use static_cell::StaticCell;
use wire_weaver::prelude::*;
use wire_weaver::{MessageSink, WireWeaverAsyncApiBackend};
use ww_device::rtt::{RttBuffers, RttConfig, RttServer, rtt_server};
use ww_device::{EmbassyClock, LinkConfig};
use ww_error::CommonError;
use ww_firmware_info::ResetRecord;
use ww_firmware_info::{BUILD_ID_MAX_LEN, FirmwareSummary};
use ww_settings::shrink_wrap::ArrayString;
use ww_user_info::{MAX_NAME_LEN, UserInfoCaps, UserInfoError};

/// Longest WireWeaver message the device accepts and can reply with (reported to the host). The RTT up channel
/// (`ww_up` in main.rs) must be at least `MAX_MESSAGE_LEN + 5` bytes, better twice that.
const MAX_MESSAGE_LEN: usize = 1024;

/// Everything the API implementation needs: peripherals, drivers, state.
pub struct ServerState {
    pub led: Output<'static>,
    /// Reset cause and crash record read at boot (src/last_reset.rs)
    pub last_reset: bedrock_crash::Snapshot,
    /// Settings in flash (src/settings.rs)
    pub settings: crate::settings::SettingsStore,
}

// Implementations of the NucleoG0b1reApi methods. `method_model = "_=immediate"`: they return a result right away
// (`RpcResult` can also defer the reply, see wire_weaver::RpcResult).
impl ServerState {
    async fn led_on(&mut self, _cx: &mut Context<'_, impl EventOut>) -> RpcResult<()> {
        info!("led on");
        cnt!(led_on_calls: u32);
        self.led.set_high();
        Ready(())
    }

    async fn led_off(&mut self, _cx: &mut Context<'_, impl EventOut>) -> RpcResult<()> {
        info!("led off");
        cnt!(led_off_calls: u32);
        self.led.set_low();
        Ready(())
    }
}

// Values of the API's `const` items, served without a handler and read once per connection by a host.
// ww_firmware_info::FirmwareInfo (`firmware`): which firmware this is; the optional BUILD_RECORD is left out (`omit`
// below).
impl server_impl::NucleoG0b1reApiConsts for ServerState {
    // name, version, git commit, build time and profile, from build.rs (ww_firmware_info::build::emit())
    const FIRMWARE_FIRMWARE: FirmwareSummary<'static> = ww_firmware_info::firmware_summary!();

    // ww_user_info::UserInfo (`label`): the name is kept in flash, there is no room for notes (`omit` below)
    const LABEL_CAPABILITIES: UserInfoCaps = UserInfoCaps {
        name_max: MAX_NAME_LEN as u16,
        notes_max: 0,
        persistent: true,
    };

    // GNU build id, kept in flash by memory.x: a host finds this firmware's ELF by it (known at link time: a `fn`,
    // `const_model` below)
    fn firmware_build_id(&self) -> MaxLen<RefVec<'_, u8>, BUILD_ID_MAX_LEN> {
        MaxLen::new_unchecked(RefVec::new_bytes(ww_firmware_info::build_id()))
    }
}

// ww_firmware_info::LastReset (`last_reset`): why the device last reset, and the crash record after a fault or panic.
impl ServerState {
    async fn last_reset_last_reset(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
    ) -> RpcResult<ResetRecord<'_>> {
        Ready(crate::last_reset::record(&self.last_reset))
    }

    async fn last_reset_boot_count(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
    ) -> RpcResult<u32> {
        Ready(self.last_reset.boot_count())
    }

    async fn last_reset_clear(&mut self, _cx: &mut Context<'_, impl EventOut>) -> RpcResult<()> {
        bedrock_crash::clear(&mut self.last_reset);
        Ready(())
    }
}

// ww_user_info::UserInfo (`label`): the name a user gives this unit. A write is in flash before the call answers,
// and the flash is written only when the name changed.
impl ServerState {
    async fn set_label_name(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        value: MaxLen<&'_ str, MAX_NAME_LEN>,
    ) -> SetResult<UserInfoError<'_>> {
        let Ok(label) = ArrayString::try_from(*value) else {
            return SetError(UserInfoError::TooLong {
                max: MAX_NAME_LEN as u16,
            });
        };
        self.settings.set(|s| s.label = label);
        match self.settings.persist().await {
            Ok(()) => Set,
            Err(_e) => {
                defmt::error!("label not saved: {}", defmt::Debug2Format(&_e));
                SetError(UserInfoError::Common(CommonError::Fault { code: 1 }))
            }
        }
    }

    async fn get_label_name(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
    ) -> GetResult<MaxLen<&'_ str, MAX_NAME_LEN>, UserInfoError<'_>> {
        Value(MaxLen::new_unchecked(self.settings.get().label.as_str()))
    }
}

mod server_impl {
    // Re-parses the API trait and generates request dispatch + serdes for ServerState.
    // To see the generated code, add: debug_to_file = "./target/ww_server.rs"
    wire_weaver::ww_codegen!(
        nucleo_g0b1re_api :: NucleoG0b1reApi for super::ServerState,
        server = true, no_alloc = true, use_async = true,
        method_model = "_=immediate",
        property_model = "_=get_set",
        introspect = "with_docs",
        const_model = "BUILD_ID=fn",
        omit = ["firmware.BUILD_RECORD", "label.notes"],
    );
}

impl WireWeaverAsyncApiBackend for ServerState {
    type Medium = ();

    async fn process_bytes<'a>(
        &mut self,
        out: &mut EventWriter<'_, impl MessageSink>,
        medium: (),
        data: &[u8],
        scratch: &'a mut [u8],
    ) -> Result<&'a [u8], shrink_wrap::Error> {
        self.process_request_bytes(data, scratch, out, medium).await
    }

    fn version(&self) -> FullVersion<'_> {
        nucleo_g0b1re_api::NUCLEO_G_0_B_1_RE_API_FULL_GID
    }
}

fn link_config() -> LinkConfig<'static> {
    LinkConfig::new(
        nucleo_g0b1re_api::NUCLEO_G_0_B_1_RE_API_FULL_GID,
        server_impl::api_hash(),
        ww_client_server::COMPACT_VERSION,
    )
}

static RTT_BUFFERS: StaticCell<RttBuffers<MAX_MESSAGE_LEN>> = StaticCell::new();

pub fn start(spawner: Spawner, up: UpChannel, down: DownChannel, state: ServerState) {
    let server = rtt_server(
        link_config(),
        up,
        down,
        EmbassyClock,
        RttConfig::default(),
        RTT_BUFFERS.init(RttBuffers::new()),
    );
    spawner.spawn(unwrap!(server_task(server, state)));
}

#[embassy_executor::task]
async fn server_task(mut server: RttServer<'static, EmbassyClock>, mut state: ServerState) {
    loop {
        // also while no host is connected: a change made just before it left still reaches the flash
        match select(server.wait(), Timer::at(state.settings.due_at())).await {
            Either::First(ready) => {
                server.handle(ready, &mut state).await;
            }
            Either::Second(()) => state.settings.persist_if_due().await,
        }
    }
}
