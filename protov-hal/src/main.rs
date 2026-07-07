#![no_std]
#![no_main]

use protov_hal::{app, config, hal, scpi, task, ui};

use core::cell::RefCell;

use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_rp::adc::{self, Adc};
use embassy_rp::i2c::I2c;
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_rp::peripherals::{DMA_CH0, I2C0, I2C1, PIO0, USB};
use embassy_rp::pio;
use embassy_rp::pio::Pio;
use embassy_rp::spi::{self, Spi};
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, i2c};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex};

use embassy_sync::blocking_mutex::Mutex as I2cMutex;

use embassy_sync::channel::{Channel, Sender};
use embassy_time::{Duration, Ticker, Timer};

use hal::backlight::Backlight;
use hal::display::DisplayInterface;
use hal::event::{AppEvent, HardwareEvent, InterfaceEvent, Task};
use hal::firmware;
use hal::interface::{ButtonsInterface, matrix};
use hal::watchdog;

use app::App;
use task::{
    DfuOutcome, handle_dfu_task, handle_display_task, handle_hardware_task, is_dfu_hardware_task,
    run_followup_tasks,
};
use ui::Ui;

use hal::event::{
    AppTask, AppTaskBuilder, Channel as OutputChannel, DfuStatus, DisplayTask, HardwareTask,
    PowerType,
};
use hal::led::LedsInterface;
use hal::temperature::TemperatureReading;
use hal::{
    Hal, HalSense, HalTempSense, INA226_DUMP_REQ, INA226_DUMP_RESP, SENSE_CHANNEL, converter_a_irq,
    converter_b_irq, poll_sense, temp_sense,
};
use scpi::RegisterChannelExt;
use scpi::parser::ScpiCommand;
use scpi::state::ScpiState;
use scpi::usb::{build_usb_cdc, fwup_payload, fwup_payload_len, spawn_usb_tasks};
use scpi::{RESPONSE_BUF, SCPI_CMD, SCPI_RESP, ScpiContext, ScpiResponse};

use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

// Static channels
pub static INTERFACE_CHANNEL: Channel<ThreadModeRawMutex, InterfaceEvent, 32> = Channel::new();

const HW_CH_SIZE: usize = 32;
pub static HARDWARE_CHANNEL: Channel<ThreadModeRawMutex, HardwareEvent, HW_CH_SIZE> =
    Channel::new();
pub type HardwareChannelSender = hal::HardwareChannelSender;

// Multicore setup
static mut CORE1_STACK: Stack<8192> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

static BUTTONS_INTERFACE: StaticCell<ButtonsInterface> = StaticCell::new();

type StaticI2c0 = I2c<'static, I2C0, i2c::Blocking>;
type StaticI2c0Bus = Mutex<NoopRawMutex, RefCell<StaticI2c0>>;
static I2C0_BUS: StaticCell<StaticI2c0Bus> = StaticCell::new();

type StaticI2c1 = I2c<'static, I2C1, i2c::Blocking>;
type StaticI2c1Bus = Mutex<NoopRawMutex, RefCell<StaticI2c1>>;
static I2C1_BUS: StaticCell<StaticI2c1Bus> = StaticCell::new();

type StaticHalSense = HalSense<'static, NoopRawMutex, StaticI2c1>;
static HAL_SENSE: StaticCell<StaticHalSense> = StaticCell::new();

type StaticHalTempSense = HalTempSense<'static>;
static HAL_TEMP_SENSE: StaticCell<StaticHalTempSense> = StaticCell::new();

static SCPI_STATE: StaticCell<ScpiState> = StaticCell::new();

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
    ADC_IRQ_FIFO => adc::InterruptHandler;
    USBCTRL_IRQ => InterruptHandler<USB>;
});

const FWUP_APPL_RESET_DELAY_MS: u64 = 200;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Confirm swap / clear stale bootloader state before any other init (Embassy b.rs).
    let fw_ctx = firmware::init(p.FLASH);
    watchdog::init(p.WATCHDOG);
    watchdog::stop_bootloader();
    watchdog::cancel_after_boot();

    // USB must start before any lengthy blocking init — the host will enumerate
    // and SET_CONFIGURATION while we are still booting otherwise (error -110).
    let usb_driver = Driver::new(p.USB, Irqs);
    let usb_resources = build_usb_cdc(usb_driver);
    spawn_usb_tasks(&spawner, usb_resources);

    // Power hardware initialization
    let i2c0 = I2c::new_blocking(p.I2C0, p.PIN_1, p.PIN_0, i2c::Config::default());
    let i2c0_bus: Mutex<NoopRawMutex, _> = I2cMutex::new(RefCell::new(i2c0));
    let i2c0_bus = I2C0_BUS.init(i2c0_bus);

    let i2c1 = I2c::new_blocking(p.I2C1, p.PIN_23, p.PIN_22, i2c::Config::default());
    let i2c1_bus: Mutex<NoopRawMutex, _> = I2cMutex::new(RefCell::new(i2c1));
    let i2c1_bus = I2C1_BUS.init(i2c1_bus);

    // Power hardware interrupts initialization
    spawner.spawn(unwrap!(converter_a_irq(
        HARDWARE_CHANNEL.sender(),
        p.PIN_15
    )));
    spawner.spawn(unwrap!(converter_b_irq(
        HARDWARE_CHANNEL.sender(),
        p.PIN_14
    )));

    // Output measurement loop
    let hal_sense = HalSense::new(i2c1_bus);
    let hal_sense = HAL_SENSE.init(hal_sense);
    spawner.spawn(unwrap!(poll_sense(
        hal_sense,
        SENSE_CHANNEL.receiver(),
        HARDWARE_CHANNEL.sender()
    )));

    // Temperature sense loop
    let adc: Adc<'_, adc::Async> = Adc::new(p.ADC, Irqs, adc::Config::default());
    let hal_temp_sense = HalTempSense::new(adc, p.PIN_26, p.PIN_27, p.ADC_TEMP_SENSOR);
    let hal_temp_sense = HAL_TEMP_SENSE.init(hal_temp_sense);
    spawner.spawn(unwrap!(temp_sense(
        hal_temp_sense,
        HARDWARE_CHANNEL.sender()
    )));

    let mut hal = Hal::new(i2c0_bus, i2c0_bus, p.PIN_24, p.PIN_25);

    let scpi_state = SCPI_STATE.init(ScpiState::default());
    let mut last_temp = TemperatureReading {
        ch_a: config::BOOT_TEMP_CH_A,
        ch_b: config::BOOT_TEMP_CH_B,
        mcu: config::BOOT_TEMP_MCU,
    };
    let mut input_voltage = 20.0f32;
    let mut input_current = 0.35f32;
    let mut input_type_pd = true;
    let mut power_type = PowerType::default();
    let mut last_serial_connected = scpi::serial_connected();

    // Buttons (moved to static so Core 1 owns them)
    let buttons = ButtonsInterface::new(
        [p.PIN_8.into(), p.PIN_9.into(), p.PIN_10.into()],
        [p.PIN_5.into(), p.PIN_6.into(), p.PIN_7.into()],
    );
    let buttons = BUTTONS_INTERFACE.init(buttons);

    // SPI display setup
    let spi = Spi::new_blocking(p.SPI0, p.PIN_18, p.PIN_19, p.PIN_20, spi::Config::default());
    let spi_shared: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));
    let mut display = DisplayInterface::new(&spi_shared, p.PIN_17, p.PIN_21, p.PIN_28);
    let backlight = Backlight::new(p.PWM_SLICE0, p.PIN_16);

    // Interfacing LEDs setup
    let pio = Pio::new(p.PIO0, Irqs);
    let leds = LedsInterface::new(pio, p.DMA_CH0, Irqs, p.PIN_11);

    // App logic
    let mut app = App::default();
    let mut ui = Ui::new(&mut display.target, leds, backlight);
    ui.clear().unwrap();
    ui.set_lcd_brightness(scpi_state.lcd_brightness);

    // Start core 1 and spawn poll_interface there
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                spawner.spawn(unwrap!(poll_interface(buttons, INTERFACE_CHANNEL.sender())));
            });
        },
    );

    // Start application
    let hw_sender = HARDWARE_CHANNEL.sender();
    hw_sender.send(HardwareEvent::PowerOn).await;
    let int_sender = INTERFACE_CHANNEL.sender();

    let mut ticker = Ticker::every(Duration::from_hz(100));
    let mut poll_counter = 0u32;

    loop {
        if scpi_state.lcd_brightness != ui.lcd_brightness() {
            ui.set_lcd_brightness(scpi_state.lcd_brightness);
        }

        if let Ok(cmd) = SCPI_CMD.try_receive() {
            let ctx = build_scpi_context(
                &app,
                &last_temp,
                input_type_pd,
                input_voltage,
                input_current,
                scpi_state,
                firmware::flash_unique_id(),
            );
            let (mut response, tasks) = match cmd {
                ScpiCommand::Ina226RegQuery { channel } => {
                    INA226_DUMP_REQ.send(channel.to_hal()).await;
                    let response = match INA226_DUMP_RESP.receive().await {
                        Ok(text) => ScpiResponse::with_text(text),
                        Err(()) => {
                            scpi_state.push_error(-200, "Register dump unavailable");
                            ScpiResponse::none()
                        }
                    };
                    (response, None)
                }
                ScpiCommand::Tps55289RegQuery { channel } => {
                    let mut buf = heapless::String::<RESPONSE_BUF>::new();
                    let response = if hal.dump_tps55289(channel.to_hal(), &mut buf).is_ok() {
                        ScpiResponse::with_text(buf)
                    } else {
                        scpi_state.push_error(-200, "Register dump unavailable");
                        ScpiResponse::none()
                    };
                    (response, None)
                }
                ScpiCommand::FwupData => {
                    let result = app.handle_fwup_data(fwup_payload_len() as u32, scpi_state);
                    (result.response, result.tasks)
                }
                ScpiCommand::FwupAbor => {
                    let result = app.handle_scpi(cmd, scpi_state, &ctx);
                    fw_ctx.dfu_abort();
                    (result.response, result.tasks)
                }
                _ => {
                    let result = app.handle_scpi(cmd, scpi_state, &ctx);
                    (result.response, result.tasks)
                }
            };
            let mut fwup_appl_outcome = None;
            if let Some(tasks) = tasks {
                for slot in tasks.tasks[..tasks.count].iter().filter_map(|t| t.as_ref()) {
                    if let Task::Display(disp_task) = slot {
                        handle_display_task(
                            *disp_task,
                            &mut ui,
                            scpi_state,
                            &hw_sender,
                            &int_sender,
                        )
                        .await;
                    }
                }

                for task in tasks {
                    match task {
                        Task::Display(_) => {}
                        Task::Hardware(hw_task) if is_dfu_hardware_task(&hw_task) => {
                            match handle_dfu_task(
                                hw_task,
                                fw_ctx,
                                &mut app,
                                scpi_state,
                                fwup_payload(),
                            ) {
                                DfuOutcome::VerifySucceeded(extra) => {
                                    fwup_appl_outcome = Some(true);
                                    if let Some(extra) = extra {
                                        run_followup_tasks(
                                            extra,
                                            &mut ui,
                                            scpi_state,
                                            &hw_sender,
                                            &int_sender,
                                        )
                                        .await;
                                    }
                                }
                                DfuOutcome::VerifyFailed(extra) => {
                                    fwup_appl_outcome = Some(false);
                                    fw_ctx.dfu_abort();
                                    if let Some(extra) = extra {
                                        run_followup_tasks(
                                            extra,
                                            &mut ui,
                                            scpi_state,
                                            &hw_sender,
                                            &int_sender,
                                        )
                                        .await;
                                    }
                                }
                                DfuOutcome::Progress(extra) => {
                                    if let Some(extra) = extra {
                                        run_followup_tasks(
                                            extra,
                                            &mut ui,
                                            scpi_state,
                                            &hw_sender,
                                            &int_sender,
                                        )
                                        .await;
                                    }
                                }
                            }
                        }
                        Task::Hardware(hw_task) => {
                            handle_hardware_task(hw_task, &mut hal, &hw_sender, &int_sender).await;
                        }
                    }
                }
            }
            if let Some(success) = fwup_appl_outcome {
                response = app.fwup_appl_response(success, scpi_state);
            }
            SCPI_RESP.send(response).await;
            if fwup_appl_outcome == Some(true) {
                handle_display_task(
                    DisplayTask::DfuStatus(DfuStatus::Flashing),
                    &mut ui,
                    scpi_state,
                    &hw_sender,
                    &int_sender,
                )
                .await;
                Timer::after(Duration::from_millis(FWUP_APPL_RESET_DELAY_MS)).await;
                firmware::dfu_reset_after_verify();
            }
        }

        let mut tasks = AppTaskBuilder::new();
        let mut pending_readout_a = None;
        let mut pending_readout_b = None;

        while let Ok(hw_event) = HARDWARE_CHANNEL.try_receive() {
            match hw_event {
                HardwareEvent::TempAcquired(temp) => {
                    last_temp = temp;
                    tasks =
                        append_app_event(tasks, &mut app, scpi_state, AppEvent::Hardware(hw_event));
                }
                HardwareEvent::PowerDeliveryReady(pt) => {
                    power_type = pt;
                    match pt {
                        hal::event::PowerType::PowerDelivery(limits) => {
                            input_type_pd = true;
                            input_voltage = limits.voltage;
                            input_current = limits.current;
                        }
                        hal::event::PowerType::Standard(limits) => {
                            input_type_pd = false;
                            input_voltage = limits.voltage;
                            input_current = limits.current;
                        }
                    }
                    tasks = append_app_event(
                        tasks,
                        &mut app,
                        scpi_state,
                        AppEvent::Hardware(HardwareEvent::PowerDeliveryReady(pt)),
                    );
                }
                HardwareEvent::ReadoutAcquired(channel, readout) => match channel {
                    OutputChannel::A => pending_readout_a = Some(readout),
                    OutputChannel::B => pending_readout_b = Some(readout),
                },
                other => {
                    tasks =
                        append_app_event(tasks, &mut app, scpi_state, AppEvent::Hardware(other));
                }
            }
        }

        if let Some(readout) = pending_readout_a {
            tasks = append_app_event(
                tasks,
                &mut app,
                scpi_state,
                AppEvent::Hardware(HardwareEvent::ReadoutAcquired(OutputChannel::A, readout)),
            );
        }
        if let Some(readout) = pending_readout_b {
            tasks = append_app_event(
                tasks,
                &mut app,
                scpi_state,
                AppEvent::Hardware(HardwareEvent::ReadoutAcquired(OutputChannel::B, readout)),
            );
        }

        let mut last_ui_event = None;
        while let Ok(ui_event) = INTERFACE_CHANNEL.try_receive() {
            last_ui_event = Some(ui_event);
        }
        if let Some(ui_event) = last_ui_event {
            tasks = append_app_event(tasks, &mut app, scpi_state, AppEvent::Interface(ui_event));
        }

        poll_counter = poll_counter.wrapping_add(1);
        if app.is_standby() && poll_counter >= 20 {
            poll_counter = 0;
            tasks = tasks
                .hardware(HardwareTask::PollConverterStatus(OutputChannel::A))
                .hardware(HardwareTask::PollConverterStatus(OutputChannel::B));
        }

        let serial_connected = scpi::serial_connected();
        if serial_connected != last_serial_connected && app.is_standby() {
            last_serial_connected = serial_connected;
            handle_display_task(
                DisplayTask::UpdatePowerInfo(power_type),
                &mut ui,
                scpi_state,
                &hw_sender,
                &int_sender,
            )
            .await;
        }

        if let Some(app_task) = tasks.build() {
            for task in app_task {
                match task {
                    Task::Hardware(hw_task) => {
                        handle_hardware_task(hw_task, &mut hal, &hw_sender, &int_sender).await;
                    }
                    Task::Display(disp_task) => {
                        handle_display_task(disp_task, &mut ui, scpi_state, &hw_sender, &int_sender)
                            .await
                    }
                }
            }
        }

        ticker.next().await;
    }
}

fn append_app_event(
    mut tasks: AppTaskBuilder,
    app: &mut App,
    scpi: &mut ScpiState,
    event: AppEvent,
) -> AppTaskBuilder {
    if let Some(produced) = app.handle_event(event, scpi) {
        tasks = tasks.extend(app_task_into_builder(produced));
    }
    tasks
}

fn app_task_into_builder(task: AppTask) -> AppTaskBuilder {
    let mut builder = AppTaskBuilder::new();
    for item in task {
        builder = match item {
            Task::Hardware(hw) => builder.hardware(hw),
            Task::Display(disp) => builder.display(disp),
        };
    }
    builder
}

fn build_scpi_context(
    _app: &App,
    temp: &TemperatureReading,
    input_type_pd: bool,
    input_voltage: f32,
    input_current: f32,
    scpi_state: &ScpiState,
    flash_unique_id: &[u8; 8],
) -> ScpiContext {
    let prot_a = scpi_state.prot_latched(scpi::ScpiChannel::Ch1);
    let prot_b = scpi_state.prot_latched(scpi::ScpiChannel::Ch2);
    let sense_ok = !prot_a && !prot_b;
    ScpiContext {
        temp_ch_a: temp.ch_a,
        temp_ch_b: temp.ch_b,
        temp_mcu: temp.mcu,
        input_type_pd,
        input_voltage,
        input_current,
        sense_ok,
        converter_ok: sense_ok,
        prot_latched_a: prot_a,
        prot_latched_b: prot_b,
        flash_unique_id: *flash_unique_id,
    }
}

#[embassy_executor::task]
async fn poll_interface(
    buttons: &'static mut ButtonsInterface<'static>,
    channel: Sender<'static, ThreadModeRawMutex, InterfaceEvent, 32>,
) {
    let mut ticker = Ticker::every(Duration::from_millis(matrix::POLL_TIME_MS));
    loop {
        if let Some(event) = buttons.poll() {
            let _ = channel.try_send(event);
        }
        ticker.next().await;
    }
}
