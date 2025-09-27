use redis_protocol::resp3::types::BytesFrame;
use thiserror::Error;

use crate::commands::CommandKind;

/// Errors that can occur while parsing incoming frames
#[derive(Debug, Error)]
pub enum FrameError {
    #[error("ERR invalid frame type")]
    InvalidFrame,

    #[error("ERR missing command in array")]
    MissingCommand,

    #[error("ERR invalid UTF-8 in command: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("ERR unknown command")]
    UnknownCommand,

    #[error("ERR command parse error: {0}")]
    ParseError(#[from] <CommandKind as std::str::FromStr>::Err),
}

/// Errors related to key/value access or expiration
#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("ERR key has expired")]
    KeyExpired,

    #[error("ERR key not found")]
    KeyNotFound,
}

/// Errors related to executing commands with arguments and parameters
#[derive(Debug, Error)]
pub enum CommandExecutionError {
    #[error("ERR wrong number of arguments for '{0}' command")]
    WrongArity(&'static str),

    #[error("ERR invalid expire time")]
    InvalidExpire,

    #[error("ERR missing expire time")]
    MissingExpire,

    #[error("ERR syntax error near '{0}'")]
    SyntaxError(String),

    #[error("ERR invalid parameters for command '{0}'")]
    InvalidParams(&'static str),

    #[error("ERR unknown command")]
    UnknownCommand,
}

impl From<CommandExecutionError> for BytesFrame {
    /// Convert a CommandExecutionError into a RESP3 simple error frame.
    ///
    /// The produced frame is `BytesFrame::SimpleError` whose data is the error's
    /// string representation and whose attributes are `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::errors::CommandExecutionError;
    /// use redis_protocol::resp3::types::BytesFrame;
    ///
    /// let err = CommandExecutionError::InvalidExpire;
    /// let frame: BytesFrame = err.into();
    /// assert!(matches!(frame, BytesFrame::SimpleError { .. }));
    /// if let BytesFrame::SimpleError { attributes, .. } = frame {
    ///     assert_eq!(attributes, None);
    /// } else {
    ///     panic!("expected SimpleError frame");
    /// }
    /// ```
    fn from(err: CommandExecutionError) -> Self {
        BytesFrame::SimpleError { data: err.to_string().into(), attributes: None }
    }
}

impl From<FrameError> for BytesFrame {
    /// Converts a `FrameError` into a `BytesFrame::SimpleError` whose data is the error's
    /// string representation and whose attributes are `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let frame: BytesFrame = FrameError::MissingCommand.into();
    /// if let BytesFrame::SimpleError { data, attributes } = frame {
    ///     assert_eq!(attributes, None);
    ///     assert_eq!(data, b"missing command");
    /// } else {
    ///     panic!("expected SimpleError");
    /// }
    /// ```
    fn from(err: FrameError) -> Self {
        BytesFrame::SimpleError { data: err.to_string().into(), attributes: None }
    }
}

impl From<ExecutionError> for BytesFrame {
    /// Convert an `ExecutionError` into a RESP3 `SimpleError` `BytesFrame`.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::errors::ExecutionError;
    /// use redis_protocol::resp3::types::BytesFrame;
    ///
    /// let err = ExecutionError::KeyNotFound;
    /// let frame: BytesFrame = err.into();
    /// if let BytesFrame::SimpleError { data, attributes } = frame {
    ///     assert!(attributes.is_none());
    ///     let text = String::from_utf8_lossy(&data);
    ///     assert_eq!(text, err.to_string());
    /// } else {
    ///     panic!("expected SimpleError frame");
    /// }
    /// ```
    fn from(err: ExecutionError) -> Self {
        BytesFrame::SimpleError { data: err.to_string().into(), attributes: None }
    }
}
