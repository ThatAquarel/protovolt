use core::time::Duration;
use std::collections::VecDeque;

use protov_scpi::{ClientError, FWUP_SIGNATURE_LEN, ScpiClient, Transport, TransportError};

struct MockTransport {
    written: Vec<Vec<u8>>,
    responses: VecDeque<String>,
}

impl MockTransport {
    fn new(responses: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            written: Vec::new(),
            responses: responses.into_iter().map(Into::into).collect(),
        }
    }
}

impl Transport for MockTransport {
    fn write_all(&mut self, data: &[u8]) -> Result<(), TransportError> {
        self.written.push(data.to_vec());
        Ok(())
    }

    fn read_line(&mut self, _timeout: Duration) -> Result<String, TransportError> {
        self.responses.pop_front().ok_or(TransportError::Timeout)
    }
}

#[test]
fn query_sends_line_and_returns_response() {
    let transport = MockTransport::new(["ProtoV,1.0"]);
    let mut client = ScpiClient::new(transport);
    let resp = client.query("*IDN?").unwrap();
    assert_eq!(resp, "ProtoV,1.0");
}

#[test]
fn write_ok_accepts_ok_or_empty() {
    let transport = MockTransport::new(["OK"]);
    let mut client = ScpiClient::new(transport);
    client.write_ok("*RST").unwrap();
}

#[test]
fn fwup_upload_sequence() {
    let transport = MockTransport::new(["OK", "RECV,4096,0", "OK", "OK", "VERIFIED,4096"]);
    let mut client = ScpiClient::new(transport);
    let image = vec![0u8; 4096];
    let sig = [0u8; FWUP_SIGNATURE_LEN];
    client.fwup_upload(&image, &sig, 4096).unwrap();
}

#[test]
fn write_ok_rejects_unexpected() {
    let transport = MockTransport::new(["ERROR"]);
    let mut client = ScpiClient::new(transport);
    let err = client.write_ok("*RST").unwrap_err();
    assert!(matches!(err, ClientError::UnexpectedResponse(_)));
}
