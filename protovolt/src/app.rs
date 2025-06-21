use cortex_m::register::control::read;
use embassy_time::Duration;

use defmt::*;

use crate::lib::event::{
    AppEvent, AppTask, Channel, DisplayTask, HardwareEvent, HardwareTask, InterfaceEvent, Readout,
};

pub struct App {
    pub ch_a: ChannelState,
    pub ch_b: ChannelState,

    pub interface_state: InterfaceState,
    pub hardware_state: HardwareState,
}

struct ChannelState {
    pub enable: bool,
    pub v_set: f32,
    pub i_set: f32,

    pub readout: Option<Readout>,
}

struct InterfaceState {
    pub screen: Screen,
    pub selected_channel: Option<Channel>,
}

enum HardwareState {
    PowerOn,

    WaitingForPowerDelivery,
    WaitingForSense,
    WaitingForConverter,

    WaitingMainUi,
    Standby,

    Error,
}

enum Screen {
    Boot,
    Main,
    Settings,
}

impl App {
    pub fn new() -> Self {
        Self {
            ch_a: ChannelState {
                enable: false,
                v_set: 5.000,
                i_set: 1.00,
                readout: None,
            },
            ch_b: ChannelState {
                enable: false,
                v_set: 3.300,
                i_set: 1.00,
                readout: None,
            },
            interface_state: InterfaceState {
                screen: Screen::Boot,
                selected_channel: None,
            },
            hardware_state: HardwareState::PowerOn,
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) -> Option<AppTask> {
        match event {
            AppEvent::Hardware(hw) => self.handle_hardware_event(hw),
            AppEvent::Interface(ui) => self.handle_interface_event(ui),
        }
    }

    fn handle_hardware_event(&mut self, event: HardwareEvent) -> Option<AppTask> {
        match (&self.hardware_state, event) {
            (HardwareState::PowerOn, HardwareEvent::PowerOn) => {
                self.hardware_state = HardwareState::WaitingForPowerDelivery;

                Some(AppTask {
                    hardware: Some(HardwareTask::EnablePowerDelivery),
                    display: Some(DisplayTask::SetupSplash),
                })
            }
            (HardwareState::WaitingForPowerDelivery, HardwareEvent::PowerDeliveryReady) => {
                self.hardware_state = HardwareState::WaitingForSense;

                Some(AppTask {
                    hardware: Some(HardwareTask::EnableSense),
                    display: Some(DisplayTask::ConfirmPowerDelivery),
                })
            }
            (HardwareState::WaitingForSense, HardwareEvent::SenseReady) => {
                self.hardware_state = HardwareState::WaitingForConverter;

                Some(AppTask {
                    hardware: Some(HardwareTask::EnableConverter),
                    display: Some(DisplayTask::ConfirmSense),
                })
            }
            (HardwareState::WaitingForConverter, HardwareEvent::ConverterReady) => {
                self.hardware_state = HardwareState::WaitingMainUi;

                Some(AppTask {
                    hardware: Some(HardwareTask::DelayedHardwareEvent(
                        Duration::from_millis(500),
                        HardwareEvent::StartMainInterface,
                    )),
                    display: Some(DisplayTask::ConfirmConverter),
                })
            }
            (HardwareState::WaitingMainUi, HardwareEvent::StartMainInterface) => {
                self.hardware_state = HardwareState::Standby;
                self.interface_state.screen = Screen::Main;
                
                info!("call readout loop");

                Some(AppTask {
                    hardware: Some(HardwareTask::EnableReadoutLoop),
                    display: Some(DisplayTask::SetupMain),
                })
            }
            (HardwareState::Standby, HardwareEvent::ReadoutAcquired(channel, readout)) => {
                let current_readout = match channel {
                    Channel::A => &mut self.ch_a.readout,
                    Channel::B => &mut self.ch_b.readout,
                };
                *current_readout = Some(readout);

                Some(AppTask {
                    hardware: None,
                    display: Some(DisplayTask::UpdateReadout(channel, readout))
                })
            }
            _ => None,
        }
    }

    fn handle_interface_event(&mut self, event: InterfaceEvent) -> Option<AppTask> {
        match event {
            InterfaceEvent::ButtonUp => {
                // Some(AppTask{
                //     hardware:
                // })

                None
            }
            _ => None,
        }
    }
}
