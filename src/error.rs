use libc::c_int;
use std::ffi::NulError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RsqliteError {
    #[error("sqlite error code `{0}` - see https://www.sqlite.org/rescode.html")]
    Sqlite(c_int),

    #[error(transparent)]
    InvalidCString(#[from] NulError),
}

impl From<c_int> for RsqliteError {
    fn from(ecode: c_int) -> Self {
        RsqliteError::Sqlite(ecode)
    }
}
