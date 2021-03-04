use crate::app::{Msg, StateReady};
use crate::generated::css_classes::C;
use crate::util::{simple_ev, DATE_INPUT_FMT};
use chrono::{DateTime, Datelike, Duration, IsoWeek, NaiveDate, Utc, Weekday};
use seed::{prelude::*, *};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use strecklistan_api::{inventory::InventoryItemId, transaction::Transaction};

#[derive(Clone, Debug)]
pub enum AnalyticsMsg {
    ComputeCharts,
    ComputedChart {
        item_id: InventoryItemId,
        chart: Node<AnalyticsMsg>,
    },
    SetStartDate(String),
    SetEndDate(String),
}

#[derive(Clone)]
pub struct AnalyticsPage {
    /// An index of the inventory stocks at the start of every week
    inventory_by_week: Rc<BTreeMap<IsoWeek, HashMap<InventoryItemId, i32>>>,

    /// Pre-computed and cached charts
    charts: HashMap<InventoryItemId, Node<AnalyticsMsg>>,

    /// Start-date filter for computing charts
    start_date: DateTime<Utc>,

    /// End-date filter for computing charts
    end_date: DateTime<Utc>,

    /// Toggle for disabling the "calculate charts" button
    calculation_in_progress: bool,
}

impl AnalyticsPage {
    pub fn new(global: &StateReady) -> Self {
        let now = Utc::now();
        AnalyticsPage {
            inventory_by_week: Rc::new(calculate_inventory_by_week(&global.transaction_history)),
            charts: HashMap::new(),
            start_date: now - Duration::days(365),
            end_date: now,
            calculation_in_progress: false,
        }
    }

    pub fn update(
        &mut self,
        msg: AnalyticsMsg,
        global: &mut StateReady,
        orders: &mut impl Orders<Msg>,
    ) {
        let mut orders_local = orders.proxy(|msg| Msg::AnalyticsMsg(msg));
        match msg {
            AnalyticsMsg::ComputeCharts => {
                self.charts.clear();
                self.calculation_in_progress = true;
                self.plot_next_item(global, &mut orders_local);
            }
            AnalyticsMsg::ComputedChart { item_id, chart } => {
                self.charts.insert(item_id, chart);
                self.plot_next_item(global, &mut orders_local);
            }
            AnalyticsMsg::SetStartDate(input) => {
                if let Ok(date) = NaiveDate::parse_from_str(&input, DATE_INPUT_FMT) {
                    self.start_date = DateTime::from_utc(date.and_hms(0, 0, 0), Utc);
                }
            }
            AnalyticsMsg::SetEndDate(input) => {
                if let Ok(date) = NaiveDate::parse_from_str(&input, DATE_INPUT_FMT) {
                    self.end_date = DateTime::from_utc(date.and_hms(0, 0, 0), Utc);
                }
            }
        }
    }

    pub fn view(&self, global: &StateReady) -> Node<Msg> {
        div![
            C![C.accounting_page],
            div![
                input![
                    attrs! {At::Type => "date"},
                    attrs! {At::Value => self.start_date.format(DATE_INPUT_FMT).to_string()},
                    input_ev(Ev::Input, |input| AnalyticsMsg::SetStartDate(input)),
                ],
                input![
                    attrs! {At::Type => "date"},
                    attrs! {At::Value => self.end_date.format(DATE_INPUT_FMT).to_string()},
                    input_ev(Ev::Input, |input| AnalyticsMsg::SetEndDate(input)),
                ],
                if self.calculation_in_progress {
                    button![
                        C![C.wide_button],
                        div![
                            C![C.lds_ripple],
                            style! {
                                St::Position => "absolute",
                                St::MarginTop => "-20px",
                            },
                            div![],
                            div![],
                        ],
                        attrs! { At::Disabled => true },
                        "Beräkna Statistik",
                    ]
                } else {
                    button![
                        C![C.wide_button],
                        simple_ev(Ev::Click, AnalyticsMsg::ComputeCharts),
                        "Beräkna Statistik",
                    ]
                },
            ],
            if self.charts.is_empty() {
                i!["Här var det tomt... Prova att trycka på knappen ;)"]
            } else {
                div![global.inventory.values().map(|item| {
                    if let Some(chart) = self.charts.get(&item.id) {
                        chart.clone()
                    } else {
                        h2![&item.name, i![" - laddar..."],]
                    }
                })]
            },
        ]
        .map_msg(|msg| Msg::AnalyticsMsg(msg))
    }

    fn plot_next_item(&mut self, global: &StateReady, orders: &mut impl Orders<AnalyticsMsg>) {
        self.calculation_in_progress = false;
        for item in global.inventory.values() {
            if !self.charts.contains_key(&item.id) {
                self.calculation_in_progress = true;

                let inventory_by_week = self.inventory_by_week.clone();
                let start_date = self.start_date;
                let end_date = self.end_date;
                let item_id = item.id;
                let item_name = item.name.clone();

                orders.after_next_render(move |_| {
                    let chart = plot_sales_over_time(
                        inventory_by_week,
                        start_date,
                        end_date,
                        item_id,
                        item_name,
                    );
                    AnalyticsMsg::ComputedChart { item_id, chart }
                });
                break;
            }
        }
    }
}

fn week_date(week: IsoWeek) -> DateTime<Utc> {
    let naive = NaiveDate::from_isoywd(week.year(), week.week(), Weekday::Mon).and_hms(0, 0, 0);
    DateTime::from_utc(naive, Utc)
}

fn calculate_inventory_by_week(
    transactions_unsorted: &[Transaction],
) -> BTreeMap<IsoWeek, HashMap<InventoryItemId, i32>> {
    let mut transactions = BTreeMap::new();

    for transaction in transactions_unsorted.iter() {
        transactions
            .entry(transaction.time.iso_week())
            .or_insert(vec![])
            .push(transaction.clone());
    }

    let mut inventory = HashMap::new();
    let mut result = BTreeMap::new();

    for (week, weeks_transactions) in transactions.iter() {
        for transaction in weeks_transactions.iter() {
            for bundle in transaction.bundles.iter() {
                for (item_id, item_count) in bundle.item_ids.iter() {
                    *inventory.entry(*item_id).or_default() += *item_count as i32 * bundle.change;
                }
            }
        }
        result.insert(*week, inventory.clone());
    }

    result
}

fn plot_sales_over_time(
    inventory_by_week: Rc<BTreeMap<IsoWeek, HashMap<InventoryItemId, i32>>>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    item_id: InventoryItemId,
    name: String,
) -> Node<AnalyticsMsg> {
    let mut iter = inventory_by_week
        .iter()
        .map(|(week, inventory)| {
            let stock = *inventory.get(&item_id).unwrap_or(&0);
            (week_date(*week), stock)
        })
        .filter(|&(date, _)| date >= start_date)
        .filter(|&(date, _)| date <= end_date);

    let mut last_weeks_stock = iter.next().map(|(_, s)| s).unwrap_or(0);
    let points: Vec<(String, u32)> = iter
        .map(|(date, this_weeks_stock)| {
            let sales = last_weeks_stock - this_weeks_stock;
            last_weeks_stock = this_weeks_stock;
            (date, sales)
        })
        .filter(|(_, sales)| *sales >= 0)
        .map(|(date, sales)| {
            let datefmt = format!("{} w{:.02}", date.year(), date.iso_week().week());
            (datefmt, sales as u32)
        })
        .collect();

    plot(name, &points)
}

fn plot<K>(name: String, points: &[(K, u32)]) -> Node<AnalyticsMsg>
where
    K: std::fmt::Display,
{
    let y_max = points.iter().map(|(_, v)| *v).max().unwrap();
    div![
        h2![name],
        div![
            C![C.chart_histogram],
            points
                .iter()
                .map(|(k, v)| {
                    let percentage = if y_max == 0 { 0 } else { v * 100 / y_max };

                    div![
                        C![C.chart_histogram_col],
                        div![style!(St::FlexBasis => format!("{}%", 100 - percentage)),],
                        div![
                            C![C.chart_histogram_col_line, C.chart_col_tooltip],
                            style!(St::FlexBasis => format!("{}%", percentage)),
                            span![C![C.chart_col_tooltiptext], format!("{}", v),],
                        ],
                        div![C![C.chart_histogram_col_label], format!("{}", k),],
                    ]
                })
                .collect::<Vec<_>>()
        ],
    ]
}
