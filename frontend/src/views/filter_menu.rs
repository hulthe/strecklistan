use crate::generated::css_classes::C;
use seed::prelude::*;
use seed::{fetch::FetchObject, *};

#[derive(Clone, Debug)]
pub enum FilterMenuMsg {}

pub struct FilterMenu {}

impl FilterMenu {
    pub fn new() -> Self {
        FilterMenu {
            // fields
        }
    }

    pub fn update(&mut self, msg: FilterMenuMsg, orders: &mut impl Orders<FilterMenuMsg>) {
        //let mut orders_local = orders.proxy(|msg| Msg::TransactionsMsg(msg));
        match msg {
            // match
        }
    }

    pub fn view(&self) -> Node<FilterMenuMsg> {
        div![]
    }
}
