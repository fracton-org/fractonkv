pub(crate) mod get;
pub(crate) mod set;

use fractonkv_macros::commands;
use redis_protocol::resp3::types::BytesFrame;

use crate::errors::FrameError;

#[commands]
pub enum CommandKind {}

impl CommandKind {
    /// Convert a RESP3 `BytesFrame` into a typed `CommandKind`.
    ///
    /// Expects an Array frame whose first element is a command as a BlobString or SimpleString;
    /// the command bytes are interpreted as UTF-8, converted to ASCII uppercase, and parsed into
    /// `CommandKind`.
    ///
    /// # Errors
    ///
    /// Returns a `FrameError` when:
    /// - the top-level frame is not an Array (`InvalidFrame`),
    /// - the array has no first element (`MissingCommand`),
    /// - the first element is not a BlobString or SimpleString (`InvalidFrame`),
    /// - the command bytes are not valid UTF-8 (converted to a `FrameError`), or
    /// - the uppercased command string does not parse into a `CommandKind` (`UnknownCommand`).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use redis_protocol::resp3::types::BytesFrame;
    /// // construct an array frame where the first element is the command "GET"
    /// let frame = BytesFrame::Array { data: vec![BytesFrame::SimpleString { data: b"GET".to_vec(), ..Default::default() }], attributes: Default::default() };
    /// let cmd = CommandKind::from_frame(&frame).unwrap();
    /// ```
    pub fn from_frame(frame: &BytesFrame) -> Result<CommandKind, FrameError> {
        let cmd_bytes = match frame {
            BytesFrame::Array { data, .. } => data.first().ok_or(FrameError::MissingCommand)?,
            _ => return Err(FrameError::InvalidFrame),
        };

        let cmd_str = match cmd_bytes {
            BytesFrame::BlobString { data, .. } | BytesFrame::SimpleString { data, .. } => {
                std::str::from_utf8(data)?.to_ascii_uppercase()
            }
            _ => return Err(FrameError::InvalidFrame),
        };

        cmd_str.parse::<CommandKind>().map_err(|_| FrameError::UnknownCommand)
    }
}
