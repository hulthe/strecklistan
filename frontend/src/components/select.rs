use seed::prelude::*;
use seed::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum SelectInputMsg<T> {
    ChangeStr(String),
    Change(T),
}

type DisplayOpt<T> = Arc<dyn Fn(&T) -> &'static str>;

#[derive(Clone)]
pub struct SelectInput<T> {
    options: Vec<T>,
    display: DisplayOpt<T>,
    selected: T,
    select_styles: &'static [&'static str],
    option_styles: &'static [&'static str],
}

impl<T: 'static + Clone> SelectInput<T> {
    pub fn new(options: Vec<T>, display: impl Fn(&T) -> &'static str + 'static) -> Self {
        assert_ne!(options.len(), 0);
        SelectInput {
            selected: options.first().unwrap().clone(),
            options,
            display: Arc::new(display),
            select_styles: &[],
            option_styles: &[],
        }
    }

    pub fn with_select_styles(self, styles: &'static [&'static str]) -> Self {
        SelectInput {
            select_styles: styles,
            ..self
        }
    }

    #[allow(dead_code)]
    pub fn with_option_styles(self, styles: &'static [&'static str]) -> Self {
        SelectInput {
            option_styles: styles,
            ..self
        }
    }

    pub fn update(&mut self, msg: SelectInputMsg<T>, orders: &mut impl Orders<SelectInputMsg<T>>) {
        match msg {
            SelectInputMsg::ChangeStr(input) => {
                match self.options.iter().find(|opt| input == (self.display)(opt)) {
                    None => todo!(),
                    Some(opt) => {
                        orders.send_msg(SelectInputMsg::Change(opt.clone()));
                    }
                }
            }
            SelectInputMsg::Change(opt) => self.selected = opt,
        }
    }

    pub fn view(&self) -> Node<SelectInputMsg<T>> {
        select![
            C![self.select_styles],
            input_ev(Ev::Change, SelectInputMsg::<T>::ChangeStr),
            self.options.iter().map(&*self.display).map(|s| option![s]),
        ]
    }

    pub fn selected(&self) -> &T {
        &self.selected
    }
}
