use crate::lib::event::{AppEvent, AppTask, Channel, DisplayTask, HardwareEvent, HardwareTask, InterfaceEvent};

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
            },
            ch_b: ChannelState {
                enable: false,
                v_set: 3.300,
                i_set: 1.00,
            },
            interface_state: InterfaceState {
                screen: Screen::Boot,
                selected_channel: None,
            },
            hardware_state: HardwareState::PowerOn
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) -> Option<AppTask> {
        match event {
            AppEvent::Hardware(hw) => {
                self.handle_hardware_event(hw)
            }
            AppEvent::Interface(ui) => {
                self.handle_interface_event(ui)
            }
        }
    }

    fn handle_hardware_event(&mut self, event: HardwareEvent) -> Option<AppTask> {
        match (&self.hardware_state, event) {
            (HardwareState::PowerOn, _) => {
                self.hardware_state = HardwareState::WaitingForPowerDelivery;

                Some(AppTask{
                    hardware: Some(HardwareTask::EnablePowerDelivery),
                    display: Some(DisplayTask::SetupSplash)
                })
            }
            (HardwareState::WaitingForPowerDelivery, HardwareEvent::PowerDeliveryReady) => {
                self.hardware_state = HardwareState::WaitingForSense;
                
                Some(AppTask{
                    hardware: Some(HardwareTask::EnableSense),
                    display: Some(DisplayTask::ConfirmPowerDelivery)
                })
            }
            (HardwareState::WaitingForSense, HardwareEvent::SenseReady) => {
                self.hardware_state = HardwareState::WaitingForConverter;
                
                Some(AppTask{
                    hardware: Some(HardwareTask::EnableConverter),
                    display: Some(DisplayTask::ConfirmSense)
                })
            }
            (HardwareState::WaitingForConverter, HardwareEvent::ConverterReady) => {
                self.hardware_state = HardwareState::Standby;

                Some(AppTask{
                    hardware: Some(HardwareTask::EnableConverter),
                    display: Some(DisplayTask::ConfirmSense)
                })
            }
            _ => None
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
            _ => None
        }
    }
}
