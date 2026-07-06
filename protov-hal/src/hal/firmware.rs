//! Embassy-boot firmware update backend (Embassy a.rs / b.rs patterns).

use core::cell::RefCell;

use defmt::{error, info, warn};
use embassy_boot_rp::{AlignedBuffer, BlockingFirmwareUpdater, FirmwareUpdaterConfig, State};
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_storage::nor_flash::NorFlash;

use embassy_embedded_hal::flash::partition::BlockingPartition;

use protov_core::model::DfuEvent;
use protov_nvm::{FLASH_SIZE, FWUP_SIGNATURE_LEN, PUBLIC_KEY};
use static_cell::StaticCell;

use crate::hal::watchdog;

pub type InnerFlash = Flash<'static, FLASH, Blocking, FLASH_SIZE>;
pub type FlashBus = Mutex<NoopRawMutex, RefCell<InnerFlash>>;
type DfuPartition<'a> = BlockingPartition<'a, NoopRawMutex, InnerFlash>;

pub type BoardFirmwareCtx = FirmwareCtx<'static, DfuPartition<'static>, DfuPartition<'static>>;

static FLASH_BUS: StaticCell<FlashBus> = StaticCell::new();
static UPDATER_STATE: StaticCell<AlignedBuffer<1>> = StaticCell::new();
static FW_CTX: StaticCell<BoardFirmwareCtx> = StaticCell::new();

pub struct FirmwareCtx<'d, DFU, STATE>
where
    DFU: NorFlash,
    STATE: NorFlash,
{
    pub updater: BlockingFirmwareUpdater<'d, DFU, STATE>,
    session_active: bool,
}

fn with_flash<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    watchdog::pause_for_flash();
    let result = f();
    watchdog::resume_after_flash();
    result
}

/// Init flash updater and confirm swap on boot (Embassy b.rs).
pub fn init(flash: embassy_rp::Peri<'static, FLASH>) -> &'static mut BoardFirmwareCtx {
    let flash = InnerFlash::new_blocking(flash);
    let flash_bus = FLASH_BUS.init(Mutex::new(RefCell::new(flash)));
    let state_aligned = UPDATER_STATE.init(AlignedBuffer([0; 1]));

    let ctx = FW_CTX.init_with(|| {
        let config = FirmwareUpdaterConfig::from_linkerfile_blocking(flash_bus, flash_bus);
        let updater = BlockingFirmwareUpdater::new(config, &mut state_aligned.0);
        FirmwareCtx::new(updater)
    });

    on_boot(&mut ctx.updater);
    ctx
}

impl<'d, DFU, STATE> FirmwareCtx<'d, DFU, STATE>
where
    DFU: NorFlash,
    STATE: NorFlash,
{
    pub fn new(updater: BlockingFirmwareUpdater<'d, DFU, STATE>) -> Self {
        Self {
            updater,
            session_active: false,
        }
    }

    pub fn dfu_prepare(&mut self) -> Result<(), ()> {
        with_flash(|| {
            self.updater.prepare_update().map(|_| ()).map_err(|e| {
                warn!("prepare_update: {:?}", defmt::Debug2Format(&e));
            })
        })?;
        self.session_active = true;
        Ok(())
    }

    pub fn dfu_write_block(&mut self, offset: u32, data: &[u8], len: u32) -> Result<(), ()> {
        if !self.session_active {
            return Err(());
        }
        let len = len as usize;
        if len == 0 || len > data.len() {
            return Err(());
        }
        static mut WRITE_BUF: AlignedBuffer<4096> = AlignedBuffer([0; 4096]);
        let buf = unsafe { &mut *core::ptr::addr_of_mut!(WRITE_BUF) };
        if len > buf.0.len() {
            return Err(());
        }
        buf.0[..len].copy_from_slice(&data[..len]);
        with_flash(|| {
            self.updater
                .write_firmware(offset as usize, &buf.0[..len])
                .map_err(|e| {
                    warn!("write_firmware @{}: {:?}", offset, defmt::Debug2Format(&e));
                })
        })
    }

    pub fn dfu_verify_apply(
        &mut self,
        len: u32,
        signature: [u8; FWUP_SIGNATURE_LEN],
    ) -> Result<(), ()> {
        if !self.session_active {
            return Err(());
        }
        match with_flash(|| {
            self.updater
                .verify_and_mark_updated(PUBLIC_KEY, &signature, len)
                .map_err(|e| {
                    warn!("verify_and_mark_updated: {:?}", defmt::Debug2Format(&e));
                })
        }) {
            Ok(()) => {
                self.session_active = false;
                Ok(())
            }
            Err(()) => {
                self.session_active = false;
                Err(())
            }
        }
    }

    pub fn dfu_abort(&mut self) {
        self.session_active = false;
    }
}

pub fn dfu_reset_after_verify() -> ! {
    info!("Firmware verified; resetting");
    cortex_m::peripheral::SCB::sys_reset();
}

pub fn on_boot<'d, DFU, STATE>(updater: &mut BlockingFirmwareUpdater<'d, DFU, STATE>)
where
    DFU: NorFlash,
    STATE: NorFlash,
{
    with_flash(|| match updater.get_state() {
        Ok(State::Swap) => match updater.mark_booted() {
            Ok(()) => info!("Confirmed firmware swap"),
            Err(e) => error!("mark_booted failed: {:?}", defmt::Debug2Format(&e)),
        },
        Ok(State::DfuDetach) => {
            // SCPI FWUP does not use USB DFU detach; clear stale magic so the next
            // reset is not treated as a failed trial boot.
            if let Err(e) = updater.mark_booted() {
                error!("clear DfuDetach failed: {:?}", defmt::Debug2Format(&e));
            }
        }
        Ok(State::Revert) => {
            warn!("Bootloader reverted to previous firmware");
        }
        Ok(State::Boot) => {}
        Err(e) => error!("failed to get update state: {:?}", defmt::Debug2Format(&e)),
    });
}

pub fn dfu_hardware_event(
    task: protov_core::model::HardwareTask,
    fw: &mut BoardFirmwareCtx,
    payload: &[u8],
) -> Option<DfuEvent> {
    use protov_core::model::HardwareTask;

    match task {
        HardwareTask::DfuPrepare => {
            if fw.dfu_prepare().is_ok() {
                Some(DfuEvent::PrepareComplete)
            } else {
                Some(DfuEvent::PrepareFailed)
            }
        }
        HardwareTask::DfuWriteBlock { offset, len } => {
            if fw.dfu_write_block(offset, payload, len).is_ok() {
                Some(DfuEvent::BlockWriteComplete { offset, len })
            } else {
                Some(DfuEvent::BlockWriteFailed)
            }
        }
        HardwareTask::DfuVerifyApply { len, signature } => {
            match fw.dfu_verify_apply(len, signature) {
                Ok(()) => Some(DfuEvent::VerifyApplyComplete),
                Err(()) => Some(DfuEvent::VerifyApplyFailed),
            }
        }
        _ => None,
    }
}
