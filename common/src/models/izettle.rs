#[cfg(feature = "serde_impl")]
use serde_derive::{Deserialize, Serialize};

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct IZettleErrorResponse {
    pub message: String,
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ClientPollResult {
    Paid,
    NotPaid,
    Canceled,
    Failed(IZettleErrorResponse),
    NoTransaction(IZettleErrorResponse),
}
