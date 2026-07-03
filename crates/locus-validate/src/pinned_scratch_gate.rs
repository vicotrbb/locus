use std::fmt;

use locus_alloc::{
    parse_pinned_scratch_pool_probe_output, PinnedScratchPoolProbeOutput,
    PinnedScratchPoolProbeOutputParseError, PinnedScratchPoolProbeStatus,
};

/// Host page-locked scratch validation gate status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchValidationGateStatus {
    /// Host page-locked scratch reuse was proven by the probe output.
    Ready,
    /// Host page-locked scratch reuse was not proven by the probe output.
    NotReady,
}

impl PinnedScratchValidationGateStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NotReady => "not_ready",
        }
    }

    /// Parses a stable machine-readable status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "not_ready" => Some(Self::NotReady),
            _ => None,
        }
    }
}

impl fmt::Display for PinnedScratchValidationGateStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reason for the host page-locked scratch validation gate status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchValidationGateReason {
    /// Host page-locked scratch reuse was proven by the probe output.
    Ready,
    /// First checkout failed.
    CheckoutFailed,
    /// Allocation through the checked-out arena failed or was missing.
    AllocationFailed,
    /// First release failed or was missing.
    ReleaseFailed,
    /// Reuse checkout failed or was missing.
    ReuseCheckoutFailed,
    /// Reuse release failed or was missing.
    ReuseReleaseFailed,
    /// First release did not leave an idle arena in pool stats.
    ReleaseDidNotLeaveIdleArena,
    /// Reuse checkout did not increase reused arena accounting.
    ReuseNotObserved,
    /// Final stats did not show locked bytes remaining in the pool.
    LockedBytesMissing,
}

impl PinnedScratchValidationGateReason {
    /// Returns a stable machine-readable reason string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::CheckoutFailed => "checkout_failed",
            Self::AllocationFailed => "allocation_failed",
            Self::ReleaseFailed => "release_failed",
            Self::ReuseCheckoutFailed => "reuse_checkout_failed",
            Self::ReuseReleaseFailed => "reuse_release_failed",
            Self::ReleaseDidNotLeaveIdleArena => "release_did_not_leave_idle_arena",
            Self::ReuseNotObserved => "reuse_not_observed",
            Self::LockedBytesMissing => "locked_bytes_missing",
        }
    }

    /// Parses a stable machine-readable reason string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "checkout_failed" => Some(Self::CheckoutFailed),
            "allocation_failed" => Some(Self::AllocationFailed),
            "release_failed" => Some(Self::ReleaseFailed),
            "reuse_checkout_failed" => Some(Self::ReuseCheckoutFailed),
            "reuse_release_failed" => Some(Self::ReuseReleaseFailed),
            "release_did_not_leave_idle_arena" => Some(Self::ReleaseDidNotLeaveIdleArena),
            "reuse_not_observed" => Some(Self::ReuseNotObserved),
            "locked_bytes_missing" => Some(Self::LockedBytesMissing),
            _ => None,
        }
    }
}

impl fmt::Display for PinnedScratchValidationGateReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed host page-locked scratch validation gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchValidationGate {
    /// Final gate status.
    pub status: PinnedScratchValidationGateStatus,
    /// Reason for the status.
    pub reason: PinnedScratchValidationGateReason,
    /// Parsed pinned scratch pool probe output.
    pub probe: PinnedScratchPoolProbeOutput,
}

/// Status and reason parsed from a pinned scratch validation gate line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchValidationGateVerdict {
    /// Parsed gate status.
    pub status: PinnedScratchValidationGateStatus,
    /// Parsed gate reason.
    pub reason: PinnedScratchValidationGateReason,
}

impl fmt::Display for PinnedScratchValidationGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pinned_scratch_validation_gate={} reason={}",
            self.status, self.reason
        )
    }
}

impl fmt::Display for PinnedScratchValidationGateVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pinned_scratch_validation_gate={} reason={}",
            self.status, self.reason
        )
    }
}

impl PinnedScratchValidationGateVerdict {
    /// Builds a verdict only when the status and reason are coherent.
    ///
    /// # Errors
    ///
    /// Returns an error when the reason is not valid for the status.
    pub fn from_parts(
        status: PinnedScratchValidationGateStatus,
        reason: PinnedScratchValidationGateReason,
    ) -> Result<Self, PinnedScratchValidationGateLineParseError> {
        let verdict = Self { status, reason };
        if verdict.is_consistent() {
            Ok(verdict)
        } else {
            Err(PinnedScratchValidationGateLineParseError::InconsistentVerdict { status, reason })
        }
    }

    /// Returns true when the reason is valid for the status.
    #[must_use]
    pub fn is_consistent(self) -> bool {
        matches!(
            (self.status, self.reason),
            (
                PinnedScratchValidationGateStatus::Ready,
                PinnedScratchValidationGateReason::Ready
            ) | (
                PinnedScratchValidationGateStatus::NotReady,
                PinnedScratchValidationGateReason::CheckoutFailed
                    | PinnedScratchValidationGateReason::AllocationFailed
                    | PinnedScratchValidationGateReason::ReleaseFailed
                    | PinnedScratchValidationGateReason::ReuseCheckoutFailed
                    | PinnedScratchValidationGateReason::ReuseReleaseFailed
                    | PinnedScratchValidationGateReason::ReleaseDidNotLeaveIdleArena
                    | PinnedScratchValidationGateReason::ReuseNotObserved
                    | PinnedScratchValidationGateReason::LockedBytesMissing
            )
        )
    }
}

/// Error returned when parsing a pinned scratch validation gate line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchValidationGateLineParseError {
    /// The line does not contain a `pinned_scratch_validation_gate=` token.
    MissingStatus,
    /// The line does not contain a `reason=` token.
    MissingReason,
    /// The line contains a duplicate `pinned_scratch_validation_gate=` token.
    DuplicateStatus,
    /// The line contains a duplicate `reason=` token.
    DuplicateReason,
    /// The line contains a token outside the pinned scratch validation gate schema.
    InvalidToken(String),
    /// The gate status token is not recognized.
    UnknownStatus(String),
    /// The gate reason token is not recognized.
    UnknownReason(String),
    /// The status and reason tokens are individually valid but inconsistent together.
    InconsistentVerdict {
        /// Parsed gate status.
        status: PinnedScratchValidationGateStatus,
        /// Parsed gate reason.
        reason: PinnedScratchValidationGateReason,
    },
}

impl fmt::Display for PinnedScratchValidationGateLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStatus => f.write_str("missing pinned_scratch_validation_gate token"),
            Self::MissingReason => f.write_str("missing reason token"),
            Self::DuplicateStatus => f.write_str("duplicate pinned_scratch_validation_gate token"),
            Self::DuplicateReason => f.write_str("duplicate reason token"),
            Self::InvalidToken(token) => {
                write!(f, "invalid pinned scratch validation gate token: {token}")
            }
            Self::UnknownStatus(status) => {
                write!(f, "unknown pinned scratch validation gate status: {status}")
            }
            Self::UnknownReason(reason) => {
                write!(f, "unknown pinned scratch validation gate reason: {reason}")
            }
            Self::InconsistentVerdict { status, reason } => {
                write!(
                    f,
                    "inconsistent pinned scratch validation gate: {status} {reason}"
                )
            }
        }
    }
}

impl std::error::Error for PinnedScratchValidationGateLineParseError {}

/// Error returned when extracting a pinned scratch validation gate from output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchValidationGateOutputParseError {
    /// The output does not contain a `pinned_scratch_validation_gate=` line.
    MissingGateLine,
    /// The output contains more than one `pinned_scratch_validation_gate=` line.
    DuplicateGateLine,
    /// The discovered gate line is malformed.
    Line(PinnedScratchValidationGateLineParseError),
}

impl fmt::Display for PinnedScratchValidationGateOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingGateLine => f.write_str("missing pinned_scratch_validation_gate line"),
            Self::DuplicateGateLine => f.write_str("duplicate pinned_scratch_validation_gate line"),
            Self::Line(source) => {
                write!(f, "invalid pinned_scratch_validation_gate line: {source}")
            }
        }
    }
}

impl std::error::Error for PinnedScratchValidationGateOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line(source) => Some(source),
            Self::MissingGateLine | Self::DuplicateGateLine => None,
        }
    }
}

/// Error returned when evaluating pinned scratch pool probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchValidationGateParseError {
    /// Pinned scratch pool probe output was missing or malformed.
    Probe(PinnedScratchPoolProbeOutputParseError),
}

impl fmt::Display for PinnedScratchValidationGateParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Probe(source) => write!(f, "invalid pinned scratch pool output: {source}"),
        }
    }
}

impl std::error::Error for PinnedScratchValidationGateParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Probe(source) => Some(source),
        }
    }
}

impl PinnedScratchValidationGate {
    /// Builds a gate from parsed pinned scratch pool probe output.
    #[must_use]
    pub fn from_probe(probe: PinnedScratchPoolProbeOutput) -> Self {
        let reason = pinned_scratch_not_ready_reason(&probe);
        let status = if reason == PinnedScratchValidationGateReason::Ready {
            PinnedScratchValidationGateStatus::Ready
        } else {
            PinnedScratchValidationGateStatus::NotReady
        };

        Self {
            status,
            reason,
            probe,
        }
    }

    /// Returns true only when host page-locked scratch reuse is proven.
    #[must_use]
    pub fn is_ready(self) -> bool {
        self.status == PinnedScratchValidationGateStatus::Ready
    }
}

/// Parses pinned scratch pool probe output and returns the validation gate.
///
/// # Errors
///
/// Returns an error when the probe output is missing required stable lines or
/// contains malformed stable lines.
pub fn evaluate_pinned_scratch_validation_output(
    output: &str,
) -> Result<PinnedScratchValidationGate, PinnedScratchValidationGateParseError> {
    let probe = parse_pinned_scratch_pool_probe_output(output)
        .map_err(PinnedScratchValidationGateParseError::Probe)?;
    Ok(PinnedScratchValidationGate::from_probe(probe))
}

/// Parses a pinned scratch validation gate verdict line.
///
/// The expected format is `pinned_scratch_validation_gate=<status> reason=<reason>`.
///
/// # Errors
///
/// Returns an error when the line is missing required tokens, contains duplicate
/// tokens, contains unsupported tokens, uses an unknown status or reason, or
/// combines a status with an incoherent reason.
pub fn parse_pinned_scratch_validation_gate_line(
    line: &str,
) -> Result<PinnedScratchValidationGateVerdict, PinnedScratchValidationGateLineParseError> {
    let mut status_token = None;
    let mut reason_token = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(PinnedScratchValidationGateLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "pinned_scratch_validation_gate" => {
                if status_token.replace(value).is_some() {
                    return Err(PinnedScratchValidationGateLineParseError::DuplicateStatus);
                }
            }
            "reason" => {
                if reason_token.replace(value).is_some() {
                    return Err(PinnedScratchValidationGateLineParseError::DuplicateReason);
                }
            }
            _ => {
                return Err(PinnedScratchValidationGateLineParseError::InvalidToken(
                    token.to_owned(),
                ));
            }
        }
    }

    let status_token =
        status_token.ok_or(PinnedScratchValidationGateLineParseError::MissingStatus)?;
    let reason_token =
        reason_token.ok_or(PinnedScratchValidationGateLineParseError::MissingReason)?;

    let status =
        PinnedScratchValidationGateStatus::from_str_token(status_token).ok_or_else(|| {
            PinnedScratchValidationGateLineParseError::UnknownStatus(status_token.to_owned())
        })?;
    let reason =
        PinnedScratchValidationGateReason::from_str_token(reason_token).ok_or_else(|| {
            PinnedScratchValidationGateLineParseError::UnknownReason(reason_token.to_owned())
        })?;

    PinnedScratchValidationGateVerdict::from_parts(status, reason)
}

/// Extracts a pinned scratch validation gate verdict from multiline output.
///
/// # Errors
///
/// Returns an error when the output has no pinned scratch validation gate line,
/// has more than one pinned scratch validation gate line, or contains a
/// malformed pinned scratch validation gate line.
pub fn parse_pinned_scratch_validation_gate_output(
    output: &str,
) -> Result<PinnedScratchValidationGateVerdict, PinnedScratchValidationGateOutputParseError> {
    let mut gate = None;

    for line in output.lines().map(str::trim) {
        if !line.starts_with("pinned_scratch_validation_gate=") {
            continue;
        }

        if gate.is_some() {
            return Err(PinnedScratchValidationGateOutputParseError::DuplicateGateLine);
        }

        gate = Some(
            parse_pinned_scratch_validation_gate_line(line)
                .map_err(PinnedScratchValidationGateOutputParseError::Line)?,
        );
    }

    gate.ok_or(PinnedScratchValidationGateOutputParseError::MissingGateLine)
}

fn pinned_scratch_not_ready_reason(
    probe: &PinnedScratchPoolProbeOutput,
) -> PinnedScratchValidationGateReason {
    if probe.checkout.status != PinnedScratchPoolProbeStatus::Ok {
        return PinnedScratchValidationGateReason::CheckoutFailed;
    }

    match probe.allocation {
        Some(event) if event.status == PinnedScratchPoolProbeStatus::Ok => {}
        _ => return PinnedScratchValidationGateReason::AllocationFailed,
    }
    match probe.release {
        Some(event) if event.status == PinnedScratchPoolProbeStatus::Ok => {}
        _ => return PinnedScratchValidationGateReason::ReleaseFailed,
    }
    match probe.reuse_checkout {
        Some(event) if event.status == PinnedScratchPoolProbeStatus::Ok => {}
        _ => return PinnedScratchValidationGateReason::ReuseCheckoutFailed,
    }
    match probe.reuse_release {
        Some(event) if event.status == PinnedScratchPoolProbeStatus::Ok => {}
        _ => return PinnedScratchValidationGateReason::ReuseReleaseFailed,
    }

    match probe.after_release_stats {
        Some(stats) if stats.idle >= 1 => {}
        _ => return PinnedScratchValidationGateReason::ReleaseDidNotLeaveIdleArena,
    }
    match probe.after_reuse_checkout_stats {
        Some(stats) if stats.reused_arenas >= 1 => {}
        _ => return PinnedScratchValidationGateReason::ReuseNotObserved,
    }
    match probe.after_reuse_release_stats {
        Some(stats) if stats.locked_bytes > 0 => {}
        _ => return PinnedScratchValidationGateReason::LockedBytesMissing,
    }

    PinnedScratchValidationGateReason::Ready
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_pinned_scratch_validation_output, parse_pinned_scratch_validation_gate_line,
        parse_pinned_scratch_validation_gate_output, PinnedScratchValidationGateLineParseError,
        PinnedScratchValidationGateOutputParseError, PinnedScratchValidationGateParseError,
        PinnedScratchValidationGateReason, PinnedScratchValidationGateStatus,
        PinnedScratchValidationGateVerdict,
    };

    const READY_OUTPUT: &str = "\
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
checked_out_mapping_start=0xffffab7fb000
checked_out_mapping_len=20479
checked_out_allocation=ok bytes=256
pool_stats phase=after_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=0
pool_release=ok handle=0
pool_stats phase=after_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1
pool_reuse_checkout=ok handle=1
pool_stats phase=after_reuse_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=1
pool_reuse_release=ok handle=1
pool_stats phase=after_reuse_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=2
";

    #[test]
    fn reports_ready_gate_from_probe_output() {
        let gate = evaluate_pinned_scratch_validation_output(READY_OUTPUT).expect("gate");

        assert_eq!(gate.status, PinnedScratchValidationGateStatus::Ready);
        assert_eq!(gate.reason, PinnedScratchValidationGateReason::Ready);
        assert_eq!(
            gate.to_string(),
            "pinned_scratch_validation_gate=ready reason=ready"
        );
        assert!(gate.is_ready());
    }

    #[test]
    fn reports_checkout_failure_as_not_ready() {
        let output = "\
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=error
pool_checkout_error=pinned scratch pool arena failed
pool_stats phase=checkout_error locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
";

        let gate = evaluate_pinned_scratch_validation_output(output).expect("gate");

        assert_eq!(gate.status, PinnedScratchValidationGateStatus::NotReady);
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::CheckoutFailed
        );
        assert!(!gate.is_ready());
    }

    #[test]
    fn reports_event_failures_as_not_ready() {
        let allocation_error = READY_OUTPUT.replace(
            "checked_out_allocation=ok bytes=256",
            "checked_out_allocation=error",
        );
        let gate = evaluate_pinned_scratch_validation_output(&allocation_error).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::AllocationFailed
        );

        let release_error = READY_OUTPUT.replace("pool_release=ok handle=0", "pool_release=error");
        let gate = evaluate_pinned_scratch_validation_output(&release_error).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::ReleaseFailed
        );

        let reuse_checkout_error = READY_OUTPUT.replace(
            "pool_reuse_checkout=ok handle=1",
            "pool_reuse_checkout=error",
        );
        let gate = evaluate_pinned_scratch_validation_output(&reuse_checkout_error).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::ReuseCheckoutFailed
        );

        let reuse_release_error =
            READY_OUTPUT.replace("pool_reuse_release=ok handle=1", "pool_reuse_release=error");
        let gate = evaluate_pinned_scratch_validation_output(&reuse_release_error).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::ReuseReleaseFailed
        );
    }

    #[test]
    fn reports_accounting_failures_as_not_ready() {
        let no_idle_after_release = READY_OUTPUT.replace(
            "phase=after_release locked_bytes=20479 checked_out=0 idle=1",
            "phase=after_release locked_bytes=20479 checked_out=0 idle=0",
        );
        let gate = evaluate_pinned_scratch_validation_output(&no_idle_after_release).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::ReleaseDidNotLeaveIdleArena
        );

        let no_reuse_observed = READY_OUTPUT.replace(
            "phase=after_reuse_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=1",
            "phase=after_reuse_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=0",
        );
        let gate = evaluate_pinned_scratch_validation_output(&no_reuse_observed).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::ReuseNotObserved
        );

        let no_locked_bytes = READY_OUTPUT.replace(
            "phase=after_reuse_release locked_bytes=20479",
            "phase=after_reuse_release locked_bytes=0",
        );
        let gate = evaluate_pinned_scratch_validation_output(&no_locked_bytes).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::LockedBytesMissing
        );
    }

    #[test]
    fn reports_probe_parse_errors() {
        let error = evaluate_pinned_scratch_validation_output("pool_checkout=ok handle=0\n")
            .expect_err("missing initial stats");

        assert!(matches!(
            error,
            PinnedScratchValidationGateParseError::Probe(_)
        ));
    }

    #[test]
    fn parses_gate_lines() {
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready reason=ready"
            )
            .expect("ready"),
            PinnedScratchValidationGateVerdict {
                status: PinnedScratchValidationGateStatus::Ready,
                reason: PinnedScratchValidationGateReason::Ready,
            }
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=not_ready reason=reuse_not_observed"
            )
            .expect("not ready"),
            PinnedScratchValidationGateVerdict {
                status: PinnedScratchValidationGateStatus::NotReady,
                reason: PinnedScratchValidationGateReason::ReuseNotObserved,
            }
        );
        assert_eq!(
            PinnedScratchValidationGateStatus::Ready.to_string(),
            "ready"
        );
        assert_eq!(
            PinnedScratchValidationGateReason::LockedBytesMissing.to_string(),
            "locked_bytes_missing"
        );
        assert!(PinnedScratchValidationGateVerdict {
            status: PinnedScratchValidationGateStatus::NotReady,
            reason: PinnedScratchValidationGateReason::AllocationFailed,
        }
        .is_consistent());
        assert!(!PinnedScratchValidationGateVerdict {
            status: PinnedScratchValidationGateStatus::Ready,
            reason: PinnedScratchValidationGateReason::AllocationFailed,
        }
        .is_consistent());
    }

    #[test]
    fn rejects_invalid_gate_lines() {
        assert_eq!(
            parse_pinned_scratch_validation_gate_line("reason=ready").expect_err("missing status"),
            PinnedScratchValidationGateLineParseError::MissingStatus
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line("pinned_scratch_validation_gate=ready")
                .expect_err("missing reason"),
            PinnedScratchValidationGateLineParseError::MissingReason
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=maybe reason=ready"
            )
            .expect_err("unknown status"),
            PinnedScratchValidationGateLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready reason=maybe"
            )
            .expect_err("unknown reason"),
            PinnedScratchValidationGateLineParseError::UnknownReason("maybe".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready reason=ready extra=true"
            )
            .expect_err("extra token"),
            PinnedScratchValidationGateLineParseError::InvalidToken("extra=true".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready pinned_scratch_validation_gate=not_ready reason=ready"
            )
            .expect_err("duplicate status"),
            PinnedScratchValidationGateLineParseError::DuplicateStatus
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready reason=ready reason=checkout_failed"
            )
            .expect_err("duplicate reason"),
            PinnedScratchValidationGateLineParseError::DuplicateReason
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready reason=checkout_failed"
            )
            .expect_err("inconsistent verdict"),
            PinnedScratchValidationGateLineParseError::InconsistentVerdict {
                status: PinnedScratchValidationGateStatus::Ready,
                reason: PinnedScratchValidationGateReason::CheckoutFailed,
            }
        );
    }

    #[test]
    fn parses_gate_from_output() {
        let output = "\
pool_checkout=ok handle=0
pinned_scratch_validation_gate=ready reason=ready
";

        assert_eq!(
            parse_pinned_scratch_validation_gate_output(output).expect("gate"),
            PinnedScratchValidationGateVerdict {
                status: PinnedScratchValidationGateStatus::Ready,
                reason: PinnedScratchValidationGateReason::Ready,
            }
        );
    }

    #[test]
    fn rejects_invalid_gate_output() {
        assert_eq!(
            parse_pinned_scratch_validation_gate_output("pool_checkout=ok handle=0\n")
                .expect_err("missing gate"),
            PinnedScratchValidationGateOutputParseError::MissingGateLine
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_output(
                "pinned_scratch_validation_gate=ready reason=ready\npinned_scratch_validation_gate=not_ready reason=checkout_failed\n"
            )
            .expect_err("duplicate gate"),
            PinnedScratchValidationGateOutputParseError::DuplicateGateLine
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_output(
                "pinned_scratch_validation_gate=maybe reason=ready\n"
            )
            .expect_err("bad gate"),
            PinnedScratchValidationGateOutputParseError::Line(
                PinnedScratchValidationGateLineParseError::UnknownStatus("maybe".to_owned())
            )
        );
    }
}
