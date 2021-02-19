use crate::transaction::TransactionId;

#[cfg(feature = "serde_impl")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum IZettlePayment {
    /// The transaction has been paid
    Paid {
        /// The ID of the completed transaction
        transaction_id: TransactionId,
    },

    /// The transaction is still awaiting payment
    Pending,

    /// The payment was intentionally aborted
    Cancelled,

    /// The payment failed for some reason
    Failed { reason: String },

    /// No pending payment exists for the given ID
    NoTransaction,
}
