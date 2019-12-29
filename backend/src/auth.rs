use std::fmt::{Display, Formatter, Result as FmtResult};

pub enum AuthLevel {
    Read,
    Write,
}

pub enum EventAuthScope {
    /// R/W auth for the list of events
    List(AuthLevel),

    /// Read auth for event signups
    SignupRead,
}

pub enum AuthScope {
    /// Auth for events
    Events(EventAuthScope),

    /// Write access for the inventory manifest
    InventoryWrite,

    /// R/W access for the transaction list
    Transactions(AuthLevel),
}

impl AuthScope {
    pub fn from_str(s: &str) -> Result<Self, ()> {
        use self::AuthLevel::*;
        use self::AuthScope::*;
        use self::EventAuthScope::*;
        match s {
            "/events/list/read" => Ok(Events(List(Read))),
            "/events/list/write" => Ok(Events(List(Write))),
            "/events/signup/read" => Ok(Events(SignupRead)),
            "/inventory/write" => Ok(InventoryWrite),
            "/transactions/read" => Ok(Transactions(Read)),
            "/transactions/write" => Ok(Transactions(Write)),
            _ => Err(()),
        }
    }

    pub fn to_str(&self) -> &'static str {
        use self::AuthLevel::*;
        use self::AuthScope::*;
        use self::EventAuthScope::*;
        match self {
            Events(List(Read)) => "/events/list/read",
            Events(List(Write)) => "/events/list/write",
            Events(SignupRead) => "/events/signup/read",
            InventoryWrite => "/inventory/write",
            Transactions(Read) => "/transactions/read",
            Transactions(Write) => "/transactions/write",
        }
    }
}

impl Display for AuthScope {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.to_str())
    }
}

pub use self::AuthLevel::*;
pub use self::AuthScope::*;
pub use self::EventAuthScope::*;
