use libc::c_int;
use std::ffi::NulError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RsqliteError {
    /// Can not convert the provided String into a CString
    #[error("Can not convert the provided String into a CString - {0}")]
    InvalidCString(#[from] NulError),
    /// SQLITE_ABORT: An operation was aborted prior to completion
    #[error("SQLITE_ABORT: An operation was aborted prior to completion.")]
    Abort,
    /// SQLITE_AUTH: An SQL statement being prepared is not authorized.
    #[error("SQLITE_AUTH: An SQL statement being prepared is not authorized.")]
    Auth,
    /// SQLITE_BUSY: The database file could not be written (or in some cases read).
    #[error("SQLITE_BUSY: The database file could not be written (or in some cases read).")]
    Busy,
    /// SQLITE_CANTOPEN: SQLite was unable to open a file.
    #[error("SQLITE_CANTOPEN: SQLite was unable to open a file.")]
    CantOpen,
    /// SQLITE_CONSTRAINT: An SQL constraint violation occurred.
    #[error("SQLITE_CONSTRAINT: An SQL constraint violation occurred.")]
    Constraint,
    /// SQLITE_CORRUPT: The database file has been corrupted.
    #[error("SQLITE_CORRUPT: The database file has been corrupted.")]
    Corrupt,
    /// SQLITE_ERROR: Sqlite generic error code.
    #[error("SQLITE_ERROR: Sqlite generic error code.")]
    Error,
    /// SQLITE_FULL: A write could not complete because the disk is full.
    #[error("SQLITE_FULL: A write could not complete because the disk is full.")]
    Full,
    /// SQLITE_INTERNAL: An internal malfunction. you should never see this.
    #[error("SQLITE_INTERNAL: An internal malfunction. you should never see this.")]
    Internal,
    /// SQLITE_INTERRUPT: An operation was interrupted.
    #[error("SQLITE_INTERRUPT: An operation was interrupted.")]
    Interrupt,
    /// SQLITE_IOERR: Operation did not finish because the OS reported an I/O error.
    #[error("SQLITE_IOERR: Operation could not finish because the OS reported an I/O error.")]
    IOerr,
    /// SQLITE_LOCKED: A write operation could not continue because of a conflict
    /// within the same database connection or a conflict
    /// with a different database connection that uses a shared cache.
    #[error(
        "SQLITE_LOCKED: A write operation could not continue because of a conflict \
    within the same database connection or a conflict with a different database connection."
    )]
    Locked,
    /// SQLITE_MISMATCH: This error code indicates a datatype mismatch.
    #[error("SQLITE_MISMATCH: This error code indicates a datatype mismatch.")]
    Mismatch,
    /// SQLITE_MISUSE: SQLite interface was used in an undefined or unsupported way.
    #[error("SQLITE_MISUSE: SQLite interface was used in an undefined or unsupported way.")]
    Misuse,
    /// SQLITE_NOLFS: System does not support large files,
    /// or the database grew to be larger than what the filesystem can handle.
    #[error(
        "SQLITE_NOLFS: System does not support large files, \
    or the database grew to be larger than what the filesystem can handle."
    )]
    Nolfs,
    /// SQLITE_NOMEM: SQLite was unable to allocate all the memory \
    /// it needed to complete the operation.
    #[error(
        "SQLITE_NOMEM: SQLite was unable to allocate all the memory 
    it needed to complete the operation."
    )]
    Nomem,
    /// SQLITE_NOTADB: The file being opened does not appear to be an SQLite database file.
    #[error(
        "SQLITE_NOTADB: The file being opened does not appear to be \
    an SQLite database file."
    )]
    Notadb,
    /// SQLITE_NOTFOUND: See https://sqlite.org/rescode.html
    #[error("SQLITE_NOTFOUND: See https://sqlite.org/rescode.html")]
    Notfound,
    /// SQLITE_PERM: The requested access mode for a newly created database could not
    /// be provided.
    #[error(
        "SQLITE_PERM: The requested access mode for a \
    newly created database could not be provided."
    )]
    Perm,
    /// SQLITE_PROTOCOL: A problem with the file locking protocol used by SQLite.
    #[error("SQLITE_PROTOCOL: A problem with the file locking protocol used by SQLite.")]
    Protocol,
    /// SQLITE_RANGE: The parameter number argument to one of the sqlite3_bind routines
    /// or the column number in one of the sqlite3_column routines is out of range.
    #[error(
        "SQLITE_RANGE: The parameter number argument to one of the sqlite3_bind \
    routines or the column number in one of the sqlite3_column routines is out of range."
    )]
    Range,
    /// SQLITE_READONLY: An attempt was made to alter some data for which the current
    /// database connection does not have write permission.
    #[error(
        "SQLITE_READONLY: An attempt was made to alter some data for which the \
    current database connection does not have write permission."
    )]
    Readonly,
    /// SQLITE_SCHEMA: The database schema was changed by some other process in between
    /// the time that the statement was prepared and the time the statement was run.
    #[error(
        "SQLITE_SCHEMA: The database schema was changed by some other process in \
    between the time that the statement was prepared and the time the statement was run."
    )]
    Schema,
    /// SQLITE_TOOBIG: A string or BLOB was too large.
    #[error("SQLITE_TOOBIG: A string or BLOB was too large.")]
    Toobig,
    /// Unknown SQLITE error, See https://sqlite.org/rescode.html
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
