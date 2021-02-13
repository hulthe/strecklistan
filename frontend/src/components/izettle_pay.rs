use crate::app::StateReady;
use seed::app::cmds::timeout;
use seed::prelude::*;
use seed::*;
use strecklistan_api::{
    izettle::IZettlePayment,
    transaction::{NewTransaction, TransactionId},
};

const POLL_TIMEOUT_MS: u32 = 1000;

#[derive(Clone, Debug)]
pub enum IZettlePayMsg {
    /// Poll for payment completion
    PollPendingPayment(i32),

    /// There was an error processing the payment
    PaymentError {
        reference: i32,
        error: IZettlePayErr,
    },

    /// The payment was completed and the transaction committed
    PaymentCompleted { transaction_id: TransactionId },

    /// The payment was intentionally cancelled
    PaymentCancelled,
}

#[derive(Clone, Debug)]
pub enum IZettlePayErr {
    /// No transaction existed with the given ID.
    NoTransaction,

    /// The payment failed for some reason.
    PaymentFailed { reason: String },
}

#[derive(Clone)]
pub struct IZettlePay;

impl IZettlePay {
    pub fn new(_global: &StateReady) -> Self {
        IZettlePay
    }

    pub fn pay(&mut self, transaction: NewTransaction, mut orders: impl Orders<IZettlePayMsg>) {
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
                    error!("Failed to post purchase", e);
                    None
                }
            }
        });
    }

    pub fn update(
        &mut self,
        msg: IZettlePayMsg,
        _global: &mut StateReady,
        mut orders: impl Orders<IZettlePayMsg>,
    ) {
        match msg {
            IZettlePayMsg::PaymentCancelled => {}
            IZettlePayMsg::PaymentCompleted { .. } => {}
            IZettlePayMsg::PaymentError { reference, error } => match error {
                IZettlePayErr::PaymentFailed { reason } => {
                    error!("iZettle payment {} failed: {}", reference, reason);
                }
                IZettlePayErr::NoTransaction => {
                    error!("iZettle payment {} does not exist", reference);
                }
            },
            IZettlePayMsg::PollPendingPayment(reference) => {
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
                        Ok(IZettlePayment::NoTransaction) => Some(IZettlePayMsg::PaymentError {
                            reference,
                            error: IZettlePayErr::NoTransaction,
                        }),
                        Ok(IZettlePayment::Failed { reason }) => {
                            Some(IZettlePayMsg::PaymentError {
                                reference,
                                error: IZettlePayErr::PaymentFailed { reason },
                            })
                        }
                        Err(e) => {
                            error!("Failed to poll for payment", e);
                            None
                        }
                    }
                });
            }
        }
    }
}
