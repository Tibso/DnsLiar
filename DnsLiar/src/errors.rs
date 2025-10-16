use core::fmt;
use std::{io, result};
use redis::RedisError;
use hickory_proto::error::ProtoError;
use hickory_resolver::error::ResolveError;

pub type DnsLiarResult<T> = result::Result<T, DnsLiarError>;

/// Custom error type
pub enum DnsLiarError {
    InvalidOpCode(u8),
    MessageTypeNotQuery,
    SocketBinding,
    SocketFilters,
    NoQueryInRequest,
    MispTaskFailed(String),

    Redis(RedisError),
    IO(io::Error),
    Resolver(ResolveError),
    // SystemTime(SystemTimeError),
    Proto(ProtoError)
}

impl fmt::Display for DnsLiarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DnsLiarError::InvalidOpCode(code) => write!(f, "Opcode received '{code}' was not query (0)"),
            DnsLiarError::MessageTypeNotQuery => write!(f, "Message type received was not query"),
            DnsLiarError::SocketBinding => write!(f, "Failed to bind any socket"),
            DnsLiarError::SocketFilters => write!(f, "Failed to find filters for a socket"),
            DnsLiarError::NoQueryInRequest => write!(f, "No query found in request"),
            DnsLiarError::MispTaskFailed(e) => write!(f, "Misp task failed: {e}"),
            DnsLiarError::Redis(e) => write!(f, "A Redis error occured: {e}"),
            DnsLiarError::IO(e) => write!(f, "An IO error occured: {e}"),
            DnsLiarError::Resolver(e) => write!(f, "A Resolver error occured: {e}"),
            DnsLiarError::Proto(e) => write!(f, "A Proto error occured: {e}")
        }
    }
}

impl From<RedisError> for DnsLiarError {
    fn from(e: RedisError) -> Self {
        DnsLiarError::Redis(e)
    }
}
impl From<ResolveError> for DnsLiarError {
    fn from(e: ResolveError) -> Self {
        DnsLiarError::Resolver(e)
    }
}
impl From<ProtoError> for DnsLiarError {
    fn from(e: ProtoError) -> Self {
        DnsLiarError::Proto(e)
    }
}
impl From<io::Error> for DnsLiarError {
    fn from(e: io::Error) -> Self {
        DnsLiarError::IO(e)
    }
}
