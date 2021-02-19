use crate::generated::css_classes::C;
use crate::util::{simple_ev, CompareToStr};
use seed::prelude::*;
use seed::*;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub enum FilterMenuMsg {
    AddFilter,
    SetField { filter_i: usize, field_i: usize },
    SetOp { filter_i: usize, op: FilterOp },
    SetValue { filter_i: usize, value: String },
    DeleteFilter { filter_i: usize },
}

#[derive(Clone, Copy, Debug)]
pub enum FilterOp {
    Equals,
    NotEquals,
    GrTh,
    GrEq,
    LeTh,
    LeEq,
    //RxMatch, // TODO: Regex matching
}

#[derive(Clone)]
pub struct FilterMenu {
    fields: Vec<&'static str>,
    filters: Vec<(usize, FilterOp, String)>,
}

impl FilterMenu {
    pub fn new(fields: Vec<&'static str>) -> Self {
        assert_ne!(fields.len(), 0);
        FilterMenu {
            fields,
            filters: vec![],
        }
    }

    pub fn filter(&self, values: &[&dyn CompareToStr]) -> bool {
        self.filters.iter().all(|(field_i, op, value)| {
            let ord = values[*field_i].cmp_to_str(value);

            match (op, ord) {
                (FilterOp::GrTh, Ordering::Greater)            // >  true if greater
                | (FilterOp::GrEq, Ordering::Greater)          // >=     ... greater
                | (FilterOp::GrEq, Ordering::Equal)            // >=     ... equals
                | (FilterOp::LeTh, Ordering::Less)             // <      ... less
                | (FilterOp::LeEq, Ordering::Less)             // <=     ... less
                | (FilterOp::LeEq, Ordering::Equal)            // <=     ... equals
                | (FilterOp::NotEquals, Ordering::Greater)     // !=     ... greater
                | (FilterOp::NotEquals, Ordering::Less)        // !=     ... less
                | (FilterOp::Equals, Ordering::Equal) => true, // ==     ... equals
                _ => false,                                    // otherwise false
            }
        })
    }

    pub fn update(&mut self, msg: FilterMenuMsg, _orders: &mut impl Orders<FilterMenuMsg>) {
        match msg {
            FilterMenuMsg::AddFilter => self.filters.push((0, FilterOp::NotEquals, String::new())),
            FilterMenuMsg::SetField { filter_i, field_i } => self.filters[filter_i].0 = field_i,
            FilterMenuMsg::SetOp { filter_i, op } => self.filters[filter_i].1 = op,
            FilterMenuMsg::SetValue { filter_i, value } => self.filters[filter_i].2 = value,
            FilterMenuMsg::DeleteFilter { filter_i } => {
                self.filters.remove(filter_i);
            }
        }
    }

    pub fn view(&self) -> Node<FilterMenuMsg> {
        div![
            button![
                C![C.wide_button],
                simple_ev(Ev::Click, FilterMenuMsg::AddFilter),
                "➕",
            ],
            div![self
                .filters
                .iter()
                .enumerate()
                .map(|(filter_i, (_field_i, _op, value))| {
                    let op_ev = |op, name: &str| {
                        option![
                            // TODO: FIXME: This event only triggers on _mouse clicks_ and not
                            // specifically when the option changes.
                            simple_ev(Ev::Click, FilterMenuMsg::SetOp { filter_i, op }),
                            name,
                        ]
                    };
                    div![
                        C![C.filter_menu_item],
                        select![
                            C![C.filter_menu_item_elem, C.filter_menu_field],
                            self.fields
                                .iter()
                                .enumerate()
                                .map(|(field_i, field)| option![
                                    simple_ev(
                                        Ev::Click,
                                        FilterMenuMsg::SetField { filter_i, field_i }
                                    ),
                                    field,
                                ])
                                .collect::<Vec<_>>()
                        ],
                        select![
                            C![C.filter_menu_item_elem, C.filter_menu_operator],
                            op_ev(FilterOp::NotEquals, "!="),
                            op_ev(FilterOp::Equals, "=="),
                            op_ev(FilterOp::GrTh, ">"),
                            op_ev(FilterOp::GrEq, ">="),
                            op_ev(FilterOp::LeTh, "<"),
                            op_ev(FilterOp::LeEq, "<="),
                        ],
                        input![
                            C![C.filter_menu_item_elem, C.filter_menu_value],
                            attrs! { At::Value => value },
                            input_ev(Ev::Input, move |value| FilterMenuMsg::SetValue {
                                filter_i,
                                value,
                            }),
                        ],
                        button![
                            simple_ev(Ev::Click, FilterMenuMsg::DeleteFilter { filter_i }),
                            C![C.filter_menu_delete],
                            "✖",
                        ]
                    ]
                })
                .collect::<Vec<_>>(),],
        ]
    }
}
