use std::fmt;

/// Stable status for mapped scratch page-lock probe lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLockProbeStatus {
    /// The operation succeeded.
    Ok,
    /// The operation failed.
    Error,
}

impl PageLockProbeStatus {
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

impl fmt::Display for PageLockProbeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable page-lock probe field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLockProbeField {
    /// `page_lock=<status>`.
    Lock,
    /// `page_unlock=<status>`.
    Unlock,
}

impl PageLockProbeField {
    /// Returns the stable machine-readable field name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lock => "page_lock",
            Self::Unlock => "page_unlock",
        }
    }

    /// Parses a stable machine-readable field name.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "page_lock" => Some(Self::Lock),
            "page_unlock" => Some(Self::Unlock),
            _ => None,
        }
    }
}

impl fmt::Display for PageLockProbeField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed mapped scratch page-lock status line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageLockProbeStatusLine {
    /// Parsed status field.
    pub field: PageLockProbeField,
    /// Parsed status value.
    pub status: PageLockProbeStatus,
}

/// Parsed mapped scratch page-lock probe output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchLockProbeOutput {
    /// Page-lock status.
    pub page_lock: PageLockProbeStatus,
    /// Page-unlock status, present when lock succeeded and unlock was attempted.
    pub page_unlock: Option<PageLockProbeStatus>,
}

/// Error returned when parsing a mapped scratch lock status line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageLockProbeStatusLineParseError {
    /// The line does not contain a supported page-lock status token.
    MissingStatus,
    /// The line contains a duplicate page-lock status token.
    DuplicateStatus,
    /// The line contains a token outside the page-lock status schema.
    InvalidToken(String),
    /// The status field is not recognized.
    UnknownField(String),
    /// The status token is not recognized.
    UnknownStatus(String),
}

/// Error returned when extracting mapped scratch lock statuses from multiline output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchLockProbeOutputParseError {
    /// The output does not contain a `page_lock=` line.
    MissingLockLine,
    /// The output has `page_lock=ok` but no `page_unlock=` line.
    MissingUnlockLine,
    /// The output contains more than one `page_lock=` line.
    DuplicateLockLine,
    /// The output contains more than one `page_unlock=` line.
    DuplicateUnlockLine,
    /// A discovered status line is malformed.
    Line(PageLockProbeStatusLineParseError),
}

impl fmt::Display for PageLockProbeStatusLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStatus => f.write_str("missing page lock status token"),
            Self::DuplicateStatus => f.write_str("duplicate page lock status token"),
            Self::InvalidToken(token) => write!(f, "invalid page lock status token: {token}"),
            Self::UnknownField(field) => write!(f, "unknown page lock status field: {field}"),
            Self::UnknownStatus(status) => write!(f, "unknown page lock status: {status}"),
        }
    }
}

impl std::error::Error for PageLockProbeStatusLineParseError {}

impl fmt::Display for MappedScratchLockProbeOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLockLine => f.write_str("missing page_lock line"),
            Self::MissingUnlockLine => f.write_str("missing page_unlock line after page_lock=ok"),
            Self::DuplicateLockLine => f.write_str("duplicate page_lock line"),
            Self::DuplicateUnlockLine => f.write_str("duplicate page_unlock line"),
            Self::Line(source) => write!(f, "invalid page lock status line: {source}"),
        }
    }
}

impl std::error::Error for MappedScratchLockProbeOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line(source) => Some(source),
            Self::MissingLockLine
            | Self::MissingUnlockLine
            | Self::DuplicateLockLine
            | Self::DuplicateUnlockLine => None,
        }
    }
}

/// Parses one mapped scratch page-lock status line.
///
/// # Errors
///
/// Returns an error when the line is missing a status token, contains duplicate
/// status tokens, contains unsupported tokens, or uses an unknown field or status.
pub fn parse_page_lock_probe_status_line(
    line: &str,
) -> Result<PageLockProbeStatusLine, PageLockProbeStatusLineParseError> {
    let mut field = None;
    let mut status_token = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(PageLockProbeStatusLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        let parsed_field = PageLockProbeField::from_str_token(key)
            .ok_or_else(|| PageLockProbeStatusLineParseError::UnknownField(key.to_owned()))?;
        if field.replace(parsed_field).is_some() {
            return Err(PageLockProbeStatusLineParseError::DuplicateStatus);
        }
        if status_token.replace(value).is_some() {
            return Err(PageLockProbeStatusLineParseError::DuplicateStatus);
        }
    }

    let field = field.ok_or(PageLockProbeStatusLineParseError::MissingStatus)?;
    let status_token = status_token.ok_or(PageLockProbeStatusLineParseError::MissingStatus)?;
    let status = PageLockProbeStatus::from_str_token(status_token)
        .ok_or_else(|| PageLockProbeStatusLineParseError::UnknownStatus(status_token.to_owned()))?;

    Ok(PageLockProbeStatusLine { field, status })
}

/// Extracts mapped scratch page-lock statuses from multiline probe output.
///
/// # Errors
///
/// Returns an error when the output has no `page_lock=` line, has duplicate
/// status lines, has `page_lock=ok` without a `page_unlock=` line, or contains a
/// malformed page-lock status line.
pub fn parse_mapped_scratch_lock_probe_output(
    output: &str,
) -> Result<MappedScratchLockProbeOutput, MappedScratchLockProbeOutputParseError> {
    let mut page_lock = None;
    let mut page_unlock = None;

    for line in output.lines().map(str::trim) {
        if !(line.starts_with("page_lock=") || line.starts_with("page_unlock=")) {
            continue;
        }

        let parsed = parse_page_lock_probe_status_line(line)
            .map_err(MappedScratchLockProbeOutputParseError::Line)?;
        match parsed.field {
            PageLockProbeField::Lock => {
                if page_lock.replace(parsed.status).is_some() {
                    return Err(MappedScratchLockProbeOutputParseError::DuplicateLockLine);
                }
            }
            PageLockProbeField::Unlock => {
                if page_unlock.replace(parsed.status).is_some() {
                    return Err(MappedScratchLockProbeOutputParseError::DuplicateUnlockLine);
                }
            }
        }
    }

    let page_lock = page_lock.ok_or(MappedScratchLockProbeOutputParseError::MissingLockLine)?;
    if page_lock == PageLockProbeStatus::Ok && page_unlock.is_none() {
        return Err(MappedScratchLockProbeOutputParseError::MissingUnlockLine);
    }

    Ok(MappedScratchLockProbeOutput {
        page_lock,
        page_unlock,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        parse_mapped_scratch_lock_probe_output, parse_page_lock_probe_status_line,
        MappedScratchLockProbeOutput, MappedScratchLockProbeOutputParseError, PageLockProbeField,
        PageLockProbeStatus, PageLockProbeStatusLine, PageLockProbeStatusLineParseError,
    };

    #[test]
    fn parses_mapped_scratch_lock_status_lines() {
        assert_eq!(
            parse_page_lock_probe_status_line("page_lock=ok").expect("lock ok"),
            PageLockProbeStatusLine {
                field: PageLockProbeField::Lock,
                status: PageLockProbeStatus::Ok,
            }
        );
        assert_eq!(
            parse_page_lock_probe_status_line("page_unlock=error").expect("unlock error"),
            PageLockProbeStatusLine {
                field: PageLockProbeField::Unlock,
                status: PageLockProbeStatus::Error,
            }
        );
        assert_eq!(PageLockProbeStatus::Ok.to_string(), "ok");
        assert_eq!(PageLockProbeField::Unlock.to_string(), "page_unlock");
    }

    #[test]
    fn rejects_invalid_mapped_scratch_lock_status_lines() {
        assert_eq!(
            parse_page_lock_probe_status_line("page_lock").expect_err("invalid token"),
            PageLockProbeStatusLineParseError::InvalidToken("page_lock".to_owned())
        );
        assert_eq!(
            parse_page_lock_probe_status_line("page_lock=maybe").expect_err("unknown status"),
            PageLockProbeStatusLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_page_lock_probe_status_line("page_lock_error=mlock failed")
                .expect_err("unknown field"),
            PageLockProbeStatusLineParseError::UnknownField("page_lock_error".to_owned())
        );
        assert_eq!(
            parse_page_lock_probe_status_line("page_lock=ok page_unlock=ok")
                .expect_err("duplicate status"),
            PageLockProbeStatusLineParseError::DuplicateStatus
        );
    }

    #[test]
    fn parses_mapped_scratch_lock_probe_output() {
        let output = "\
mapping_start=0xffff8367a000
mapping_len=20479
touched=5
page_lock=ok
page_unlock=ok
";

        assert_eq!(
            parse_mapped_scratch_lock_probe_output(output).expect("probe output"),
            MappedScratchLockProbeOutput {
                page_lock: PageLockProbeStatus::Ok,
                page_unlock: Some(PageLockProbeStatus::Ok),
            }
        );

        let lock_error_output = "\
mapping_start=0xffff8367a000
page_lock=error
page_lock_error=mlock failed: Cannot allocate memory
";

        assert_eq!(
            parse_mapped_scratch_lock_probe_output(lock_error_output).expect("lock error output"),
            MappedScratchLockProbeOutput {
                page_lock: PageLockProbeStatus::Error,
                page_unlock: None,
            }
        );
    }

    #[test]
    fn rejects_invalid_mapped_scratch_lock_probe_output() {
        assert_eq!(
            parse_mapped_scratch_lock_probe_output("page_unlock=ok\n").expect_err("missing lock"),
            MappedScratchLockProbeOutputParseError::MissingLockLine
        );
        assert_eq!(
            parse_mapped_scratch_lock_probe_output("page_lock=ok\n").expect_err("missing unlock"),
            MappedScratchLockProbeOutputParseError::MissingUnlockLine
        );
        assert_eq!(
            parse_mapped_scratch_lock_probe_output("page_lock=error\npage_lock=ok\n")
                .expect_err("duplicate lock"),
            MappedScratchLockProbeOutputParseError::DuplicateLockLine
        );
        assert_eq!(
            parse_mapped_scratch_lock_probe_output(
                "page_lock=ok\npage_unlock=ok\npage_unlock=error\n"
            )
            .expect_err("duplicate unlock"),
            MappedScratchLockProbeOutputParseError::DuplicateUnlockLine
        );
        assert_eq!(
            parse_mapped_scratch_lock_probe_output("page_lock=maybe\n").expect_err("bad lock line"),
            MappedScratchLockProbeOutputParseError::Line(
                PageLockProbeStatusLineParseError::UnknownStatus("maybe".to_owned())
            )
        );
    }
}
