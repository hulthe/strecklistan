use crate::generated::css_classes::C;
use seed::prelude::*;
use seed::*;
use strecklistan_api::book_account::BookAccount;
use strecklistan_api::member::Member;

pub fn view_tillgodo<M: 'static + Clone>(
    account: &BookAccount,
    member: &Member,
    msg: M,
) -> Node<M> {
    let tillgodo_money_class;
    if account.balance < 0.into() {
        tillgodo_money_class = class![C.tillgodo_money, C.tillgodo_money_angry];
    } else {
        tillgodo_money_class = class![C.tillgodo_money];
    }

    div![
        class![C.tillgodo_entry],
        div![
            class![C.tillgodo_nick],
            member.nickname.as_ref().map(|s| s.as_str()).unwrap_or(""),
        ],
        div![
            class![C.tillgodo_name],
            member.first_name.clone(),
            " ",
            member.last_name.clone(),
        ],
        div![tillgodo_money_class, format!("{}:-", account.balance)],
        simple_ev(Ev::Click, msg),
    ]
}
