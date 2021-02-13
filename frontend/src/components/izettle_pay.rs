use crate::app::StateReady;
use crate::strings;
use seed::app::cmds::timeout;
use seed::prelude::*;
use seed::*;
use strecklistan_api::{
    izettle::IZettlePayment,
    transaction::{NewTransaction, TransactionId},
};

const POLL_TIMEOUT_MS: u32 = 1000;

/// Helper component for handling iZettle payments
#[derive(Clone)]
pub struct IZettlePay {
    pending: Option<i32>,
}

#[derive(Clone, Debug)]
pub enum IZettlePayMsg {
    /// Poll for payment completion
    PollPendingPayment(i32),

    /// There was an error processing the payment
    Error(IZettlePayErr),

    /// The payment was completed and the transaction committed
    PaymentCompleted { transaction_id: TransactionId },

    /// The payment was intentionally cancelled
    PaymentCancelled,
}

#[derive(Clone, Debug)]
pub enum IZettlePayErr {
    /// No transaction existed with the given ID
    NoTransaction { reference: i32 },

    /// The payment failed for some reason
    PaymentFailed { reference: i32, reason: String },

    /// A network request has failed
    NetworkError { reason: String },
}

impl IZettlePay {
    pub fn new(_global: &StateReady) -> Self {
        IZettlePay { pending: None }
    }

    pub fn pay(&mut self, transaction: NewTransaction, mut orders: impl Orders<IZettlePayMsg>) {
        if self.pending.is_some() {
            return;
        }

        orders.perform_cmd(async move {
            let result = async {
                Request::new("/api/izettle/client/transaction")
                    .method(Method::Post)
                    .json(&transaction)?
                    .fetch()
                    .await?
                    .json()
                    .await
            }
            .await;
            match result {
                Ok(reference) => Some(IZettlePayMsg::PollPendingPayment(reference)),
                Err(e) => {
                    error!("Failed to post transaction", e);
                    Some(IZettlePayMsg::Error(IZettlePayErr::NetworkError {
                        reason: strings::POSTING_TRANSACTION_FAILED.to_string(),
                    }))
                }
            }
        });
    }

    pub fn pending(&self) -> Option<i32> {
        self.pending
    }

    pub fn update(
        &mut self,
        msg: IZettlePayMsg,
        _global: &mut StateReady,
        mut orders: impl Orders<IZettlePayMsg>,
    ) {
        match msg {
            IZettlePayMsg::PaymentCancelled | IZettlePayMsg::PaymentCompleted { .. } => {
                self.pending = None
            }
            IZettlePayMsg::Error(error) => {
                self.pending = None;
                match error {
                    IZettlePayErr::PaymentFailed { reference, reason } => {
                        error!("iZettle payment {} failed: {}", reference, reason);
                    }
                    IZettlePayErr::NoTransaction { reference } => {
                        error!("iZettle payment {} does not exist", reference);
                    }
                    IZettlePayErr::NetworkError { .. } => {}
                }
            }
            IZettlePayMsg::PollPendingPayment(reference) => {
                self.pending = Some(reference);

                orders.perform_cmd(async move {
                    let result = async {
                        Request::new(&format!("/api/izettle/client/poll/{}", reference))
                            .method(Method::Get)
                            .fetch()
                            .await?
                            .json()
                            .await
                    }
                    .await;
                    match result {
                        Ok(IZettlePayment::Pending) => {
                            timeout(POLL_TIMEOUT_MS, || ()).await;
                            Some(IZettlePayMsg::PollPendingPayment(reference))
                        }
                        Ok(IZettlePayment::Paid { transaction_id }) => {
                            Some(IZettlePayMsg::PaymentCompleted { transaction_id })
                        }
                        Ok(IZettlePayment::Canceled) => Some(IZettlePayMsg::PaymentCancelled),
                        Ok(IZettlePayment::NoTransaction) => {
                            Some(IZettlePayMsg::Error(IZettlePayErr::NoTransaction {
                                reference,
                            }))
                        }
                        Ok(IZettlePayment::Failed { reason }) => {
                            Some(IZettlePayMsg::Error(IZettlePayErr::PaymentFailed {
                                reference,
                                reason,
                            }))
                        }
                        Err(e) => {
                            error!("Failed to poll for payment", e);
                            Some(IZettlePayMsg::Error(IZettlePayErr::NetworkError {
                                reason: strings::POLLING_TRANSACTION_FAILED.to_string(),
                            }))
                        }
                    }
                });
            }
        }
    }
}
