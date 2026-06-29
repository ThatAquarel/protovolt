use crate::config::{CH1_FACTORY, CH2_FACTORY};
use crate::model::{
    AppEvent, AppTask, Change, Channel, ChannelFocus, ChannelHardwareState, ConfirmState,
    ConverterFlags, DisplayTask, FunctionButton, HardwareEvent, HardwareTask, InterfaceEvent,
    PowerType, Readout, SetSelect, SetState, Task, TemperatureReading,
};
use crate::scpi::ScpiChannel;
use crate::scpi::state::ScpiState;

use super::AppCore;

fn standby_app() -> (AppCore, ScpiState) {
    let mut app = AppCore::default();
    app.force_standby();
    (app, ScpiState::default())
}

fn boot_to_standby(app: &mut AppCore, scpi: &mut ScpiState) {
    let _ = app.handle_event(AppEvent::Hardware(HardwareEvent::PowerOn), scpi);
    let _ = app.handle_event(
        AppEvent::Hardware(HardwareEvent::PowerDeliveryReady(PowerType::default())),
        scpi,
    );
    let _ = app.handle_event(AppEvent::Hardware(HardwareEvent::SenseReady(Ok(()))), scpi);
    let _ = app.handle_event(
        AppEvent::Hardware(HardwareEvent::ConverterReady(Ok(()))),
        scpi,
    );
    let _ = app.handle_event(AppEvent::Hardware(HardwareEvent::StartMainInterface), scpi);
    assert!(app.is_standby());
}

fn press_channel(app: &mut AppCore, scpi: &mut ScpiState, ch: Channel) -> Option<AppTask> {
    app.handle_event(AppEvent::Interface(InterfaceEvent::ButtonChannel(ch)), scpi)
}

fn press_interface(
    app: &mut AppCore,
    scpi: &mut ScpiState,
    event: InterfaceEvent,
) -> Option<AppTask> {
    app.handle_event(AppEvent::Interface(event), scpi)
}

fn iter_tasks(task: &AppTask) -> impl Iterator<Item = &Task> {
    task.tasks.iter().filter_map(|t| t.as_ref())
}

fn has_converter_state_task(task: &AppTask, ch: Channel, enabled: bool) -> bool {
    iter_tasks(task).any(|t| {
        matches!(
            t,
            Task::Hardware(HardwareTask::UpdateConverterState(c, on))
                if *c == ch && *on == enabled
        )
    })
}

fn channel_focus(task: &AppTask) -> Option<(ChannelFocus, ChannelFocus)> {
    iter_tasks(task).find_map(|t| match t {
        Task::Display(DisplayTask::UpdateChannelFocus(fa, fb, _, _)) => Some((*fa, *fb)),
        _ => None,
    })
}

fn button_update(task: &AppTask) -> Option<(ConfirmState, Option<FunctionButton>)> {
    iter_tasks(task).find_map(|t| match t {
        Task::Display(DisplayTask::UpdateButton(confirm, button)) => match button {
            Some(FunctionButton::Enter) => Some((*confirm, Some(FunctionButton::Enter))),
            Some(FunctionButton::Switch) => Some((*confirm, Some(FunctionButton::Switch))),
            Some(FunctionButton::Settings) => Some((*confirm, Some(FunctionButton::Settings))),
            None => Some((*confirm, None)),
        },
        _ => None,
    })
}

fn settings_update(task: &AppTask) -> Option<bool> {
    iter_tasks(task).find_map(|t| match t {
        Task::Display(DisplayTask::UpdateSettings(visible)) => Some(*visible),
        _ => None,
    })
}

fn has_readout_update(task: &AppTask) -> bool {
    iter_tasks(task).any(|t| matches!(t, Task::Display(DisplayTask::UpdateReadout(_, _))))
}

fn has_channel_units_update(task: &AppTask, channel: Channel) -> bool {
    iter_tasks(task).any(|t| {
        matches!(
            t,
            Task::Display(DisplayTask::UpdateChannelUnits(ch)) if *ch == channel
        )
    })
}

fn open_settings(app: &mut AppCore, scpi: &mut ScpiState) -> AppTask {
    press_interface(app, scpi, InterfaceEvent::ButtonSettings(Change::Pressed))
        .expect("settings open")
}

fn sample_readout() -> Readout {
    Readout {
        voltage: 5.0,
        current: 1.0,
        power: 5.0,
    }
}

fn acquire_readout(app: &mut AppCore, scpi: &mut ScpiState, channel: Channel) -> Option<AppTask> {
    app.handle_event(
        AppEvent::Hardware(HardwareEvent::ReadoutAcquired(channel, sample_readout())),
        scpi,
    )
}

fn focus_eq(a: ChannelFocus, b: ChannelFocus) -> bool {
    matches!(
        (a, b),
        (ChannelFocus::SelectedActive, ChannelFocus::SelectedActive)
            | (
                ChannelFocus::SelectedInactive,
                ChannelFocus::SelectedInactive
            )
            | (
                ChannelFocus::UnselectedActive,
                ChannelFocus::UnselectedActive
            )
            | (
                ChannelFocus::UnselectedInactive,
                ChannelFocus::UnselectedInactive
            )
    )
}

fn confirm_eq(a: ConfirmState, b: ConfirmState) -> bool {
    match (a, b) {
        (ConfirmState::AwaitModify, ConfirmState::AwaitModify) => true,
        (ConfirmState::AwaitConfirmModify(ch_a), ConfirmState::AwaitConfirmModify(ch_b)) => {
            ch_a == ch_b
        }
        _ => false,
    }
}

fn function_eq(a: Option<FunctionButton>, b: Option<FunctionButton>) -> bool {
    matches!(
        (a, b),
        (None, None)
            | (Some(FunctionButton::Enter), Some(FunctionButton::Enter))
            | (Some(FunctionButton::Switch), Some(FunctionButton::Switch))
            | (
                Some(FunctionButton::Settings),
                Some(FunctionButton::Settings)
            )
    )
}

fn assert_both_channels_off(app: &AppCore) {
    assert!(!app.channel_enable(Channel::A));
    assert!(!app.channel_enable(Channel::B));
    assert_eq!(app.hw_state(Channel::A), ChannelHardwareState::Off);
    assert_eq!(app.hw_state(Channel::B), ChannelHardwareState::Off);
}

fn select_and_activate(app: &mut AppCore, scpi: &mut ScpiState, ch: Channel) {
    press_channel(app, scpi, ch);
    press_channel(app, scpi, ch);
    assert!(app.channel_enable(ch));
}

// --- Startup and channel select/activate ---

#[test]
fn standby_startup_no_channel_selected_or_enabled() {
    let (app, _scpi) = standby_app();
    assert_both_channels_off(&app);
    assert_eq!(app.selected_channel(), None);
    assert!(!app.is_setpoint_edit());
    assert!(matches!(app.set_state(), SetState::Set));
}

#[test]
fn boot_sequence_leaves_channels_off() {
    let mut app = AppCore::default();
    let mut scpi = ScpiState::default();
    assert_both_channels_off(&app);

    boot_to_standby(&mut app, &mut scpi);

    assert_both_channels_off(&app);
    assert_eq!(app.selected_channel(), None);
}

#[test]
fn boot_then_first_channel_press_selects_without_enabling() {
    let mut app = AppCore::default();
    let mut scpi = ScpiState::default();
    boot_to_standby(&mut app, &mut scpi);

    let task = press_channel(&mut app, &mut scpi, Channel::A).expect("first channel press");

    assert_eq!(app.selected_channel(), Some(Channel::A));
    assert_both_channels_off(&app);
    assert!(!has_converter_state_task(&task, Channel::A, true));
    let (fa, fb) = channel_focus(&task).expect("focus update");
    assert!(focus_eq(fa, ChannelFocus::SelectedInactive));
    assert!(focus_eq(fb, ChannelFocus::UnselectedInactive));
}

#[test]
fn channel_button_first_press_selects_without_enabling() {
    let (mut app, mut scpi) = standby_app();

    for ch in [Channel::A, Channel::B] {
        let task = press_channel(&mut app, &mut scpi, ch).expect("channel press task");
        assert_eq!(app.selected_channel(), Some(ch));
        assert!(!app.channel_enable(ch));
        assert_eq!(app.hw_state(ch), ChannelHardwareState::Off);
        assert!(!has_converter_state_task(&task, ch, true));

        let (focus_ch, focus_other) = match ch {
            Channel::A => (
                ChannelFocus::SelectedInactive,
                ChannelFocus::UnselectedInactive,
            ),
            Channel::B => (
                ChannelFocus::UnselectedInactive,
                ChannelFocus::SelectedInactive,
            ),
        };
        let (fa, fb) = channel_focus(&task).expect("focus update");
        assert!(focus_eq(fa, focus_ch));
        assert!(focus_eq(fb, focus_other));
    }
}

#[test]
fn channel_button_second_press_activates() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    let task = press_channel(&mut app, &mut scpi, Channel::A).expect("activate task");

    assert_eq!(app.selected_channel(), Some(Channel::A));
    assert!(app.channel_enable(Channel::A));
    assert_eq!(
        app.hw_state(Channel::A),
        ChannelHardwareState::ConstantVoltage
    );
    assert!(has_converter_state_task(&task, Channel::A, true));
}

#[test]
fn channel_button_third_press_deactivates() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    press_channel(&mut app, &mut scpi, Channel::A);
    let task = press_channel(&mut app, &mut scpi, Channel::A).expect("deactivate task");

    assert_eq!(app.selected_channel(), Some(Channel::A));
    assert!(!app.channel_enable(Channel::A));
    assert_eq!(app.hw_state(Channel::A), ChannelHardwareState::Off);
    assert!(has_converter_state_task(&task, Channel::A, false));
}

#[test]
fn switching_channel_selection_does_not_enable() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    let task = press_channel(&mut app, &mut scpi, Channel::B).expect("switch selection");

    assert_eq!(app.selected_channel(), Some(Channel::B));
    assert_both_channels_off(&app);
    assert!(!has_converter_state_task(&task, Channel::B, true));

    let (fa, fb) = channel_focus(&task).expect("focus update");
    assert!(focus_eq(fa, ChannelFocus::UnselectedInactive));
    assert!(focus_eq(fb, ChannelFocus::SelectedInactive));
}

#[test]
fn active_channel_stays_on_when_selecting_other() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    press_channel(&mut app, &mut scpi, Channel::A);
    let task = press_channel(&mut app, &mut scpi, Channel::B).expect("select B while A active");

    assert_eq!(app.selected_channel(), Some(Channel::B));
    assert!(app.channel_enable(Channel::A));
    assert!(!app.channel_enable(Channel::B));
    assert!(!has_converter_state_task(&task, Channel::B, true));

    let (fa, fb) = channel_focus(&task).expect("focus update");
    assert!(focus_eq(fa, ChannelFocus::UnselectedActive));
    assert!(focus_eq(fb, ChannelFocus::SelectedInactive));
}

#[test]
fn both_channels_can_be_active_independently() {
    let (mut app, mut scpi) = standby_app();

    select_and_activate(&mut app, &mut scpi, Channel::A);
    press_channel(&mut app, &mut scpi, Channel::B);
    let task = press_channel(&mut app, &mut scpi, Channel::B).expect("activate B");

    assert!(app.channel_enable(Channel::A));
    assert!(app.channel_enable(Channel::B));
    assert_eq!(app.selected_channel(), Some(Channel::B));
    assert!(has_converter_state_task(&task, Channel::B, true));

    let (fa, fb) = channel_focus(&task).expect("focus update");
    assert!(focus_eq(fa, ChannelFocus::UnselectedActive));
    assert!(focus_eq(fb, ChannelFocus::SelectedActive));
}

// --- Arrow navigation and set select ---

#[test]
fn left_right_navigate_between_selected_channels() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    assert_eq!(app.selected_channel(), Some(Channel::A));

    let task =
        press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonRight).expect("navigate to B");
    assert_eq!(app.selected_channel(), Some(Channel::B));
    let (fa, fb) = channel_focus(&task).expect("focus update");
    assert!(focus_eq(fa, ChannelFocus::UnselectedInactive));
    assert!(focus_eq(fb, ChannelFocus::SelectedInactive));

    let task =
        press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonLeft).expect("navigate to A");
    assert_eq!(app.selected_channel(), Some(Channel::A));
    let (fa, fb) = channel_focus(&task).expect("focus update");
    assert!(focus_eq(fa, ChannelFocus::SelectedInactive));
    assert!(focus_eq(fb, ChannelFocus::UnselectedInactive));
}

#[test]
fn arrow_navigation_no_op_without_selection() {
    let (mut app, mut scpi) = standby_app();

    assert!(press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonLeft).is_none());
    assert!(press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonRight).is_none());
    assert_eq!(app.selected_channel(), None);
}

#[test]
fn navigation_copies_set_select_to_newly_selected_channel() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonDown);
    assert!(matches!(app.set_select(Channel::A), SetSelect::Current));

    press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonRight);
    assert_eq!(app.selected_channel(), Some(Channel::B));
    assert!(matches!(app.set_select(Channel::B), SetSelect::Current));
}

#[test]
fn up_down_toggle_voltage_current_select() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    assert!(matches!(app.set_select(Channel::A), SetSelect::Voltage));

    press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonDown);
    assert!(matches!(app.set_select(Channel::A), SetSelect::Current));

    press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonUp);
    assert!(matches!(app.set_select(Channel::A), SetSelect::Voltage));
}

// --- Set / limits switch ---

#[test]
fn switch_button_toggles_set_limits_mode() {
    let (mut app, mut scpi) = standby_app();

    assert!(matches!(app.set_state(), SetState::Set));

    press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSwitch(Change::Pressed),
    );
    assert!(matches!(app.set_state(), SetState::Limits));

    press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSwitch(Change::Pressed),
    );
    assert!(matches!(app.set_state(), SetState::Set));
}

#[test]
fn switch_during_setpoint_edit_preserves_edit_mode() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonEnter(Change::Pressed),
    );
    assert!(app.is_setpoint_edit());

    press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSwitch(Change::Pressed),
    );
    assert!(app.is_setpoint_edit());
    assert!(matches!(app.set_state(), SetState::Limits));
    assert!(confirm_eq(
        button_update(
            &press_interface(
                &mut app,
                &mut scpi,
                InterfaceEvent::ButtonSwitch(Change::Released),
            )
            .expect("switch release"),
        )
        .expect("button update")
        .0,
        ConfirmState::AwaitConfirmModify(Some(Channel::A)),
    ));
}

// --- Enter / setpoint edit ---

#[test]
fn enter_without_selection_does_not_enter_setpoint_edit() {
    let (mut app, mut scpi) = standby_app();

    press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonEnter(Change::Pressed),
    );
    assert_eq!(app.selected_channel(), None);
    assert!(!app.is_setpoint_edit());
}

#[test]
fn enter_toggles_setpoint_edit_for_selected_channel() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::B);
    press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonEnter(Change::Pressed),
    );
    assert!(app.is_setpoint_edit());

    let task = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonEnter(Change::Pressed),
    )
    .expect("confirm edit");
    assert!(!app.is_setpoint_edit());
    assert!(iter_tasks(&task).any(|t| matches!(
        t,
        Task::Hardware(HardwareTask::UpdateConverterVoltage(Channel::B, _))
            | Task::Hardware(HardwareTask::UpdateConverterCurrent(Channel::B, _))
    )));
}

#[test]
fn channel_select_resets_setpoint_edit_mode() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonEnter(Change::Pressed),
    );
    assert!(app.is_setpoint_edit());

    press_channel(&mut app, &mut scpi, Channel::B);
    assert!(!app.is_setpoint_edit());
    assert_eq!(app.selected_channel(), Some(Channel::B));
}

#[test]
fn setpoint_edit_while_active_changes_value_and_updates_converter() {
    let (mut app, mut scpi) = standby_app();
    let initial = app.target_voltage(Channel::A);

    select_and_activate(&mut app, &mut scpi, Channel::A);
    press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonEnter(Change::Pressed),
    );
    assert!(app.is_setpoint_edit());

    press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonDown);
    assert!(app.target_voltage(Channel::A) < initial);

    let task = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonEnter(Change::Pressed),
    )
    .expect("confirm setpoint");
    assert!(!app.is_setpoint_edit());
    assert!(app.channel_enable(Channel::A));
    assert!(iter_tasks(&task).any(|t| matches!(
        t,
        Task::Hardware(HardwareTask::UpdateConverterVoltage(
            Channel::A,
            v
        )) if (*v - app.target_voltage(Channel::A)).abs() < f32::EPSILON
    )));
}

// --- Button press / release highlight ---

#[test]
fn enter_release_restores_confirm_button_in_setpoint_edit() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonEnter(Change::Pressed),
    );

    let task = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonEnter(Change::Released),
    )
    .expect("enter release");
    let (confirm, button) = button_update(&task).expect("button update");
    assert!(confirm_eq(
        confirm,
        ConfirmState::AwaitConfirmModify(Some(Channel::A)),
    ));
    assert!(function_eq(button, Some(FunctionButton::Enter)));
}

#[test]
fn enter_release_clears_highlight_in_navigation_mode() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    let task = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonEnter(Change::Released),
    )
    .expect("enter release");
    let (confirm, button) = button_update(&task).expect("button update");
    assert!(confirm_eq(confirm, ConfirmState::AwaitModify));
    assert!(function_eq(button, None));
}

#[test]
fn settings_button_toggles_latched_menu() {
    let (mut app, mut scpi) = standby_app();

    let opened = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSettings(Change::Pressed),
    )
    .expect("settings press open");
    assert!(app.settings_open());
    let (_, button) = button_update(&opened).expect("button update");
    assert!(function_eq(button, Some(FunctionButton::Settings)));
    assert!(settings_update(&opened) == Some(true));

    let released = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSettings(Change::Released),
    );
    assert!(released.is_none());
    assert!(app.settings_open());
    assert!(function_eq(
        Some(FunctionButton::Settings),
        Some(FunctionButton::Settings)
    ));

    let closed = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSettings(Change::Pressed),
    )
    .expect("settings press close");
    assert!(!app.settings_open());
    let (_, button) = button_update(&closed).expect("button update");
    assert!(function_eq(button, None));
    assert!(settings_update(&closed) == Some(false));
}

#[test]
fn readout_display_suppressed_while_settings_open() {
    let (mut app, mut scpi) = standby_app();

    let _ = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSettings(Change::Pressed),
    )
    .expect("settings open");
    assert!(app.settings_open());

    let task = acquire_readout(&mut app, &mut scpi, Channel::A).expect("readout handled");
    assert!(!has_readout_update(&task));
    assert!(!app.allows_display(&DisplayTask::UpdateReadout(Channel::A, sample_readout())));
}

#[test]
fn readout_display_resumes_after_settings_close() {
    let (mut app, mut scpi) = standby_app();

    let _ = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSettings(Change::Pressed),
    )
    .expect("settings open");
    let _ = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSettings(Change::Pressed),
    )
    .expect("settings close");
    assert!(!app.settings_open());

    let task = acquire_readout(&mut app, &mut scpi, Channel::A).expect("readout handled");
    assert!(has_readout_update(&task));
}

#[test]
fn settings_close_refreshes_channel_display() {
    let (mut app, mut scpi) = standby_app();

    let _ = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSettings(Change::Pressed),
    )
    .expect("settings open");

    let closed = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSettings(Change::Pressed),
    )
    .expect("settings close");

    assert!(settings_update(&closed) == Some(false));
    assert!(channel_focus(&closed).is_some());
}

#[test]
fn settings_open_does_not_draw_channel_units() {
    let (mut app, mut scpi) = standby_app();

    let opened = open_settings(&mut app, &mut scpi);

    assert!(settings_update(&opened) == Some(true));
    assert!(!has_channel_units_update(&opened, Channel::A));
    assert!(!has_channel_units_update(&opened, Channel::B));
}

#[test]
fn settings_open_exits_setpoint_edit_mode() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonEnter(Change::Pressed),
    )
    .expect("enter setpoint edit");
    assert!(app.is_setpoint_edit());

    let _ = open_settings(&mut app, &mut scpi);
    assert!(app.settings_open());
    assert!(!app.is_setpoint_edit());
}

#[test]
fn switch_blocked_while_settings_open() {
    let (mut app, mut scpi) = standby_app();

    let _ = open_settings(&mut app, &mut scpi);
    assert!(matches!(app.set_state(), SetState::Set));

    assert!(
        press_interface(
            &mut app,
            &mut scpi,
            InterfaceEvent::ButtonSwitch(Change::Pressed),
        )
        .is_none()
    );
    assert!(matches!(app.set_state(), SetState::Set));
}

#[test]
fn enter_and_setpoint_arrows_blocked_while_settings_open() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    let _ = open_settings(&mut app, &mut scpi);

    assert!(
        press_interface(
            &mut app,
            &mut scpi,
            InterfaceEvent::ButtonEnter(Change::Pressed),
        )
        .is_none()
    );
    assert!(!app.is_setpoint_edit());

    assert!(press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonDown).is_none());
    assert!(matches!(app.set_select(Channel::A), SetSelect::Voltage));

    assert!(press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonUp).is_none());
}

#[test]
fn channel_and_navigation_blocked_while_settings_open() {
    let (mut app, mut scpi) = standby_app();

    press_channel(&mut app, &mut scpi, Channel::A);
    let _ = open_settings(&mut app, &mut scpi);

    assert!(press_channel(&mut app, &mut scpi, Channel::B).is_none());
    assert_eq!(app.selected_channel(), Some(Channel::A));
    assert!(press_interface(&mut app, &mut scpi, InterfaceEvent::ButtonRight).is_none());
    assert!(!app.interface_event_allowed(&InterfaceEvent::ButtonSwitch(Change::Pressed)));
}

#[test]
fn settings_close_redraws_channel_units() {
    let (mut app, mut scpi) = standby_app();

    let _ = open_settings(&mut app, &mut scpi);
    let closed = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSettings(Change::Pressed),
    )
    .expect("settings close");

    assert!(has_channel_units_update(&closed, Channel::A));
    assert!(has_channel_units_update(&closed, Channel::B));
}

#[test]
fn switch_press_and_release_updates_button_highlight() {
    let (mut app, mut scpi) = standby_app();

    let pressed = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSwitch(Change::Pressed),
    )
    .expect("switch press");
    let (_, button) = button_update(&pressed).expect("button update");
    assert!(function_eq(button, Some(FunctionButton::Switch)));

    let released = press_interface(
        &mut app,
        &mut scpi,
        InterfaceEvent::ButtonSwitch(Change::Released),
    )
    .expect("switch release");
    let (_, button) = button_update(&released).expect("button update");
    assert!(function_eq(button, None));
}

// --- Protection / hardware (existing) ---

#[test]
fn factory_defaults_match_channel_profiles() {
    let (app, _scpi) = standby_app();
    assert!((app.target_voltage(Channel::A) - CH1_FACTORY.voltage_set).abs() < f32::EPSILON);
    assert!((app.target_current(Channel::A) - CH1_FACTORY.current_set).abs() < f32::EPSILON);
    assert!((app.target_voltage(Channel::B) - CH2_FACTORY.voltage_set).abs() < f32::EPSILON);
    assert!((app.target_current(Channel::B) - CH2_FACTORY.current_set).abs() < f32::EPSILON);
}

#[test]
fn readout_ocp_trips_channel() {
    let (mut app, mut scpi) = standby_app();
    app.set_channel_enable(Channel::A, true);
    app.set_converter_flags(
        Channel::A,
        ConverterFlags {
            enabled: true,
            scp: false,
            ocp: false,
            ovp: false,
        },
    );
    let readout = Readout {
        voltage: 5.0,
        current: 6.0,
        power: 30.0,
    };
    let _ = app.handle_event(
        AppEvent::Hardware(HardwareEvent::ReadoutAcquired(Channel::A, readout)),
        &mut scpi,
    );
    assert!(!app.channel_enable(Channel::A));
    assert_eq!(app.hw_state(Channel::A), ChannelHardwareState::OverCurrent);
    assert!(scpi.prot_latched(ScpiChannel::Ch1));
}

#[test]
fn mcu_temp_trips_both_channels() {
    let (mut app, mut scpi) = standby_app();
    app.set_channel_enable(Channel::A, true);
    app.set_channel_enable(Channel::B, true);
    let temp = TemperatureReading {
        mcu: 50.0,
        ch_a: 25.0,
        ch_b: 25.0,
    };
    let _ = app.handle_event(
        AppEvent::Hardware(HardwareEvent::TempAcquired(temp)),
        &mut scpi,
    );
    assert!(!app.channel_enable(Channel::A));
    assert!(!app.channel_enable(Channel::B));
    assert_eq!(
        app.hw_state(Channel::A),
        ChannelHardwareState::OverTemperature
    );
    assert!(scpi.prot_latched(ScpiChannel::Ch1));
    assert!(scpi.prot_latched(ScpiChannel::Ch2));
}

#[test]
fn converter_cc_flag() {
    let (mut app, mut scpi) = standby_app();
    app.set_channel_enable(Channel::B, true);
    let flags = ConverterFlags {
        enabled: true,
        scp: false,
        ocp: true,
        ovp: false,
    };
    let _ = app.handle_event(
        AppEvent::Hardware(HardwareEvent::ConverterStatusAcquired(Channel::B, flags)),
        &mut scpi,
    );
    assert_eq!(
        app.hw_state(Channel::B),
        ChannelHardwareState::ConstantCurrent
    );
}

#[test]
fn reset_prot_re_derives_state() {
    let (mut app, mut scpi) = standby_app();
    app.set_channel_enable(Channel::A, true);
    app.set_converter_flags(
        Channel::A,
        ConverterFlags {
            enabled: true,
            scp: false,
            ocp: true,
            ovp: false,
        },
    );
    let _ = app.update_hw_state(&mut scpi, Channel::A);
    scpi.set_prot_latched(Some(ScpiChannel::Ch1), true);
    app.set_channel_enable(Channel::A, true);
    let cmd = crate::scpi::parser::parse_command("OUTP:RESET:PROT CH1").unwrap();
    let ctx = crate::scpi::ScpiContext::default();
    let _ = app.handle_scpi(cmd, &mut scpi, &ctx);
    assert!(!scpi.prot_latched(ScpiChannel::Ch1));
    assert_eq!(
        app.hw_state(Channel::A),
        ChannelHardwareState::ConstantCurrent
    );
}
