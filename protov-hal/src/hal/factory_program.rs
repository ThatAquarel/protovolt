//! One-shot factory sector programming (feature `factory-program`).

#[cfg(feature = "factory-program")]
use defmt::info;

#[cfg(feature = "factory-program")]
use protov_nvm::{FACTORY_RECORD_SIZE, factory_erase_size, factory_flash_offset};

use crate::hal::firmware::InnerFlash;

#[cfg(feature = "factory-program")]
static FACTORY_IMAGE: [u8; FACTORY_RECORD_SIZE] =
    include!(concat!(env!("OUT_DIR"), "/factory_record.rs"));

pub fn program_if_requested(flash: &mut InnerFlash) -> Result<(), ()> {
    #[cfg(feature = "factory-program")]
    {
        let offset = factory_flash_offset() as u32;
        let end = (factory_flash_offset() + factory_erase_size()) as u32;
        super::firmware::with_flash(|| {
            flash.blocking_erase(offset, end).map_err(|e| {
                defmt::warn!("factory erase failed: {:?}", defmt::Debug2Format(&e));
            })?;
            flash.blocking_write(offset, &FACTORY_IMAGE).map_err(|e| {
                defmt::warn!("factory write failed: {:?}", defmt::Debug2Format(&e));
            })
        })?;
        info!("factory sector programmed");
    }
    #[cfg(not(feature = "factory-program"))]
    let _ = flash;
    Ok(())
}
