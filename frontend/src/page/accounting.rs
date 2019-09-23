use crate::app::{Msg, StateReady};
//use crate::generated::css_classes::C;
use chrono::{NaiveDate, NaiveTime};
use laggit_api::currency::Currency;
use seed::{prelude::*, *};
use std::collections::HashMap;

#[derive(Clone)]
pub enum AccountingMsg {
    SetEndDate(String),
    SetEndTime(String),
    UpdateFilter,
}

#[derive(Clone)]
pub struct AccountingPage {
    end_date: Option<NaiveDate>,
    end_time: Option<NaiveTime>,
    accounts_balance: HashMap<i32, Currency>,
}

const DATE_FMT: &'static str = "%Y-%m-%d";
const TIME_FMT: &'static str = "%H:%M";

impl AccountingPage {
    pub fn new(global: &StateReady) -> Self {
        let mut s = AccountingPage {
            end_date: None,
            end_time: None,
            accounts_balance: HashMap::new(),
        };
        s.update_filter(global);
        s
    }

    pub fn update(
        &mut self,
        msg: AccountingMsg,
        global: &mut StateReady,
        orders: &mut impl Orders<Msg>,
    ) {
        let mut orders_local = orders.proxy(|msg| Msg::AccountingMsg(msg));
        match msg {
            AccountingMsg::SetEndDate(input) => {
                self.end_date = NaiveDate::parse_from_str(&input, DATE_FMT).ok();
                orders_local.send_msg(AccountingMsg::UpdateFilter);
            }
            AccountingMsg::SetEndTime(input) => {
                self.end_time = NaiveTime::parse_from_str(&input, TIME_FMT).ok();
                orders_local.send_msg(AccountingMsg::UpdateFilter);
            }
            AccountingMsg::UpdateFilter => self.update_filter(global),
        }
    }

    pub fn view(&self, global: &StateReady) -> Node<Msg> {
        let show_acc = |id| {
            div![
                span![global
                    .book_accounts
                    .get(id)
                    .map(|acc| acc.name.as_str())
                    .unwrap_or("[missing]")],
                span![": "],
                span![format!(
                    "{}:-",
                    self.accounts_balance
                        .get(id)
                        .map(|&c| c)
                        .unwrap_or(0.into())
                )],
            ]
        };
        div![
            class!["accounting_page"],
            div![
                input![
                    attrs! {At::Type => "date"},
                    attrs! {At::Value => self.end_date.as_ref()
                    .map(|d| d.format(DATE_FMT).to_string())
                    .unwrap_or(String::new())},
                    input_ev(Ev::Input, |input| AccountingMsg::SetEndDate(input)),
                ],
                input![
                    attrs! {At::Type => "time"},
                    attrs! {At::Value => self.end_time.as_ref()
                    .map(|d| d.format(TIME_FMT).to_string())
                    .unwrap_or(String::new())},
                    input_ev(Ev::Input, |input| AccountingMsg::SetEndTime(input)),
                ],
            ],
            div![
                show_acc(&global.master_accounts.bank_account_id),
                show_acc(&global.master_accounts.cash_account_id),
                show_acc(&global.master_accounts.sales_account_id),
                show_acc(&global.master_accounts.purchases_account_id),
                div![
                    span!["Tillgodo Totalt"],
                    span![": "],
                    span![format!(
                        "{}:-",
                        self.accounts_balance
                            .iter()
                            .filter_map(|(id, balance)| global
                                .book_accounts
                                .get(id)
                                .map(|acc| (acc, balance)))
                            .filter(|(acc, _)| acc.creditor.is_some())
                            .map(|(_, balance)| *balance)
                            .fold(0.into(), |a: Currency, b| a + b)
                    )],
                ],
            ],
        ]
        .map_message(|msg| Msg::AccountingMsg(msg))
    }

    fn update_filter(&mut self, global: &StateReady) {
        self.accounts_balance.clear();

        let end_date = self.end_date;
        let end_time = self.end_time;

        for tr in global
            .transaction_history
            .iter()
            .filter(|tr| match (end_date, end_time) {
                (Some(ed), Some(et)) => tr.time <= ed.and_time(et),
                (Some(ed), None) => tr.time <= ed.and_hms(23, 59, 59),
                (None, _) => true,
            })
        {
            if let Some(acc) = global.book_accounts.get(&tr.debited_account) {
                *self.accounts_balance.entry(tr.debited_account).or_default() +=
                    acc.debit_diff(tr.amount);
            }
            if let Some(acc) = global.book_accounts.get(&tr.credited_account) {
                *self
                    .accounts_balance
                    .entry(tr.credited_account)
                    .or_default() += acc.credit_diff(tr.amount);
            }
        }
    }
}
