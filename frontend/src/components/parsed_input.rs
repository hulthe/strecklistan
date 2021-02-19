use crate::generated::css_classes::C;
use crate::util::simple_ev;
use seed::prelude::*;
use seed::{attrs, div, empty, input, Attrs, C};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct ParsedInput<T> {
    text: String,
    parsed: Option<T>,
    input_kind: &'static str,
    error_message: Option<&'static str>,
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
    pub fn new<S: ToString>(text: S) -> Self {
        let text = text.to_string();
        ParsedInput {
            parsed: text.parse().ok(),
            text,
            input_kind: "text",
            error_message: None,
        }
    }

    pub fn with_error_message(self, error_message: &'static str) -> Self {
        ParsedInput {
            error_message: Some(error_message),
            ..self
        }
    }

    pub fn with_input_kind(self, input_kind: &'static str) -> Self {
        ParsedInput { input_kind, ..self }
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
            C![C.parsed_input],
            input![
                attrs,
                C![C.parsed_input_text],
                attrs! { At::Value => &self.text },
                attrs! { At::Type => self.input_kind },
                input_ev(Ev::Input, ParsedInputMsg::Input),
                simple_ev(Ev::Custom("focusin".into()), ParsedInputMsg::FocusIn),
                simple_ev(Ev::Custom("focusout".into()), ParsedInputMsg::FocusOut),
            ],
            match (&self.parsed, self.error_message) {
                (None, Some(msg)) => div![C![C.parsed_input_error], msg],
                _ => empty![],
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
