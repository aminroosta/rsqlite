use libc::c_int;
use std::ffi::NulError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RsqliteError {
    #[error("Can not convert the provided String into a CString - {0}")]
    InvalidCString(#[from] NulError),
    #[error("SQLITE_ABORT: An operation was aborted prior to completion, usually be application request. See also: SQLITE_INTERRUPT.")]
    Abort,
    #[error("SQLITE_AUTH: An SQL statement being prepared is not authorized.")]
    Auth,
    #[error("SQLITE_BUSY: The database file could not be written (or in some cases read).")]
    Busy,
    #[error("SQLITE_CANTOPEN: SQLite was unable to open a file. The file in question might be a primary database file or one of several temporary disk files.")]
    CantOpen,
    #[error("SQLITE_CONSTRAINT: An SQL constraint violation occurred while trying to process an SQL statement.")]
    Constraint,
    #[error("SQLITE_CORRUPT: The database file has been corrupted.")]
    Corrupt,
    #[error("SQLITE_ERROR: Sqlite generic error code, returned when no other more specific error code is available.")]
    Error,
    #[error("SQLITE_FULL: A write could not complete because the disk is full.")]
    Full,
    #[error("SQLITE_INTERNAL: An internal malfunction. In a working version of SQLite, an application should never see this result code.")]
    Internal,
    #[error(
        "SQLITE_INTERRUPT: An operation was interrupted by the sqlite3_interrupt() interface."
    )]
    Interrupt,
    #[error("SQLITE_IOERR: the operation could not finish because the operating system reported an I/O error.")]
    IOerr,
    #[error("SQLITE_LOCKED: A write operation could not continue because of a conflict within the same database connection or a conflict with a different database connection that uses a shared cache.")]
    Locked,
    #[error("SQLITE_MISMATCH: This error code indicates a datatype mismatch.")]
    Mismatch,
    #[error("SQLITE_MISUSE: The application used some SQLite interface in a way that is undefined or unsupported.")]
    Misuse,
    #[error("SQLITE_NOLFS: System does not support large files, or the database grew to be larger than what the filesystem can handle.")]
    Nolfs,
    #[error("SQLITE_NOMEM: SQLite was unable to allocate all the memory it needed to complete the operation.")]
    Nomem,
    #[error("SQLITE_NOTADB: The file being opened does not appear to be an SQLite database file.")]
    Notadb,
    #[error("SQLITE_NOTFOUND: See https://sqlite.org/rescode.html")]
    Notfound,
    #[error("SQLITE_PERM: The requested access mode for a newly created database could not be provided.")]
    Perm,
    #[error("SQLITE_PROTOCOL: A problem with the file locking protocol used by SQLite.")]
    Protocol,
    #[error("SQLITE_RANGE: The parameter number argument to one of the sqlite3_bind routines or the column number in one of the sqlite3_column routines is out of range.")]
    Range,
    #[error("SQLITE_READONLY: An attempt was made to alter some data for which the current database connection does not have write permission.")]
    Readonly,
    #[error("SQLITE_SCHEMA: The database schema was changed by some other process in between the time that the statement was prepared and the time the statement was run.")]
    Schema,
    #[error("SQLITE_TOOBIG: A string or BLOB was too large.")]
    Toobig,
    #[error("Unknown SQLITE error({0}), See https://sqlite.org/rescode.html")]
    Unknown(c_int),
}

impl From<c_int> for RsqliteError {
    fn from(ecode: c_int) -> Self {
        use RsqliteError::*;
        let primary_error = (ecode & 255) as u8;
        match primary_error {
            4 => Abort,
            23 => Auth,
            5 => Busy,
            14 => CantOpen,
            19 => Constraint,
            11 => Corrupt,
            1 => Error,
            13 => Full,
            2 => Internal,
            9 => Interrupt,
            10 => IOerr,
            6 => Locked,
            20 => Mismatch,
            21 => Misuse,
            22 => Nolfs,
            7 => Nomem,
            26 => Notadb,
            12 => Notfound,
            3 => Perm,
            15 => Protocol,
            25 => Range,
            8 => Readonly,
            17 => Schema,
            18 => Toobig,
            _ => Unknown(ecode),
        }
    }
}
