use crate::app::{Msg, StateReady};
use crate::generated::css_classes::C;
use seed::{prelude::*, *};
use std::collections::{HashMap, BTreeMap};
use strecklistan_api::{
    inventory::InventoryItemId,
    transaction::Transaction,
};
use plotters::prelude::*;
use std::rc::Rc;
use chrono::{Duration, Datelike, Utc, DateTime, NaiveDate, Weekday, IsoWeek};
use crate::util::DATE_INPUT_FMT;


#[derive(Clone, Debug)]
pub enum AnalyticsMsg {
    ComputeCharts,
    ComputedChart {
        item_id: InventoryItemId,
        chart: String,
    },
    SetStartDate(String),
    SetEndDate(String),
}

#[derive(Clone)]
pub struct AnalyticsPage {
    inventory_by_week: Rc<BTreeMap<IsoWeek, HashMap<InventoryItemId, i32>>>,
    charts: HashMap<InventoryItemId, String>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

impl AnalyticsPage {
    pub fn new(global: &StateReady) -> Self {
        let now = Utc::now();
        AnalyticsPage {
            inventory_by_week: Rc::new(calculate_inventory_by_week(&global.transaction_history)),
            charts: HashMap::new(),
            start_date: now - Duration::days(365),
            end_date: now,
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
            class!["accounting_page"],
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
                button![
                    "Beräkna Statistik",
                    class![C.wide_button, C.border_on_focus],
                    simple_ev(Ev::Click, AnalyticsMsg::ComputeCharts),
                ],
            ],
            global.inventory.values()
                .map(|item| {
                    if let Some(svg) = self.charts.get(&item.id) {
                        div![raw![svg]]
                    } else {
                        div!["not computed yet"]
                    }
                }),
        ]
        .map_msg(|msg| Msg::AnalyticsMsg(msg))
    }

    fn plot_next_item(&mut self, global: &StateReady, orders: &mut impl Orders<AnalyticsMsg>) {
        for item in global.inventory.values() {
            if !self.charts.contains_key(&item.id) {
                let inventory_by_week = self.inventory_by_week.clone();
                let start_date = self.start_date;
                let end_date = self.end_date;
                let item_id = item.id;
                let item_name = item.name.clone();
                orders.after_next_render(move |_| {
                    let chart = plot_item_stock_over_time(inventory_by_week, start_date, end_date
                                                          ,item_id, item_name);
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
        transactions.entry(transaction.time.iso_week()).or_insert(vec![]).push(transaction.clone());
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

fn plot_item_stock_over_time(
    inventory_by_week: Rc<BTreeMap<IsoWeek, HashMap<InventoryItemId, i32>>>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    item_id: InventoryItemId,
    name: String) -> String {

    let x_min = inventory_by_week.keys()
        .map(|&week| week_date(week))
        .filter(|&date| date >= start_date)
        .next()
        .unwrap();

    let x_max = inventory_by_week.keys()
        .map(|&week| week_date(week))
        .filter(|&date| date <= end_date)
        .rev()
        .next()
        .unwrap()
        .max(x_min);

    let mut points = Vec::new();
    for (week, stock) in inventory_by_week.iter()
                .map(|(week, stock)| (week_date(*week), stock))
                .filter(|&(date, _)| date >= start_date)
                .filter(|&(date, _)| date <= end_date) {
        points.push((week, *stock.get(&item_id).unwrap_or(&0)));
    }

    let mut svg_data = String::new();
    let canvas = SVGBackend::with_string(&mut svg_data, (640, 480)).into_drawing_area();

    let y_max = *points.iter().map(|(_, count)| count).max().unwrap_or(&0);

    let mut chart = ChartBuilder::on(&canvas)
        .caption(name, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(x_min..x_max, 0..y_max)
        .unwrap();

    chart.draw_series(LineSeries::new(points, &RED,)).unwrap()
        .label("Antal i lager")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();

    drop(chart);
    drop(canvas);

    svg_data
}
