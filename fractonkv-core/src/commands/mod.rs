pub(crate) mod dispatcher;

mod get;
mod set;

use crate::errors::FrameError;

use fractonkv_macros::generate_command_kind;
use redis_protocol::resp3::types::BytesFrame;

#[generate_command_kind]
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
