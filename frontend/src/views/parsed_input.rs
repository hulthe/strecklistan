use crate::generated::css_classes::C;
use seed::prelude::*;
use seed::{attrs, class, div, empty, input, Attrs};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct ParsedInput<T> {
    text: String,
    parsed: Option<T>,
    input_kind: &'static str,
    error_msg: &'static str,
}

#[derive(Clone, Debug)]
pub enum ParsedInputMsg {
    Input(String),
    FocusIn,
    FocusOut,
}

impl<T> ParsedInput<T>
where
    T: FromStr + ToString,
{
    pub fn new<S: ToString>(text: S, input_kind: &'static str, error_msg: &'static str) -> Self {
        let text = text.to_string();
        ParsedInput {
            parsed: text.parse().ok(),
            text,
            input_kind,
            error_msg,
        }
    }

    pub fn update(&mut self, msg: ParsedInputMsg) {
        match msg {
            ParsedInputMsg::Input(text) => {
                self.parsed = text.parse().ok();
                self.text = text;
            }
            ParsedInputMsg::FocusIn | ParsedInputMsg::FocusOut => {}
        }
    }

    pub fn view(&self, attrs: Attrs) -> Node<ParsedInputMsg> {
        div![
            class![C.parsed_input],
            input![
                attrs,
                class![C.parsed_input_text],
                attrs! { At::Value => &self.text },
                attrs! { At::Type => self.input_kind },
                input_ev(Ev::Input, ParsedInputMsg::Input),
                simple_ev(Ev::Custom("focusin".into()), ParsedInputMsg::FocusIn),
                simple_ev(Ev::Custom("focusout".into()), ParsedInputMsg::FocusOut),
            ],
            match self.parsed {
                Some(_) => empty![],
                None => div![class![C.parsed_input_error], self.error_msg],
            }
        ]
    }

    pub fn set_value(&mut self, value: T) {
        self.text = value.to_string();
        self.parsed = Some(value);
    }

    pub fn get_value(&self) -> Option<&T> {
        self.parsed.as_ref()
    }
}
