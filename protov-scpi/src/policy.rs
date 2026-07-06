use crate::command::ScpiCommand;

pub fn is_mutation(cmd: &ScpiCommand) -> bool {
    !matches!(
        cmd,
        ScpiCommand::IdnQuery
            | ScpiCommand::MeasQuery { .. }
            | ScpiCommand::ChannelQuery { .. }
            | ScpiCommand::OutputQuery { .. }
            | ScpiCommand::LcdBrightnessQuery
            | ScpiCommand::LedBrightnessQuery
            | ScpiCommand::SystErrQuery
            | ScpiCommand::SystVersQuery
            | ScpiCommand::TelemQuery
            | ScpiCommand::TempQuery { .. }
            | ScpiCommand::InpQuery
            | ScpiCommand::DiagQuery
            | ScpiCommand::Ina226RegQuery { .. }
            | ScpiCommand::Tps55289RegQuery { .. }
            | ScpiCommand::FwupStatQuery
            | ScpiCommand::Unknown { .. }
    )
}

pub fn is_allowed_in_update_mode(cmd: &ScpiCommand) -> bool {
    matches!(
        cmd,
        ScpiCommand::IdnQuery
            | ScpiCommand::SystErrQuery
            | ScpiCommand::SystVersQuery
            | ScpiCommand::FwupStatQuery
            | ScpiCommand::FwupData
            | ScpiCommand::FwupAbor
            | ScpiCommand::FwupAppl { .. }
    )
}

pub fn requires_active_update_session(cmd: &ScpiCommand) -> bool {
    matches!(cmd, ScpiCommand::FwupData | ScpiCommand::FwupAppl { .. })
}
