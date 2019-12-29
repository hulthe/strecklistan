use crate::database::event::{get_event_ws, get_event_ws_range};
use crate::database::DatabasePool;
use crate::models::event::EventWithSignups as EventWS;
use crate::util::status_json::StatusJson as SJ;
use rocket::{get, State};
use rocket_contrib::json::Json;

#[get("/event/<id>")]
pub fn get_event(id: i32, db_pool: State<DatabasePool>) -> Result<Json<EventWS>, SJ> {
    Ok(Json(get_event_ws(db_pool.inner().get()?, id, true)?))
}

#[get("/events?<low>&<high>")]
pub fn get_event_range(
    low: i64,
    high: i64,
    db_pool: State<DatabasePool>,
) -> Result<Json<Vec<EventWS>>, SJ> {
    Ok(Json(get_event_ws_range(
        db_pool.inner().get()?,
        low,
        high,
        true,
    )?))
}
