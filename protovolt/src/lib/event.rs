use core::{array::IntoIter, iter::FilterMap};

use embassy_rp::pio::State;
use embassy_time::Duration;

use crate::app::SetSelect;

#[derive(Debug)]
pub enum HardwareEvent {
    PowerOn,

    PowerDeliveryReady(PowerType),
    SenseReady(Result<(), ()>),
    ConverterReady(Result<(), ()>),

    StartMainInterface,

    ReadoutAcquired(Channel, Readout),
}

#[derive(Clone, Copy, Debug)]
pub struct Readout {
    pub voltage: f32,
    pub current: f32,
    pub power: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub voltage: f32,
    pub current: f32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            voltage: 5.00,
            current: 0.50,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PowerType {
    PowerDelivery(Limits),
    Standard(Limits),
}

impl Default for PowerType {
    fn default() -> Self {
        PowerType::Standard(Default::default())
    }
}

pub enum Change {
    Pressed,
    Released,
}

pub enum InterfaceEvent {
    ButtonUp,
    ButtonDown,
    ButtonLeft,
    ButtonRight,
    ButtonEnter(Change),
    ButtonSwitch(Change),
    ButtonSettings(Change),
    ButtonChannel(Channel),
}

pub enum AppEvent {
    Hardware(HardwareEvent),
    Interface(InterfaceEvent),
}

pub enum HardwareTask {
    // Initialization sequence + self-checks
    EnablePowerDelivery,
    EnableSense,
    EnableConverter,

    // Idle
    EnableReadoutLoop,

    DelayedInterfaceEvent(Duration, InterfaceEvent),
    DelayedHardwareEvent(Duration, HardwareEvent),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Channel {
    A,
    B,
}

impl Channel {
    pub fn get_other(self) -> Self {
        match self {
            Channel::A => Channel::B,
            Channel::B => Channel::A
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum SetState {
    #[default]
    Set,
    Limits,
}

#[derive(Clone, Copy)]
pub enum ChannelFocus {
    SelectedActive,
    UnselectedActive,
    SelectedInactive,
    UnselectedInactive,
}

pub enum FunctionButton {
    Enter,
    Switch,
    Settings,
}

pub enum DisplayTask {
    // Splash Screen
    SetupSplash,

    ConfirmPowerDelivery(PowerType),
    ConfirmSense(Result<(), ()>),
    ConfirmConverter(Result<(), ()>),

    // Main Readout
    SetupMain(PowerType, Limits, Limits),

    // Updates
    UpdateReadout(Channel, Readout),
    UpdateSetpoint(Channel, Limits, Option<SetSelect>),
    UpdateChannelFocus(ChannelFocus, ChannelFocus),
    UpdateSetState(Channel, SetState, Option<SetSelect>),

    // Navbar
    UpdateButton(Option<FunctionButton>),

    // Settings
    SetupSettings,
}

// pub struct AppTask {
//     pub hardware: Option<HardwareTask>,
//     pub display: Option<DisplayTask>,
// }

pub enum Task {
    Hardware(HardwareTask),
    Display(DisplayTask),
}

const APP_TASK_SIZE_LIMIT: usize = 5;

pub struct AppTask {
    pub tasks: [Option<Task>; APP_TASK_SIZE_LIMIT],
    pub count: usize,
}

impl IntoIterator for AppTask {
    type Item = Task;
    type IntoIter = FilterMap<
        IntoIter<Option<Task>, APP_TASK_SIZE_LIMIT>,
        fn(Option<Task>) -> Option<Task>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.tasks.into_iter().filter_map(|opt| opt)
    }
}

pub struct AppTaskBuilder {
    inner: AppTask,
}

impl AppTaskBuilder {
    pub fn new() -> Self {
        Self {
            inner: AppTask {
                tasks: [const { None }; APP_TASK_SIZE_LIMIT],
                count: 0,
            },
        }
    }

    pub fn hardware(mut self, task: HardwareTask) -> Self {
        if self.inner.count < self.inner.tasks.len() {
            self.inner.tasks[self.inner.count] = Some(Task::Hardware(task));
            self.inner.count += 1;
        }

        self
    }

    pub fn display(mut self, task: DisplayTask) -> Self {
        if self.inner.count < self.inner.tasks.len() {
            self.inner.tasks[self.inner.count] = Some(Task::Display(task));
            self.inner.count += 1;
        }

        self
    }

    pub fn build(self) -> Option<AppTask> {
        Some(self.inner)
    }

    pub fn hardware_task(task: HardwareTask) -> Option<AppTask> {
        let mut tasks = [const {None}; APP_TASK_SIZE_LIMIT];
        tasks[0] = Some(Task::Hardware(task));

        Some(AppTask { tasks: tasks, count: 1 })
    }

    pub fn display_task(task: DisplayTask) -> Option<AppTask> {
        let mut tasks = [const {None}; APP_TASK_SIZE_LIMIT];
        tasks[0] = Some(Task::Display(task));

        Some(AppTask { tasks: tasks, count: 1 })
    }
}
