use core::time::Duration;

use crate::block::{encode_fwup_appl_line, encode_fwup_data_line};
use crate::client::{Transport, TransportError};
use crate::command::FWUP_SIGNATURE_LEN;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const STAR_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
    Transport(TransportError),
    UnexpectedResponse(String),
    DeviceError(String),
    Timeout,
}

impl core::fmt::Display for ClientError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "{e}"),
            Self::UnexpectedResponse(r) => write!(f, "unexpected response: {r}"),
            Self::DeviceError(e) => write!(f, "device error: {e}"),
            Self::Timeout => write!(f, "operation timed out"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ClientError {}

impl From<TransportError> for ClientError {
    fn from(value: TransportError) -> Self {
        Self::Transport(value)
    }
}

pub struct ScpiClient<T: Transport> {
    transport: T,
    default_timeout: Duration,
}

impl<T: Transport> ScpiClient<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            default_timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    pub fn query(&mut self, cmd: &str) -> Result<String, ClientError> {
        let line = if cmd.ends_with('\n') {
            cmd.to_string()
        } else {
            format!("{cmd}\n")
        };
        self.transport.write_all(line.as_bytes())?;
        self.read_response(self.default_timeout)
    }

    pub fn write_ok(&mut self, cmd: &str) -> Result<(), ClientError> {
        let resp = self.query(cmd)?;
        if resp.is_empty() || resp == "OK" {
            Ok(())
        } else {
            Err(ClientError::UnexpectedResponse(resp))
        }
    }

    pub fn err(&mut self) -> Result<String, ClientError> {
        self.query("SYST:ERR?")
    }

    pub fn fwup_stat(&mut self) -> Result<String, ClientError> {
        self.query("SYST:FWUP:STAT?")
    }

    pub fn fwup_star(&mut self, size: u32) -> Result<(), ClientError> {
        self.write_ok(&format!("SYST:FWUP:STAR {size}"))
    }

    pub fn fwup_data(&mut self, block: &[u8]) -> Result<(), ClientError> {
        let line = encode_fwup_data_line(block);
        self.transport.write_all(&line)?;
        let resp = self.read_response(self.default_timeout)?;
        if resp.is_empty() || resp == "OK" {
            Ok(())
        } else {
            Err(ClientError::UnexpectedResponse(resp))
        }
    }

    pub fn fwup_appl(&mut self, signature: &[u8; FWUP_SIGNATURE_LEN]) -> Result<(), ClientError> {
        let line = encode_fwup_appl_line(signature);
        self.transport.write_all(line.as_bytes())?;
        let resp = self.read_response(self.default_timeout)?;
        if resp.is_empty() || resp == "OK" {
            Ok(())
        } else {
            Err(ClientError::UnexpectedResponse(resp))
        }
    }

    pub fn fwup_upload(
        &mut self,
        image: &[u8],
        signature: &[u8; FWUP_SIGNATURE_LEN],
        chunk: usize,
    ) -> Result<(), ClientError> {
        let chunk = chunk.clamp(1, crate::command::FWUP_MAX_BLOCK_LEN);
        self.fwup_star(image.len() as u32)?;
        self.wait_receiving(image.len() as u32, STAR_TIMEOUT)?;

        let mut offset = 0;
        while offset < image.len() {
            let end = (offset + chunk).min(image.len());
            self.fwup_data(&image[offset..end])?;
            offset = end;
        }

        self.fwup_appl(signature)?;
        self.wait_verified(STAR_TIMEOUT)?;
        Ok(())
    }

    fn read_response(&mut self, timeout: Duration) -> Result<String, ClientError> {
        self.transport.read_line(timeout).map_err(ClientError::from)
    }

    fn wait_receiving(&mut self, total: u32, timeout: Duration) -> Result<(), ClientError> {
        let deadline = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < deadline {
            let stat = self.fwup_stat()?;
            if stat.starts_with("RECV,") || stat.starts_with("READY,") {
                return Ok(());
            }
            if stat == "ERROR" {
                return Err(ClientError::DeviceError(stat));
            }
            if stat.starts_with("PREPARE,") {
                Self::poll_delay(200);
                continue;
            }
            Self::poll_delay(100);
        }
        let _ = total;
        Err(ClientError::Timeout)
    }

    fn wait_verified(&mut self, timeout: Duration) -> Result<(), ClientError> {
        let deadline = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < deadline {
            let stat = self.fwup_stat()?;
            if stat.starts_with("VERIFIED,") || stat.starts_with("FLASHING") {
                return Ok(());
            }
            if stat == "ERROR" {
                return Err(ClientError::DeviceError(stat));
            }
            Self::poll_delay(200);
        }
        Err(ClientError::Timeout)
    }

    /// Back off between FWUP stat polls. WASM has no threads; rely on JS transport timeouts.
    fn poll_delay(ms: u64) {
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::sleep(Duration::from_millis(ms));
        #[cfg(target_arch = "wasm32")]
        let _ = ms;
    }
}
