//! `nucleo_g0b1re` Python module: [Device] wraps `nucleo_g0b1re_client::Nucleo` with blocking calls on a tokio
//! runtime of its own, so scripts and notebooks need no asyncio. Everything the module does, the library does.

use std::path::PathBuf;
use std::time::Duration;

use nucleo_g0b1re_client::{Error, Nucleo, PROBE_RS_CHIP, Probe, api_line, firmware_line};
use pyo3::exceptions::{PyConnectionError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use tokio::runtime::Runtime;

/// Talk to the Nucleo G0B1RE firmware through its debug probe.
#[pymodule]
fn nucleo_g0b1re(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Device>()?;
    m.add("PROBE_RS_CHIP", PROBE_RS_CHIP)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

fn to_py(e: Error) -> PyErr {
    match e {
        Error::Connect(_) => PyConnectionError::new_err(e.to_string()),
        Error::Invalid(_) | Error::Label(_) => PyValueError::new_err(e.to_string()),
        Error::Link { .. } | Error::Unimplemented { .. } => PyRuntimeError::new_err(e.to_string()),
    }
}

/// A board reached through a debug probe (RTT). Connects when created; use it as a context manager or call
/// `disconnect()` to release the probe.
///
/// `Device(probe_serial=None, chip="STM32G0B1RETx", elf=None, speed_hz=None)`: `probe_serial` picks a probe when
/// several are attached, `elf` (the firmware's ELF) makes attaching fast.
#[pyclass]
struct Device {
    runtime: Runtime,
    board: Option<Nucleo>,
}

impl Device {
    fn board(&self) -> PyResult<&Nucleo> {
        self.board
            .as_ref()
            .ok_or_else(|| PyConnectionError::new_err("disconnected"))
    }

    fn block<T>(&self, f: impl Future<Output = nucleo_g0b1re_client::Result<T>>) -> PyResult<T> {
        self.runtime.block_on(f).map_err(to_py)
    }
}

#[pymethods]
impl Device {
    #[new]
    #[pyo3(signature = (probe_serial=None, chip=None, elf=None, speed_hz=None))]
    fn new(
        probe_serial: Option<String>,
        chip: Option<String>,
        elf: Option<PathBuf>,
        speed_hz: Option<u32>,
    ) -> PyResult<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()?;
        let probe = Probe {
            chip: chip.unwrap_or_else(|| PROBE_RS_CHIP.to_string()),
            serial: probe_serial,
            speed_hz,
            elf,
        };
        let board = runtime.block_on(Nucleo::connect(&probe)).map_err(to_py)?;
        Ok(Self {
            runtime,
            board: Some(board),
        })
    }

    /// API name and version the board reported.
    fn api(&self) -> PyResult<String> {
        Ok(api_line(self.board()?.info()))
    }

    /// Switch the user LED (LD4) on.
    fn led_on(&self) -> PyResult<()> {
        self.block(self.board()?.led_on())
    }

    /// Switch the user LED off.
    fn led_off(&self) -> PyResult<()> {
        self.block(self.board()?.led_off())
    }

    /// Blink the LED `count` times, `period_ms` per on-off cycle.
    #[pyo3(signature = (count=3, period_ms=500))]
    fn blink(&self, count: u32, period_ms: u64) -> PyResult<()> {
        self.block(self.board()?.blink(count, Duration::from_millis(period_ms)))
    }

    /// Name, version, git commit, build time and profile of the running firmware, as text.
    fn firmware(&self) -> PyResult<String> {
        Ok(firmware_line(&self.block(self.board()?.firmware())?))
    }

    /// GNU build id of the running firmware, hex.
    fn build_id(&self) -> PyResult<String> {
        let id = self.block(self.board()?.build_id())?;
        Ok(id.iter().map(|b| format!("{b:02x}")).collect())
    }

    /// The name stored on the board.
    #[getter]
    fn label(&self) -> PyResult<String> {
        self.block(self.board()?.label())
    }

    /// Store a name on the board (at most 32 bytes, in flash when the call returns).
    #[setter]
    fn set_label(&self, label: &str) -> PyResult<()> {
        self.block(self.board()?.set_label(label))
    }

    /// Why the board last reset, with the crash record after a fault or panic, as text.
    fn last_reset(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self.block(self.board()?.last_reset())?))
    }

    /// Boots since the boot counter was cleared.
    fn boot_count(&self) -> PyResult<u32> {
        self.block(self.board()?.boot_count())
    }

    /// Forget the crash record and zero the boot counter.
    fn clear_last_reset(&self) -> PyResult<()> {
        self.block(self.board()?.clear_last_reset())
    }

    /// Release the probe. The object can't be used afterwards.
    fn disconnect(&mut self) -> PyResult<()> {
        if let Some(board) = self.board.take() {
            self.runtime.block_on(board.disconnect());
        }
        Ok(())
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    #[pyo3(signature = (*_args))]
    fn __exit__(&mut self, _args: &Bound<'_, pyo3::types::PyTuple>) -> PyResult<()> {
        self.disconnect()
    }

    fn __repr__(&self) -> String {
        match &self.board {
            Some(b) => format!("Device({})", api_line(b.info())),
            None => "Device(disconnected)".to_string(),
        }
    }
}
