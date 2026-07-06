use core::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    Io(String),
    Timeout,
    InvalidUtf8,
}

impl core::fmt::Display for TransportError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "transport I/O error: {msg}"),
            Self::Timeout => write!(f, "transport timeout"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in response"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TransportError {}

pub trait Transport {
    fn write_all(&mut self, data: &[u8]) -> Result<(), TransportError>;
    fn read_line(&mut self, timeout: Duration) -> Result<String, TransportError>;
}

/// [`Transport`] over any `Read + Write` stream.
#[cfg(feature = "std")]
pub struct IoTransport<RW> {
    inner: RW,
}

#[cfg(feature = "std")]
impl<RW: std::io::Read + std::io::Write> IoTransport<RW> {
    pub fn new(inner: RW) -> Self {
        Self { inner }
    }
}

#[cfg(feature = "std")]
impl<RW: std::io::Read + std::io::Write> Transport for IoTransport<RW> {
    fn write_all(&mut self, data: &[u8]) -> Result<(), TransportError> {
        std::io::Write::write_all(&mut self.inner, data)
            .map_err(|e| TransportError::Io(e.to_string()))
    }

    fn read_line(&mut self, timeout: Duration) -> Result<String, TransportError> {
        let deadline = std::time::Instant::now() + timeout;
        let mut buf = Vec::new();
        let mut scratch = [0u8; 256];
        loop {
            if std::time::Instant::now() >= deadline {
                return Err(TransportError::Timeout);
            }
            match std::io::Read::read(&mut self.inner, &mut scratch) {
                Ok(0) => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Ok(n) => {
                    buf.extend_from_slice(&scratch[..n]);
                    if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                        let line = &buf[..pos];
                        return core::str::from_utf8(line)
                            .map(|s| s.trim_end_matches('\r').to_string())
                            .map_err(|_| TransportError::InvalidUtf8);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(TransportError::Io(e.to_string())),
            }
        }
    }
}
