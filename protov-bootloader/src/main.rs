#![no_std]
#![no_main]

mod display;

use core::cell::RefCell;

use cortex_m_rt::{entry, exception};
use embassy_boot_rp::*;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::Duration;
use protov_nvm::FLASH_SIZE;

#[cfg(feature = "defmt")]
use defmt::info;
#[cfg(feature = "defmt")]
use defmt_rtt as _;

#[entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());

    let _display = display::DisplayHold::wake(
        p.PIN_28, p.PIN_17, p.PIN_21, p.PIN_18, p.PIN_19, p.PIN_16, p.SPI0,
    );

    #[cfg(feature = "defmt")]
    {
        // When debugging with RTT attached, avoids hard fault on early flash access.
        for _ in 0..10_000_000 {
            cortex_m::asm::nop();
        }
    }

    let flash = WatchdogFlash::<FLASH_SIZE>::start(p.FLASH, p.WATCHDOG, Duration::from_secs(8));
    let flash = Mutex::new(RefCell::new(flash));

    let config = BootLoaderConfig::from_linkerfile_blocking(&flash, &flash, &flash);
    let active_offset = config.active.offset();

    #[cfg(feature = "defmt")]
    {
        info!("active partition offset {=u32}", active_offset);
    }

    let bl: BootLoader = BootLoader::prepare(config);

    unsafe { bl.load(embassy_rp::flash::FLASH_BASE as u32 + active_offset) }
}

#[unsafe(no_mangle)]
#[cfg_attr(target_os = "none", unsafe(link_section = ".HardFault.user"))]
unsafe extern "C" fn HardFault() {
    cortex_m::peripheral::SCB::sys_reset();
}

#[exception]
unsafe fn DefaultHandler(_: i16) -> ! {
    const SCB_ICSR: *const u32 = 0xE000_ED04 as *const u32;
    let irqn = unsafe { core::ptr::read_volatile(SCB_ICSR) } as u8 as i16 - 16;
    panic!("DefaultHandler #{:?}", irqn);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::asm::udf();
}
