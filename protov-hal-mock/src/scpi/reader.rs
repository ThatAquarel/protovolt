//! HAL-style SCPI line and FWUP binary block reassembly.

use protov_scpi::{
    FWUP_DATA_PREFIX, FWUP_MAX_BLOCK_LEN, LINE_BUF, ScpiCommand, parse_command,
    parse_definite_block_at,
};

use crate::device::MockDevice;

/// Stream buffer must hold a full FWUP header plus one max-sized block (USB reassembles
/// incrementally; mock/WebSocket may deliver the whole line in one chunk).
const RX_BUF: usize = FWUP_MAX_BLOCK_LEN + 64;

enum RxMode {
    Ascii,
    FwupPayload { payload_len: usize, received: usize },
}

pub struct ScpiReader {
    buf: heapless::Vec<u8, RX_BUF>,
    mode: RxMode,
}

impl ScpiReader {
    pub fn new() -> Self {
        Self {
            buf: heapless::Vec::new(),
            mode: RxMode::Ascii,
        }
    }

    pub fn reset(&mut self) {
        self.buf.clear();
        self.mode = RxMode::Ascii;
    }

    pub fn push(&mut self, data: &[u8]) {
        for &b in data {
            let _ = self.buf.push(b);
        }
    }

    pub fn wants_more(&self) -> bool {
        match self.mode {
            RxMode::FwupPayload {
                payload_len,
                received,
            } => received < payload_len,
            RxMode::Ascii => try_fwup_header(&self.buf).is_some() || fwup_data_pending(&self.buf),
        }
    }

    pub fn drain(&mut self, device: &mut MockDevice) -> Vec<String> {
        let mut responses = Vec::new();
        loop {
            match self.mode {
                RxMode::Ascii => {
                    if let Some((header_end, payload_len)) = try_fwup_header(&self.buf) {
                        if payload_len > FWUP_MAX_BLOCK_LEN {
                            self.buf.clear();
                            break;
                        }
                        device.firmware.clear_payload();
                        let inline = self.buf.len().saturating_sub(header_end);
                        let take = inline.min(payload_len);
                        if take > 0 {
                            device
                                .firmware
                                .write_payload_at(0, &self.buf.as_slice()[header_end..header_end + take]);
                        }
                        consume_front(&mut self.buf, header_end + take);
                        if take == payload_len {
                            if let Some(text) = self.finish_fwup_block(device, payload_len) {
                                responses.push(text);
                            }
                        } else {
                            self.mode = RxMode::FwupPayload {
                                payload_len,
                                received: take,
                            };
                        }
                        continue;
                    }

                    if fwup_data_pending(&self.buf) {
                        if self.buf.len() >= RX_BUF {
                            self.buf.clear();
                        }
                        break;
                    }

                    if let Some(newline) = self.buf.iter().position(|&b| b == b'\n') {
                        let mut line_bytes = heapless::Vec::<u8, LINE_BUF>::new();
                        for &b in &self.buf[..=newline] {
                            let _ = line_bytes.push(b);
                        }
                        consume_front(&mut self.buf, newline + 1);
                        let line = trim_line(&line_bytes);
                        if !line.is_empty() {
                            responses.extend(self.dispatch_line(device, line));
                        }
                        continue;
                    }

                    if self.buf.len() >= RX_BUF {
                        self.buf.clear();
                    }
                    break;
                }
                RxMode::FwupPayload {
                    payload_len,
                    received,
                } => {
                    if self.buf.is_empty() {
                        break;
                    }
                    let need = payload_len - received;
                    let take = need.min(self.buf.len());
                    device
                        .firmware
                        .write_payload_at(received, &self.buf.as_slice()[..take]);
                    consume_front(&mut self.buf, take);
                    let received = received + take;
                    if received == payload_len {
                        self.mode = RxMode::Ascii;
                        if let Some(text) = self.finish_fwup_block(device, payload_len) {
                            responses.push(text);
                        }
                        continue;
                    }
                    self.mode = RxMode::FwupPayload {
                        payload_len,
                        received,
                    };
                    break;
                }
            }
        }
        responses
    }

    fn finish_fwup_block(&mut self, device: &mut MockDevice, _payload_len: usize) -> Option<String> {
        super::router::dispatch_command(device, ScpiCommand::FwupData)
    }

    fn dispatch_line(&mut self, device: &mut MockDevice, line: &str) -> Vec<String> {
        if let Some(cmd) = parse_command(line) {
            if matches!(cmd, ScpiCommand::FwupData) {
                return Vec::new();
            }
            if let Some(text) = super::router::dispatch_command(device, cmd) {
                return vec![text];
            }
        }
        Vec::new()
    }
}

fn consume_front(buf: &mut heapless::Vec<u8, RX_BUF>, n: usize) {
    let len = buf.len();
    if n >= len {
        buf.clear();
        return;
    }
    for i in 0..len - n {
        buf[i] = buf[i + n];
    }
    buf.truncate(len - n);
}

fn trim_line(line_bytes: &[u8]) -> &str {
    let end = line_bytes
        .iter()
        .position(|&b| b == b'\n' || b == b'\r')
        .unwrap_or(line_bytes.len());
    core::str::from_utf8(&line_bytes[..end])
        .ok()
        .map(str::trim)
        .unwrap_or("")
}

fn try_fwup_header(buf: &[u8]) -> Option<(usize, usize)> {
    let prefix = FWUP_DATA_PREFIX.as_bytes();
    if buf.len() < prefix.len() || !buf.starts_with(prefix) {
        return None;
    }
    let mut i = prefix.len();
    if buf.get(i) == Some(&b' ') {
        i += 1;
    }
    if buf.get(i) != Some(&b'#') {
        return None;
    }
    let (hdr_len, payload_len) = parse_definite_block_at(&buf[i..])?;
    Some((i + hdr_len, payload_len))
}

fn fwup_data_pending(buf: &[u8]) -> bool {
    let prefix = FWUP_DATA_PREFIX.as_bytes();
    if buf.is_empty() {
        return false;
    }
    if buf.len() < prefix.len() {
        return prefix.starts_with(buf);
    }
    buf.starts_with(prefix)
}
