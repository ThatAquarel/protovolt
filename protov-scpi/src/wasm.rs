#![cfg(feature = "wasm")]

use std::time::Duration;

use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::client::{ClientError, ScpiClient, Transport, TransportError};
use crate::command::FWUP_SIGNATURE_LEN;

struct JsTransport {
    write_fn: Function,
    read_line_fn: Function,
}

impl Transport for JsTransport {
    fn write_all(&mut self, data: &[u8]) -> Result<(), TransportError> {
        let arr = js_sys::Uint8Array::from(data);
        self.write_fn
            .call1(&JsValue::NULL, &arr)
            .map_err(|e| TransportError::Io(format!("{e:?}")))?;
        Ok(())
    }

    fn read_line(&mut self, timeout: Duration) -> Result<String, TransportError> {
        let ms = timeout.as_millis().min(u32::MAX as u128) as u32;
        let value = self
            .read_line_fn
            .call1(&JsValue::NULL, &JsValue::from(ms))
            .map_err(|e| TransportError::Io(format!("{e:?}")))?;
        if value.is_null() || value.is_undefined() {
            return Err(TransportError::Timeout);
        }
        value.as_string().ok_or(TransportError::InvalidUtf8)
    }
}

fn client_err(e: ClientError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

#[wasm_bindgen]
pub struct WasmScpiClient {
    inner: ScpiClient<JsTransport>,
}

#[wasm_bindgen]
impl WasmScpiClient {
    #[wasm_bindgen(constructor)]
    pub fn new(write: Function, read_line: Function, timeout_ms: u32) -> Self {
        let transport = JsTransport {
            write_fn: write,
            read_line_fn: read_line,
        };
        let inner =
            ScpiClient::new(transport).with_timeout(Duration::from_millis(timeout_ms as u64));
        Self { inner }
    }

    pub fn query(&mut self, cmd: &str) -> Result<String, JsValue> {
        self.inner.query(cmd).map_err(client_err)
    }

    pub fn write_ok(&mut self, cmd: &str) -> Result<(), JsValue> {
        self.inner.write_ok(cmd).map_err(client_err)
    }

    pub fn err(&mut self) -> Result<String, JsValue> {
        self.inner.err().map_err(client_err)
    }

    pub fn fwup_stat(&mut self) -> Result<String, JsValue> {
        self.inner.fwup_stat().map_err(client_err)
    }

    pub fn fwup_star(&mut self, size: u32) -> Result<(), JsValue> {
        self.inner.fwup_star(size).map_err(client_err)
    }

    pub fn fwup_data(&mut self, block: &[u8]) -> Result<(), JsValue> {
        self.inner.fwup_data(block).map_err(client_err)
    }

    pub fn fwup_appl(&mut self, signature: &[u8]) -> Result<(), JsValue> {
        if signature.len() != FWUP_SIGNATURE_LEN {
            return Err(JsValue::from_str(&format!(
                "signature must be {FWUP_SIGNATURE_LEN} bytes"
            )));
        }
        let mut sig = [0u8; FWUP_SIGNATURE_LEN];
        sig.copy_from_slice(signature);
        self.inner.fwup_appl(&sig).map_err(client_err)
    }

    pub fn fwup_abor(&mut self) -> Result<(), JsValue> {
        self.inner.write_ok("SYST:FWUP:ABOR").map_err(client_err)
    }

    pub fn fwup_upload(
        &mut self,
        image: &[u8],
        signature: &[u8],
        chunk: usize,
    ) -> Result<(), JsValue> {
        if signature.len() != FWUP_SIGNATURE_LEN {
            return Err(JsValue::from_str(&format!(
                "signature must be {FWUP_SIGNATURE_LEN} bytes"
            )));
        }
        let mut sig = [0u8; FWUP_SIGNATURE_LEN];
        sig.copy_from_slice(signature);
        self.inner
            .fwup_upload(image, &sig, chunk)
            .map_err(client_err)
    }
}
