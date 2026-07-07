//! In-memory firmware image store for mock DFU hardware tasks.

use protov_core::model::DfuEvent;
use protov_core::model::HardwareTask;
use protov_scpi::{FWUP_MAX_BLOCK_LEN, FWUP_MAX_IMAGE_SIZE, FWUP_SIGNATURE_LEN};

pub struct MockFirmwareStore {
    image: Vec<u8>,
    received: u32,
    session_active: bool,
    payload: [u8; FWUP_MAX_BLOCK_LEN],
    payload_len: usize,
}

impl Default for MockFirmwareStore {
    fn default() -> Self {
        Self {
            image: Vec::new(),
            received: 0,
            session_active: false,
            payload: [0; FWUP_MAX_BLOCK_LEN],
            payload_len: 0,
        }
    }
}

impl MockFirmwareStore {
    pub fn set_payload(&mut self, data: &[u8]) {
        let len = data.len().min(FWUP_MAX_BLOCK_LEN);
        self.payload[..len].copy_from_slice(&data[..len]);
        self.payload_len = len;
    }

    pub fn write_payload_at(&mut self, offset: usize, data: &[u8]) {
        let end = offset.saturating_add(data.len());
        if end > FWUP_MAX_BLOCK_LEN {
            return;
        }
        self.payload[offset..end].copy_from_slice(data);
        self.payload_len = self.payload_len.max(end);
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload[..self.payload_len]
    }

    pub fn payload_len(&self) -> usize {
        self.payload_len
    }

    pub fn clear_payload(&mut self) {
        self.payload_len = 0;
    }

    pub fn abort(&mut self) {
        self.session_active = false;
        self.received = 0;
        self.payload_len = 0;
        self.image.clear();
    }

    pub fn dfu_hardware_event(task: HardwareTask, store: &mut Self) -> Option<DfuEvent> {
        match task {
            HardwareTask::DfuPrepare => {
                if store.prepare().is_ok() {
                    Some(DfuEvent::PrepareComplete)
                } else {
                    Some(DfuEvent::PrepareFailed)
                }
            }
            HardwareTask::DfuWriteBlock { offset, len } => {
                let len_usize = len as usize;
                let payload = store.payload()[..len_usize.min(store.payload_len())].to_vec();
                if store.write_block(offset, len, &payload).is_ok() {
                    Some(DfuEvent::BlockWriteComplete { offset, len })
                } else {
                    Some(DfuEvent::BlockWriteFailed)
                }
            }
            HardwareTask::DfuVerifyApply { len, signature } => {
                if store.verify_apply(len, signature).is_ok() {
                    Some(DfuEvent::VerifyApplyComplete)
                } else {
                    Some(DfuEvent::VerifyApplyFailed)
                }
            }
            _ => None,
        }
    }

    fn prepare(&mut self) -> Result<(), ()> {
        self.image.clear();
        self.received = 0;
        self.session_active = true;
        Ok(())
    }

    fn write_block(&mut self, offset: u32, len: u32, payload: &[u8]) -> Result<(), ()> {
        if !self.session_active {
            return Err(());
        }
        let len = len as usize;
        if len == 0 || len > payload.len() {
            return Err(());
        }
        let end = offset as usize + len;
        if end > FWUP_MAX_IMAGE_SIZE as usize {
            return Err(());
        }
        if self.image.len() < end {
            self.image.resize(end, 0);
        }
        self.image[offset as usize..end].copy_from_slice(&payload[..len]);
        self.received = end as u32;
        Ok(())
    }

    fn verify_apply(&mut self, len: u32, _signature: [u8; FWUP_SIGNATURE_LEN]) -> Result<(), ()> {
        if !self.session_active {
            return Err(());
        }
        if len != self.received {
            return Err(());
        }
        self.session_active = false;
        Ok(())
    }
}
