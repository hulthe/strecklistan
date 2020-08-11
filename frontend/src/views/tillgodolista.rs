use crate::generated::css_classes::C;
use strecklistan_api::book_account::BookAccount;
use strecklistan_api::member::Member;
use seed::prelude::*;
use seed::*;

pub fn view_tillgodo<M: 'static + Clone>(
    account: &BookAccount,
    member: &Member,
    msg: M,
) -> Node<M> {
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
        div![class![C.tillgodo_money], format!("{}:-", account.balance)],
        simple_ev(Ev::Click, msg),
    ]
}
