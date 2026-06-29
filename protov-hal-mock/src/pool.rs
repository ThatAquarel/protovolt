use std::sync::{Arc, Mutex};

use crate::device::{MockDevice, MockIdentity};

pub const MAX_MOCK_DEVICES: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct SlotProfile {
    pub serial: &'static str,
    pub fw_version: &'static str,
    pub hw_version: &'static str,
}

pub const SLOT_PROFILES: [SlotProfile; MAX_MOCK_DEVICES] = [
    SlotProfile {
        serial: "550e8400",
        fw_version: "1.0.0",
        hw_version: "A.1",
    },
    SlotProfile {
        serial: "32983fe4",
        fw_version: "1.0.1",
        hw_version: "B.2",
    },
    SlotProfile {
        serial: "deadbeef",
        fw_version: "0.9.0",
        hw_version: "A.0",
    },
    SlotProfile {
        serial: "a1b2c3d4",
        fw_version: "1.2.3",
        hw_version: "C.1",
    },
];

pub fn identity_from_profile(profile: SlotProfile) -> MockIdentity {
    MockIdentity::new(profile.serial, profile.fw_version, profile.hw_version)
}

struct PoolInner {
    devices: [MockDevice; MAX_MOCK_DEVICES],
    in_use: [bool; MAX_MOCK_DEVICES],
}

impl PoolInner {
    fn new() -> Self {
        let devices = std::array::from_fn(|index| {
            MockDevice::with_identity(identity_from_profile(SLOT_PROFILES[index]))
        });
        Self {
            devices,
            in_use: [false; MAX_MOCK_DEVICES],
        }
    }

    fn acquire(&mut self) -> Option<usize> {
        for (index, used) in self.in_use.iter_mut().enumerate() {
            if !*used {
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
        if was_in_use {
            self.devices[index].reset_to_profile(identity_from_profile(SLOT_PROFILES[index]));
        }
    }

    fn release_all(&mut self) {
        for index in 0..MAX_MOCK_DEVICES {
            self.in_use[index] = false;
            self.devices[index].reset_to_profile(identity_from_profile(SLOT_PROFILES[index]));
        }
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
