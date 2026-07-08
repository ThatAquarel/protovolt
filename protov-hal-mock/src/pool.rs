use std::sync::{Arc, Mutex};

use protov_core::config::DEFAULT_MANUFACTURING_DATE;

use crate::device::{MockDevice, MockIdentity};

pub const MAX_MOCK_DEVICES: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct SlotProfile {
    pub serial: &'static str,
    pub fw_version: &'static str,
    pub hw_version: &'static str,
    pub manufacturing_date: (u16, u8, u8),
    pub flash_unique_id: [u8; 8],
    pub serial_signature: [u8; 64],
}

const fn slot_signature(seed: u8) -> [u8; 64] {
    [seed; 64]
}

const fn slot_flash_uid(seed: u8) -> [u8; 8] {
    [seed; 8]
}

pub const SLOT_PROFILES: [SlotProfile; MAX_MOCK_DEVICES] = [
    SlotProfile {
        serial: "550e8400",
        fw_version: "1.0.0",
        hw_version: "A.1",
        manufacturing_date: DEFAULT_MANUFACTURING_DATE,
        flash_unique_id: slot_flash_uid(0x55),
        serial_signature: slot_signature(0x55),
    },
    SlotProfile {
        serial: "32983fe4",
        fw_version: "1.0.1",
        hw_version: "B.2",
        manufacturing_date: DEFAULT_MANUFACTURING_DATE,
        flash_unique_id: slot_flash_uid(0x32),
        serial_signature: slot_signature(0x32),
    },
    SlotProfile {
        serial: "deadbeef",
        fw_version: "0.9.0",
        hw_version: "A.0",
        manufacturing_date: DEFAULT_MANUFACTURING_DATE,
        flash_unique_id: slot_flash_uid(0xDE),
        serial_signature: slot_signature(0xDE),
    },
    SlotProfile {
        serial: "a1b2c3d4",
        fw_version: "1.2.3",
        hw_version: "C.1",
        manufacturing_date: DEFAULT_MANUFACTURING_DATE,
        flash_unique_id: slot_flash_uid(0xA1),
        serial_signature: slot_signature(0xA1),
    },
];

pub fn identity_from_profile(profile: SlotProfile) -> MockIdentity {
    MockIdentity::new(
        profile.serial,
        profile.fw_version,
        profile.hw_version,
        profile.manufacturing_date,
        profile.flash_unique_id,
        profile.serial_signature,
    )
}

struct PoolInner {
    devices: [MockDevice; MAX_MOCK_DEVICES],
    in_use: [bool; MAX_MOCK_DEVICES],
    rebooting: [bool; MAX_MOCK_DEVICES],
}

impl PoolInner {
    fn new() -> Self {
        let devices = std::array::from_fn(|index| {
            MockDevice::with_identity(identity_from_profile(SLOT_PROFILES[index]))
        });
        Self {
            devices,
            in_use: [false; MAX_MOCK_DEVICES],
            rebooting: [false; MAX_MOCK_DEVICES],
        }
    }

    fn acquire(&mut self) -> Option<usize> {
        for (index, used) in self.in_use.iter_mut().enumerate() {
            if !*used && !self.rebooting[index] {
                *used = true;
                return Some(index);
            }
        }
        None
    }

    fn release(&mut self, index: usize) {
        if index >= MAX_MOCK_DEVICES {
            return;
        }
        let was_in_use = self.in_use[index];
        self.in_use[index] = false;
        if was_in_use && !self.rebooting[index] {
            self.devices[index].reset_to_profile(identity_from_profile(SLOT_PROFILES[index]));
        }
    }

    fn release_all(&mut self) {
        for index in 0..MAX_MOCK_DEVICES {
            self.in_use[index] = false;
            self.rebooting[index] = false;
            self.devices[index].reset_to_profile(identity_from_profile(SLOT_PROFILES[index]));
        }
    }

    fn finish_reboot(&mut self, index: usize) {
        if index >= MAX_MOCK_DEVICES {
            return;
        }
        self.devices[index].complete_firmware_reboot(identity_from_profile(SLOT_PROFILES[index]));
        self.rebooting[index] = false;
        self.in_use[index] = false;
    }

    fn device_mut(&mut self, index: usize) -> Option<&mut MockDevice> {
        (index < MAX_MOCK_DEVICES).then_some(&mut self.devices[index])
    }
}

#[derive(Clone)]
pub struct DevicePool {
    inner: Arc<Mutex<PoolInner>>,
}

impl DevicePool {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(PoolInner::new())),
        }
    }

    pub fn acquire(&self) -> Option<usize> {
        self.inner.lock().unwrap().acquire()
    }

    pub fn release(&self, index: usize) {
        self.inner.lock().unwrap().release(index);
    }

    pub fn release_all(&self) {
        self.inner.lock().unwrap().release_all();
    }

    pub fn begin_fwup_reboot(&self, index: usize) {
        const BOOT_DELAY: std::time::Duration = std::time::Duration::from_secs(2);
        let pool = self.clone();
        {
            let mut inner = self.inner.lock().unwrap();
            inner.rebooting[index] = true;
            inner.in_use[index] = true;
        }
        tokio::spawn(async move {
            tokio::time::sleep(BOOT_DELAY).await;
            pool.inner.lock().unwrap().finish_reboot(index);
        });
    }

    pub fn with_device<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&mut MockDevice) -> R,
    {
        let mut guard = self.inner.lock().unwrap();
        guard.device_mut(index).map(f)
    }
}

impl Default for DevicePool {
    fn default() -> Self {
        Self::new()
    }
}
