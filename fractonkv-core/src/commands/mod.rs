pub(crate) mod get;
pub(crate) mod set;

use fractonkv_macros::commands;
use redis_protocol::resp3::types::BytesFrame;

use crate::errors::FrameError;

#[commands]
pub enum CommandKind {}

impl CommandKind {
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
