//! Host-testable firmware update session state machine.

use protov_nvm::{FWUP_MAX_BLOCK_LEN, FWUP_MAX_IMAGE_SIZE, FWUP_SIGNATURE_LEN};

use crate::dfu::block::is_valid_block_len;
use crate::dfu::error::DfuError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DfuPhase {
    Idle,
    Preparing { total: u32 },
    Receiving { total: u32, received: u32 },
    Ready { total: u32 },
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DfuAction {
    Prepare,
    WriteBlock { offset: u32, len: u32 },
    VerifyApply { len: u32, signature: [u8; FWUP_SIGNATURE_LEN] },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DfuSession {
    phase: DfuPhase,
}

impl Default for DfuSession {
    fn default() -> Self {
        Self { phase: DfuPhase::Idle }
    }
}

impl DfuSession {
    pub const fn new() -> Self {
        Self {
            phase: DfuPhase::Idle,
        }
    }

    pub const fn phase(&self) -> DfuPhase {
        self.phase
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.phase, DfuPhase::Idle)
    }

    pub fn accepts_data(&self) -> bool {
        matches!(
            self.phase,
            DfuPhase::Receiving {
                received: _,
                total: _,
            }
        )
    }

    pub fn format_stat(&self, buf: &mut heapless::String<64>) {
        buf.clear();
        match self.phase {
            DfuPhase::Idle => {
                let _ = buf.push_str("IDLE");
            }
            DfuPhase::Preparing { total } => {
                let _ = core::fmt::write(buf, format_args!("PREPARE,{total}"));
            }
            DfuPhase::Receiving { total, received } => {
                let _ = core::fmt::write(buf, format_args!("RECV,{received}/{total}"));
            }
            DfuPhase::Ready { total } => {
                let _ = core::fmt::write(buf, format_args!("READY,{total}"));
            }
            DfuPhase::Error => {
                let _ = buf.push_str("ERROR");
            }
        }
    }

    pub fn to_status(&self) -> crate::model::DfuStatus {
        use crate::model::DfuStatus;
        match self.phase {
            DfuPhase::Idle => DfuStatus::Idle,
            DfuPhase::Preparing { total } => DfuStatus::Preparing { total },
            DfuPhase::Receiving { total, received } => DfuStatus::Receiving { received, total },
            DfuPhase::Ready { total } => DfuStatus::Ready { total },
            DfuPhase::Error => DfuStatus::Error,
        }
    }

    pub fn start(&mut self, total: u32) -> Result<DfuAction, DfuError> {
        if total == 0 || total > FWUP_MAX_IMAGE_SIZE {
            return Err(DfuError::InvalidSize);
        }
        if !matches!(self.phase, DfuPhase::Idle) {
            return Err(DfuError::WrongState);
        }
        self.phase = DfuPhase::Preparing { total };
        Ok(DfuAction::Prepare)
    }

    pub fn on_prepare_complete(&mut self) -> Result<(), DfuError> {
        let DfuPhase::Preparing { total } = self.phase else {
            return Err(DfuError::WrongState);
        };
        self.phase = DfuPhase::Receiving { total, received: 0 };
        Ok(())
    }

    pub fn on_prepare_failed(&mut self) {
        self.phase = DfuPhase::Error;
    }

    pub fn accept_block(&mut self, len: u32) -> Result<(u32, DfuAction), DfuError> {
        if len == 0 {
            return Err(DfuError::EmptyBlock);
        }
        if len as usize > FWUP_MAX_BLOCK_LEN {
            return Err(DfuError::BlockTooLarge);
        }
        if !is_valid_block_len(len as usize) {
            return Err(DfuError::BlockTooLarge);
        }

        let DfuPhase::Receiving { total, received } = self.phase else {
            return Err(DfuError::WrongState);
        };

        let new_received = received
            .checked_add(len)
            .ok_or(DfuError::Overflow)?;
        if new_received > total {
            return Err(DfuError::Overflow);
        }

        let offset = received;
        if new_received == total {
            self.phase = DfuPhase::Ready { total };
        } else {
            self.phase = DfuPhase::Receiving {
                total,
                received: new_received,
            };
        }

        Ok((
            offset,
            DfuAction::WriteBlock {
                offset,
                len,
            },
        ))
    }

    pub fn on_block_failed(&mut self) {
        self.phase = DfuPhase::Error;
    }

    pub fn apply(
        &mut self,
        signature: [u8; FWUP_SIGNATURE_LEN],
    ) -> Result<DfuAction, DfuError> {
        let DfuPhase::Ready { total } = self.phase else {
            return Err(DfuError::WrongState);
        };
        Ok(DfuAction::VerifyApply { len: total, signature })
    }

    pub fn on_verify_failed(&mut self) {
        self.phase = DfuPhase::Error;
    }

    pub fn abort(&mut self) {
        self.phase = DfuPhase::Idle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path() {
        let mut s = DfuSession::new();
        assert_eq!(s.start(8192).unwrap(), DfuAction::Prepare);
        s.on_prepare_complete().unwrap();
        assert_eq!(
            s.accept_block(4096).unwrap(),
            (
                0,
                DfuAction::WriteBlock {
                    offset: 0,
                    len: 4096
                }
            )
        );
        assert_eq!(
            s.accept_block(4096).unwrap(),
            (
                4096,
                DfuAction::WriteBlock {
                    offset: 4096,
                    len: 4096
                }
            )
        );
        assert!(matches!(s.phase(), DfuPhase::Ready { total: 8192 }));
        let sig = [0u8; 64];
        match s.apply(sig).unwrap() {
            DfuAction::VerifyApply { len, signature } => {
                assert_eq!(len, 8192);
                assert_eq!(signature, sig);
            }
            _ => panic!("expected verify"),
        }
    }

    #[test]
    fn reject_oversize() {
        let mut s = DfuSession::new();
        assert_eq!(s.start(FWUP_MAX_IMAGE_SIZE + 1), Err(DfuError::InvalidSize));
    }

    #[test]
    fn reject_overflow() {
        let mut s = DfuSession::new();
        s.start(10).unwrap();
        s.on_prepare_complete().unwrap();
        assert_eq!(s.accept_block(11), Err(DfuError::Overflow));
    }

    #[test]
    fn abort_resets() {
        let mut s = DfuSession::new();
        s.start(10).unwrap();
        s.abort();
        assert_eq!(s.phase(), DfuPhase::Idle);
    }

    #[test]
    fn apply_before_complete() {
        let mut s = DfuSession::new();
        s.start(10).unwrap();
        s.on_prepare_complete().unwrap();
        assert_eq!(s.apply([0; 64]), Err(DfuError::WrongState));
    }

    #[test]
    fn reject_block_while_preparing() {
        let mut s = DfuSession::new();
        s.start(4096).unwrap();
        assert_eq!(s.accept_block(4096), Err(DfuError::WrongState));
    }
}
