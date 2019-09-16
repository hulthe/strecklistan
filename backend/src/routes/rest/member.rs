use crate::database::DatabasePool;
use crate::util::status_json::StatusJson as SJ;
use diesel::prelude::*;
use laggit_api::member::Member;
use rocket::{get, State};
use rocket_contrib::json::Json;

#[get("/members")]
pub fn get_members(db_pool: State<DatabasePool>) -> Result<Json<Vec<Member>>, SJ> {
    let connection = db_pool.inner().get()?;
    use crate::schema::tables::members::dsl::*;

    Ok(Json(members.load(&connection)?))
}
