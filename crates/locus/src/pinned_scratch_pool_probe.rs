use std::fmt;

/// Stable status for pinned scratch pool probe events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchPoolProbeStatus {
    /// The operation succeeded.
    Ok,
    /// The operation failed.
    Error,
}

impl PinnedScratchPoolProbeStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }

    /// Parses a stable machine-readable status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ok" => Some(Self::Ok),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

impl fmt::Display for PinnedScratchPoolProbeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable pinned scratch pool probe event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchPoolProbeEvent {
    /// `pool_checkout=<status>`.
    Checkout,
    /// `checked_out_allocation=<status>`.
    Allocation,
    /// `pool_release=<status>`.
    Release,
    /// `pool_reuse_checkout=<status>`.
    ReuseCheckout,
    /// `pool_reuse_release=<status>`.
    ReuseRelease,
}

impl PinnedScratchPoolProbeEvent {
    /// Returns the stable machine-readable field name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Checkout => "pool_checkout",
            Self::Allocation => "checked_out_allocation",
            Self::Release => "pool_release",
            Self::ReuseCheckout => "pool_reuse_checkout",
            Self::ReuseRelease => "pool_reuse_release",
        }
    }

    /// Parses a stable machine-readable field name.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "pool_checkout" => Some(Self::Checkout),
            "checked_out_allocation" => Some(Self::Allocation),
            "pool_release" => Some(Self::Release),
            "pool_reuse_checkout" => Some(Self::ReuseCheckout),
            "pool_reuse_release" => Some(Self::ReuseRelease),
            _ => None,
        }
    }

    fn requires_handle(self) -> bool {
        matches!(
            self,
            Self::Checkout | Self::Release | Self::ReuseCheckout | Self::ReuseRelease
        )
    }
}

impl fmt::Display for PinnedScratchPoolProbeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable pinned scratch pool stats phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchPoolProbePhase {
    /// Initial pool stats.
    Initial,
    /// Stats after checkout failed.
    CheckoutError,
    /// Stats after the first checkout.
    AfterCheckout,
    /// Stats after the first release.
    AfterRelease,
    /// Stats after reuse checkout.
    AfterReuseCheckout,
    /// Stats after reuse release.
    AfterReuseRelease,
}

impl PinnedScratchPoolProbePhase {
    /// Returns the stable machine-readable phase string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::CheckoutError => "checkout_error",
            Self::AfterCheckout => "after_checkout",
            Self::AfterRelease => "after_release",
            Self::AfterReuseCheckout => "after_reuse_checkout",
            Self::AfterReuseRelease => "after_reuse_release",
        }
    }

    /// Parses a stable machine-readable phase string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "initial" => Some(Self::Initial),
            "checkout_error" => Some(Self::CheckoutError),
            "after_checkout" => Some(Self::AfterCheckout),
            "after_release" => Some(Self::AfterRelease),
            "after_reuse_checkout" => Some(Self::AfterReuseCheckout),
            "after_reuse_release" => Some(Self::AfterReuseRelease),
            _ => None,
        }
    }
}

impl fmt::Display for PinnedScratchPoolProbePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed pinned scratch pool probe event line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchPoolProbeEventLine {
    /// Parsed event field.
    pub event: PinnedScratchPoolProbeEvent,
    /// Parsed event status.
    pub status: PinnedScratchPoolProbeStatus,
    /// Parsed checkout or release handle, when present.
    pub handle: Option<u64>,
    /// Parsed allocation byte count, when present.
    pub bytes: Option<usize>,
}

/// Parsed pinned scratch pool probe stats line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchPoolProbeStatsLine {
    /// Stats phase.
    pub phase: PinnedScratchPoolProbePhase,
    /// Locked bytes owned by the pool.
    pub locked_bytes: usize,
    /// Checked-out arena count.
    pub checked_out: usize,
    /// Idle arena count.
    pub idle: usize,
    /// Arenas created by the pool.
    pub created_arenas: u64,
    /// Checkouts served from idle arenas.
    pub reused_arenas: u64,
    /// Successful checkout count.
    pub checkout_count: u64,
    /// Successful release count.
    pub release_count: u64,
}

/// Parsed pinned scratch pool probe output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchPoolProbeOutput {
    /// Initial pool stats.
    pub initial_stats: PinnedScratchPoolProbeStatsLine,
    /// First checkout event.
    pub checkout: PinnedScratchPoolProbeEventLine,
    /// Stats after checkout failed, when checkout failed.
    pub checkout_error_stats: Option<PinnedScratchPoolProbeStatsLine>,
    /// Allocation event, required when checkout succeeded.
    pub allocation: Option<PinnedScratchPoolProbeEventLine>,
    /// Release event, required when checkout succeeded.
    pub release: Option<PinnedScratchPoolProbeEventLine>,
    /// Reuse checkout event, required when checkout succeeded.
    pub reuse_checkout: Option<PinnedScratchPoolProbeEventLine>,
    /// Reuse release event, required when checkout succeeded.
    pub reuse_release: Option<PinnedScratchPoolProbeEventLine>,
    /// Stats after the first checkout, required when checkout succeeded.
    pub after_checkout_stats: Option<PinnedScratchPoolProbeStatsLine>,
    /// Stats after the first release, required when checkout succeeded.
    pub after_release_stats: Option<PinnedScratchPoolProbeStatsLine>,
    /// Stats after reuse checkout, required when checkout succeeded.
    pub after_reuse_checkout_stats: Option<PinnedScratchPoolProbeStatsLine>,
    /// Stats after reuse release, required when checkout succeeded.
    pub after_reuse_release_stats: Option<PinnedScratchPoolProbeStatsLine>,
}

/// Error returned when parsing pinned scratch pool probe lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchPoolProbeLineParseError {
    /// The line does not contain a supported event token.
    MissingEvent,
    /// The line contains a duplicate event token.
    DuplicateEvent,
    /// The line contains a token outside the stable schema.
    InvalidToken(String),
    /// The event field is not recognized.
    UnknownEvent(String),
    /// The status token is not recognized.
    UnknownStatus(String),
    /// The stats phase token is not recognized.
    UnknownPhase(String),
    /// The field is not recognized.
    UnknownField(String),
    /// The field appears more than once.
    DuplicateField(String),
    /// A required field is missing.
    MissingField(String),
    /// A numeric field is malformed.
    InvalidNumber {
        /// Field name.
        field: String,
        /// Rejected value.
        value: String,
    },
}

/// Error returned when extracting pinned scratch pool probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchPoolProbeOutputParseError {
    /// The output does not contain initial stats.
    MissingInitialStats,
    /// The output does not contain a first checkout line.
    MissingCheckoutLine,
    /// The output is missing a required successful event line.
    MissingEvent(PinnedScratchPoolProbeEvent),
    /// The output is missing a required stats phase.
    MissingStats(PinnedScratchPoolProbePhase),
    /// The output contains more than one line for an event.
    DuplicateEvent(PinnedScratchPoolProbeEvent),
    /// The output contains more than one line for a stats phase.
    DuplicateStats(PinnedScratchPoolProbePhase),
    /// A stable event line is malformed.
    EventLine(PinnedScratchPoolProbeLineParseError),
    /// A stable stats line is malformed.
    StatsLine(PinnedScratchPoolProbeLineParseError),
}

impl fmt::Display for PinnedScratchPoolProbeLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEvent => f.write_str("missing pinned scratch pool probe event"),
            Self::DuplicateEvent => f.write_str("duplicate pinned scratch pool probe event"),
            Self::InvalidToken(token) => {
                write!(f, "invalid pinned scratch pool probe token: {token}")
            }
            Self::UnknownEvent(event) => {
                write!(f, "unknown pinned scratch pool probe event: {event}")
            }
            Self::UnknownStatus(status) => {
                write!(f, "unknown pinned scratch pool probe status: {status}")
            }
            Self::UnknownPhase(phase) => {
                write!(f, "unknown pinned scratch pool probe phase: {phase}")
            }
            Self::UnknownField(field) => {
                write!(f, "unknown pinned scratch pool probe field: {field}")
            }
            Self::DuplicateField(field) => {
                write!(f, "duplicate pinned scratch pool probe field: {field}")
            }
            Self::MissingField(field) => {
                write!(f, "missing pinned scratch pool probe field: {field}")
            }
            Self::InvalidNumber { field, value } => write!(
                f,
                "invalid pinned scratch pool probe number for {field}: {value}"
            ),
        }
    }
}

impl std::error::Error for PinnedScratchPoolProbeLineParseError {}

impl fmt::Display for PinnedScratchPoolProbeOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInitialStats => f.write_str("missing initial pinned scratch pool stats"),
            Self::MissingCheckoutLine => f.write_str("missing pool_checkout line"),
            Self::MissingEvent(event) => {
                write!(f, "missing pinned scratch pool event line: {event}")
            }
            Self::MissingStats(phase) => {
                write!(f, "missing pinned scratch pool stats phase: {phase}")
            }
            Self::DuplicateEvent(event) => {
                write!(f, "duplicate pinned scratch pool event line: {event}")
            }
            Self::DuplicateStats(phase) => {
                write!(f, "duplicate pinned scratch pool stats phase: {phase}")
            }
            Self::EventLine(source) => {
                write!(f, "invalid pinned scratch pool event line: {source}")
            }
            Self::StatsLine(source) => {
                write!(f, "invalid pinned scratch pool stats line: {source}")
            }
        }
    }
}

impl std::error::Error for PinnedScratchPoolProbeOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EventLine(source) | Self::StatsLine(source) => Some(source),
            Self::MissingInitialStats
            | Self::MissingCheckoutLine
            | Self::MissingEvent(_)
            | Self::MissingStats(_)
            | Self::DuplicateEvent(_)
            | Self::DuplicateStats(_) => None,
        }
    }
}

/// Parses one pinned scratch pool probe event line.
///
/// # Errors
///
/// Returns an error when the line is missing an event token, contains duplicate
/// fields, contains unsupported tokens, uses an unknown status, or omits a
/// required handle or byte count.
pub fn parse_pinned_scratch_pool_probe_event_line(
    line: &str,
) -> Result<PinnedScratchPoolProbeEventLine, PinnedScratchPoolProbeLineParseError> {
    let mut event = None;
    let mut status_token = None;
    let mut handle = None;
    let mut bytes = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(PinnedScratchPoolProbeLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        if let Some(parsed_event) = PinnedScratchPoolProbeEvent::from_str_token(key) {
            if event.replace(parsed_event).is_some() {
                return Err(PinnedScratchPoolProbeLineParseError::DuplicateEvent);
            }
            if status_token.replace(value).is_some() {
                return Err(PinnedScratchPoolProbeLineParseError::DuplicateEvent);
            }
            continue;
        }

        match key {
            "handle" => {
                if handle.replace(parse_probe_u64_field(key, value)?).is_some() {
                    return Err(PinnedScratchPoolProbeLineParseError::DuplicateField(
                        key.to_owned(),
                    ));
                }
            }
            "bytes" => {
                if bytes
                    .replace(parse_probe_usize_field(key, value)?)
                    .is_some()
                {
                    return Err(PinnedScratchPoolProbeLineParseError::DuplicateField(
                        key.to_owned(),
                    ));
                }
            }
            _ => {
                return Err(PinnedScratchPoolProbeLineParseError::UnknownField(
                    key.to_owned(),
                ));
            }
        }
    }

    let event = event.ok_or(PinnedScratchPoolProbeLineParseError::MissingEvent)?;
    let status_token = status_token.ok_or(PinnedScratchPoolProbeLineParseError::MissingEvent)?;
    let status = PinnedScratchPoolProbeStatus::from_str_token(status_token).ok_or_else(|| {
        PinnedScratchPoolProbeLineParseError::UnknownStatus(status_token.to_owned())
    })?;

    if event == PinnedScratchPoolProbeEvent::Allocation {
        if handle.is_some() {
            return Err(PinnedScratchPoolProbeLineParseError::UnknownField(
                "handle".to_owned(),
            ));
        }
        if status == PinnedScratchPoolProbeStatus::Ok && bytes.is_none() {
            return Err(PinnedScratchPoolProbeLineParseError::MissingField(
                "bytes".to_owned(),
            ));
        }
    } else {
        if bytes.is_some() {
            return Err(PinnedScratchPoolProbeLineParseError::UnknownField(
                "bytes".to_owned(),
            ));
        }
        if status == PinnedScratchPoolProbeStatus::Ok && event.requires_handle() && handle.is_none()
        {
            return Err(PinnedScratchPoolProbeLineParseError::MissingField(
                "handle".to_owned(),
            ));
        }
    }

    Ok(PinnedScratchPoolProbeEventLine {
        event,
        status,
        handle,
        bytes,
    })
}

/// Parses one pinned scratch pool probe stats line.
///
/// # Errors
///
/// Returns an error when the line is not a stats line, contains duplicate or
/// unknown fields, has an unknown phase, or contains malformed numeric values.
pub fn parse_pinned_scratch_pool_probe_stats_line(
    line: &str,
) -> Result<PinnedScratchPoolProbeStatsLine, PinnedScratchPoolProbeLineParseError> {
    let mut tokens = line.split_whitespace();
    match tokens.next() {
        Some("pool_stats") => {}
        Some(token) => {
            return Err(PinnedScratchPoolProbeLineParseError::InvalidToken(
                token.to_owned(),
            ));
        }
        None => {
            return Err(PinnedScratchPoolProbeLineParseError::InvalidToken(
                String::new(),
            ))
        }
    }

    let mut phase = None;
    let mut locked_bytes = None;
    let mut checked_out = None;
    let mut idle = None;
    let mut created_arenas = None;
    let mut reused_arenas = None;
    let mut checkout_count = None;
    let mut release_count = None;

    for token in tokens {
        let Some((key, value)) = token.split_once('=') else {
            return Err(PinnedScratchPoolProbeLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "phase" => {
                let parsed =
                    PinnedScratchPoolProbePhase::from_str_token(value).ok_or_else(|| {
                        PinnedScratchPoolProbeLineParseError::UnknownPhase(value.to_owned())
                    })?;
                set_probe_field(&mut phase, key, parsed)?;
            }
            "locked_bytes" => {
                let parsed = parse_probe_usize_field(key, value)?;
                set_probe_field(&mut locked_bytes, key, parsed)?;
            }
            "checked_out" => {
                let parsed = parse_probe_usize_field(key, value)?;
                set_probe_field(&mut checked_out, key, parsed)?;
            }
            "idle" => {
                let parsed = parse_probe_usize_field(key, value)?;
                set_probe_field(&mut idle, key, parsed)?;
            }
            "created_arenas" => {
                let parsed = parse_probe_u64_field(key, value)?;
                set_probe_field(&mut created_arenas, key, parsed)?;
            }
            "reused_arenas" => {
                let parsed = parse_probe_u64_field(key, value)?;
                set_probe_field(&mut reused_arenas, key, parsed)?;
            }
            "checkout_count" => {
                let parsed = parse_probe_u64_field(key, value)?;
                set_probe_field(&mut checkout_count, key, parsed)?;
            }
            "release_count" => {
                let parsed = parse_probe_u64_field(key, value)?;
                set_probe_field(&mut release_count, key, parsed)?;
            }
            _ => {
                return Err(PinnedScratchPoolProbeLineParseError::UnknownField(
                    key.to_owned(),
                ));
            }
        }
    }

    Ok(PinnedScratchPoolProbeStatsLine {
        phase: require_probe_field(phase, "phase")?,
        locked_bytes: require_probe_field(locked_bytes, "locked_bytes")?,
        checked_out: require_probe_field(checked_out, "checked_out")?,
        idle: require_probe_field(idle, "idle")?,
        created_arenas: require_probe_field(created_arenas, "created_arenas")?,
        reused_arenas: require_probe_field(reused_arenas, "reused_arenas")?,
        checkout_count: require_probe_field(checkout_count, "checkout_count")?,
        release_count: require_probe_field(release_count, "release_count")?,
    })
}

/// Extracts pinned scratch pool probe events and stats from multiline output.
///
/// # Errors
///
/// Returns an error when required stable lines are missing, duplicated, or
/// malformed.
pub fn parse_pinned_scratch_pool_probe_output(
    output: &str,
) -> Result<PinnedScratchPoolProbeOutput, PinnedScratchPoolProbeOutputParseError> {
    let mut initial_stats = None;
    let mut checkout_error_stats = None;
    let mut after_checkout_stats = None;
    let mut after_release_stats = None;
    let mut after_reuse_checkout_stats = None;
    let mut after_reuse_release_stats = None;

    let mut checkout = None;
    let mut allocation = None;
    let mut release = None;
    let mut reuse_checkout = None;
    let mut reuse_release = None;

    for line in output.lines().map(str::trim) {
        if is_pinned_scratch_pool_probe_event_line(line) {
            let parsed = parse_pinned_scratch_pool_probe_event_line(line)
                .map_err(PinnedScratchPoolProbeOutputParseError::EventLine)?;
            match parsed.event {
                PinnedScratchPoolProbeEvent::Checkout => {
                    set_probe_output_event(&mut checkout, parsed)?;
                }
                PinnedScratchPoolProbeEvent::Allocation => {
                    set_probe_output_event(&mut allocation, parsed)?;
                }
                PinnedScratchPoolProbeEvent::Release => {
                    set_probe_output_event(&mut release, parsed)?;
                }
                PinnedScratchPoolProbeEvent::ReuseCheckout => {
                    set_probe_output_event(&mut reuse_checkout, parsed)?;
                }
                PinnedScratchPoolProbeEvent::ReuseRelease => {
                    set_probe_output_event(&mut reuse_release, parsed)?;
                }
            }
            continue;
        }

        if is_pinned_scratch_pool_probe_stats_line(line) {
            let parsed = parse_pinned_scratch_pool_probe_stats_line(line)
                .map_err(PinnedScratchPoolProbeOutputParseError::StatsLine)?;
            match parsed.phase {
                PinnedScratchPoolProbePhase::Initial => {
                    set_probe_output_stats(&mut initial_stats, parsed)?;
                }
                PinnedScratchPoolProbePhase::CheckoutError => {
                    set_probe_output_stats(&mut checkout_error_stats, parsed)?;
                }
                PinnedScratchPoolProbePhase::AfterCheckout => {
                    set_probe_output_stats(&mut after_checkout_stats, parsed)?;
                }
                PinnedScratchPoolProbePhase::AfterRelease => {
                    set_probe_output_stats(&mut after_release_stats, parsed)?;
                }
                PinnedScratchPoolProbePhase::AfterReuseCheckout => {
                    set_probe_output_stats(&mut after_reuse_checkout_stats, parsed)?;
                }
                PinnedScratchPoolProbePhase::AfterReuseRelease => {
                    set_probe_output_stats(&mut after_reuse_release_stats, parsed)?;
                }
            }
        }
    }

    let initial_stats =
        initial_stats.ok_or(PinnedScratchPoolProbeOutputParseError::MissingInitialStats)?;
    let checkout = checkout.ok_or(PinnedScratchPoolProbeOutputParseError::MissingCheckoutLine)?;

    if checkout.status == PinnedScratchPoolProbeStatus::Ok {
        require_probe_output_event(allocation, PinnedScratchPoolProbeEvent::Allocation)?;
        require_probe_output_event(release, PinnedScratchPoolProbeEvent::Release)?;
        require_probe_output_event(reuse_checkout, PinnedScratchPoolProbeEvent::ReuseCheckout)?;
        require_probe_output_event(reuse_release, PinnedScratchPoolProbeEvent::ReuseRelease)?;
        require_probe_output_stats(
            after_checkout_stats,
            PinnedScratchPoolProbePhase::AfterCheckout,
        )?;
        require_probe_output_stats(
            after_release_stats,
            PinnedScratchPoolProbePhase::AfterRelease,
        )?;
        require_probe_output_stats(
            after_reuse_checkout_stats,
            PinnedScratchPoolProbePhase::AfterReuseCheckout,
        )?;
        require_probe_output_stats(
            after_reuse_release_stats,
            PinnedScratchPoolProbePhase::AfterReuseRelease,
        )?;
    }

    Ok(PinnedScratchPoolProbeOutput {
        initial_stats,
        checkout,
        checkout_error_stats,
        allocation,
        release,
        reuse_checkout,
        reuse_release,
        after_checkout_stats,
        after_release_stats,
        after_reuse_checkout_stats,
        after_reuse_release_stats,
    })
}

fn parse_probe_usize_field(
    field: &str,
    value: &str,
) -> Result<usize, PinnedScratchPoolProbeLineParseError> {
    value
        .parse()
        .map_err(|_| PinnedScratchPoolProbeLineParseError::InvalidNumber {
            field: field.to_owned(),
            value: value.to_owned(),
        })
}

fn parse_probe_u64_field(
    field: &str,
    value: &str,
) -> Result<u64, PinnedScratchPoolProbeLineParseError> {
    value
        .parse()
        .map_err(|_| PinnedScratchPoolProbeLineParseError::InvalidNumber {
            field: field.to_owned(),
            value: value.to_owned(),
        })
}

fn set_probe_field<T>(
    slot: &mut Option<T>,
    field: &str,
    value: T,
) -> Result<(), PinnedScratchPoolProbeLineParseError> {
    if slot.replace(value).is_some() {
        return Err(PinnedScratchPoolProbeLineParseError::DuplicateField(
            field.to_owned(),
        ));
    }
    Ok(())
}

fn require_probe_field<T>(
    value: Option<T>,
    field: &str,
) -> Result<T, PinnedScratchPoolProbeLineParseError> {
    value.ok_or_else(|| PinnedScratchPoolProbeLineParseError::MissingField(field.to_owned()))
}

pub(crate) fn is_pinned_scratch_pool_probe_event_line(line: &str) -> bool {
    [
        "pool_checkout=",
        "checked_out_allocation=",
        "pool_release=",
        "pool_reuse_checkout=",
        "pool_reuse_release=",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
}

pub(crate) fn is_pinned_scratch_pool_probe_stats_line(line: &str) -> bool {
    line.split_whitespace().next() == Some("pool_stats")
}

fn set_probe_output_event(
    slot: &mut Option<PinnedScratchPoolProbeEventLine>,
    value: PinnedScratchPoolProbeEventLine,
) -> Result<(), PinnedScratchPoolProbeOutputParseError> {
    if slot.replace(value).is_some() {
        return Err(PinnedScratchPoolProbeOutputParseError::DuplicateEvent(
            value.event,
        ));
    }
    Ok(())
}

fn set_probe_output_stats(
    slot: &mut Option<PinnedScratchPoolProbeStatsLine>,
    value: PinnedScratchPoolProbeStatsLine,
) -> Result<(), PinnedScratchPoolProbeOutputParseError> {
    if slot.replace(value).is_some() {
        return Err(PinnedScratchPoolProbeOutputParseError::DuplicateStats(
            value.phase,
        ));
    }
    Ok(())
}

fn require_probe_output_event(
    value: Option<PinnedScratchPoolProbeEventLine>,
    event: PinnedScratchPoolProbeEvent,
) -> Result<PinnedScratchPoolProbeEventLine, PinnedScratchPoolProbeOutputParseError> {
    value.ok_or(PinnedScratchPoolProbeOutputParseError::MissingEvent(event))
}

fn require_probe_output_stats(
    value: Option<PinnedScratchPoolProbeStatsLine>,
    phase: PinnedScratchPoolProbePhase,
) -> Result<PinnedScratchPoolProbeStatsLine, PinnedScratchPoolProbeOutputParseError> {
    value.ok_or(PinnedScratchPoolProbeOutputParseError::MissingStats(phase))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_pinned_scratch_pool_probe_event_line, parse_pinned_scratch_pool_probe_output,
        parse_pinned_scratch_pool_probe_stats_line, PinnedScratchPoolProbeEvent,
        PinnedScratchPoolProbeEventLine, PinnedScratchPoolProbeLineParseError,
        PinnedScratchPoolProbeOutput, PinnedScratchPoolProbeOutputParseError,
        PinnedScratchPoolProbePhase, PinnedScratchPoolProbeStatsLine, PinnedScratchPoolProbeStatus,
    };

    #[test]
    fn parses_pinned_scratch_pool_probe_event_lines() {
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("pool_checkout=ok handle=0")
                .expect("checkout"),
            PinnedScratchPoolProbeEventLine {
                event: PinnedScratchPoolProbeEvent::Checkout,
                status: PinnedScratchPoolProbeStatus::Ok,
                handle: Some(0),
                bytes: None,
            }
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("checked_out_allocation=ok bytes=256")
                .expect("allocation"),
            PinnedScratchPoolProbeEventLine {
                event: PinnedScratchPoolProbeEvent::Allocation,
                status: PinnedScratchPoolProbeStatus::Ok,
                handle: None,
                bytes: Some(256),
            }
        );
        assert_eq!(PinnedScratchPoolProbeStatus::Ok.to_string(), "ok");
        assert_eq!(
            PinnedScratchPoolProbeEvent::ReuseCheckout.to_string(),
            "pool_reuse_checkout"
        );
    }

    #[test]
    fn parses_pinned_scratch_pool_probe_stats_lines() {
        assert_eq!(
        parse_pinned_scratch_pool_probe_stats_line(
            "pool_stats phase=after_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1"
        )
        .expect("stats"),
        PinnedScratchPoolProbeStatsLine {
            phase: PinnedScratchPoolProbePhase::AfterRelease,
            locked_bytes: 20479,
            checked_out: 0,
            idle: 1,
            created_arenas: 1,
            reused_arenas: 0,
            checkout_count: 1,
            release_count: 1,
        }
    );
        assert_eq!(
            PinnedScratchPoolProbePhase::AfterReuseRelease.to_string(),
            "after_reuse_release"
        );
    }

    #[test]
    fn rejects_invalid_pinned_scratch_pool_probe_lines() {
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("pool_checkout").expect_err("invalid token"),
            PinnedScratchPoolProbeLineParseError::InvalidToken("pool_checkout".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("pool_checkout=maybe handle=0")
                .expect_err("unknown status"),
            PinnedScratchPoolProbeLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("pool_checkout=ok")
                .expect_err("missing handle"),
            PinnedScratchPoolProbeLineParseError::MissingField("handle".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("checked_out_allocation=ok")
                .expect_err("missing bytes"),
            PinnedScratchPoolProbeLineParseError::MissingField("bytes".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line(
                "checked_out_allocation=ok handle=0 bytes=256"
            )
            .expect_err("unexpected handle"),
            PinnedScratchPoolProbeLineParseError::UnknownField("handle".to_owned())
        );
        assert_eq!(
        parse_pinned_scratch_pool_probe_stats_line(
            "pool_stats phase=bad locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0"
        )
        .expect_err("unknown phase"),
        PinnedScratchPoolProbeLineParseError::UnknownPhase("bad".to_owned())
    );
        assert_eq!(
        parse_pinned_scratch_pool_probe_stats_line(
            "pool_stats phase=initial locked_bytes=abc checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0"
        )
        .expect_err("invalid number"),
        PinnedScratchPoolProbeLineParseError::InvalidNumber {
            field: "locked_bytes".to_owned(),
            value: "abc".to_owned(),
        }
    );
    }

    #[test]
    fn parses_pinned_scratch_pool_probe_output() {
        let output = "\
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
checked_out_mapping_start=0xffffbc46e000
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

        let parsed = parse_pinned_scratch_pool_probe_output(output).expect("probe output");
        assert_eq!(parsed.initial_stats.locked_bytes, 0);
        assert_eq!(parsed.checkout.handle, Some(0));
        assert_eq!(parsed.allocation.expect("allocation").bytes, Some(256));
        assert_eq!(parsed.release.expect("release").handle, Some(0));
        assert_eq!(
            parsed.reuse_checkout.expect("reuse checkout").handle,
            Some(1)
        );
        assert_eq!(
            parsed
                .after_reuse_release_stats
                .expect("after reuse release")
                .reused_arenas,
            1
        );
    }

    #[test]
    fn parses_pinned_scratch_pool_checkout_error_output() {
        let output = "\
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=error
pool_checkout_error=pinned scratch pool arena failed
pool_stats phase=checkout_error locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
";

        assert_eq!(
            parse_pinned_scratch_pool_probe_output(output).expect("checkout error output"),
            PinnedScratchPoolProbeOutput {
                initial_stats: PinnedScratchPoolProbeStatsLine {
                    phase: PinnedScratchPoolProbePhase::Initial,
                    locked_bytes: 0,
                    checked_out: 0,
                    idle: 0,
                    created_arenas: 0,
                    reused_arenas: 0,
                    checkout_count: 0,
                    release_count: 0,
                },
                checkout: PinnedScratchPoolProbeEventLine {
                    event: PinnedScratchPoolProbeEvent::Checkout,
                    status: PinnedScratchPoolProbeStatus::Error,
                    handle: None,
                    bytes: None,
                },
                checkout_error_stats: Some(PinnedScratchPoolProbeStatsLine {
                    phase: PinnedScratchPoolProbePhase::CheckoutError,
                    locked_bytes: 0,
                    checked_out: 0,
                    idle: 0,
                    created_arenas: 0,
                    reused_arenas: 0,
                    checkout_count: 0,
                    release_count: 0,
                }),
                allocation: None,
                release: None,
                reuse_checkout: None,
                reuse_release: None,
                after_checkout_stats: None,
                after_release_stats: None,
                after_reuse_checkout_stats: None,
                after_reuse_release_stats: None,
            }
        );
    }

    #[test]
    fn rejects_invalid_pinned_scratch_pool_probe_output() {
        assert_eq!(
        parse_pinned_scratch_pool_probe_output(
            "pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0\n"
        )
        .expect_err("missing checkout"),
        PinnedScratchPoolProbeOutputParseError::MissingCheckoutLine
    );
        assert_eq!(
            parse_pinned_scratch_pool_probe_output("pool_checkout=ok handle=0\n")
                .expect_err("missing initial stats"),
            PinnedScratchPoolProbeOutputParseError::MissingInitialStats
        );
        assert_eq!(
        parse_pinned_scratch_pool_probe_output(
            "pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
pool_release=ok handle=0
pool_reuse_checkout=ok handle=1
pool_reuse_release=ok handle=1
pool_stats phase=after_checkout locked_bytes=1 checked_out=1 idle=0 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=0
pool_stats phase=after_release locked_bytes=1 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1
pool_stats phase=after_reuse_checkout locked_bytes=1 checked_out=1 idle=0 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=1
pool_stats phase=after_reuse_release locked_bytes=1 checked_out=0 idle=1 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=2
"
        )
        .expect_err("missing allocation"),
        PinnedScratchPoolProbeOutputParseError::MissingEvent(
            PinnedScratchPoolProbeEvent::Allocation
        )
    );
        assert_eq!(
        parse_pinned_scratch_pool_probe_output(
            "pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=error
"
        )
        .expect_err("duplicate initial stats"),
        PinnedScratchPoolProbeOutputParseError::DuplicateStats(
            PinnedScratchPoolProbePhase::Initial
        )
    );
        assert_eq!(
        parse_pinned_scratch_pool_probe_output(
            "pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=error
pool_checkout=error
"
        )
        .expect_err("duplicate checkout"),
        PinnedScratchPoolProbeOutputParseError::DuplicateEvent(
            PinnedScratchPoolProbeEvent::Checkout
        )
    );
    }
}
