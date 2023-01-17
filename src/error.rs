use std::io::Error as IoError;

use rlua::Error as LuaError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    IoError(#[from] IoError),

    #[error("Lua Error: {0}")]
    LuaError(#[from] LuaError),

    #[error("{0}")]
    Custom(String),
}

impl Error {
    pub fn custom<T: ToString>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}
