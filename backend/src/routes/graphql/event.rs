use super::context::Context;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use juniper::FieldResult;
use models::event::EventWithSignups as EventWS;
use models::signup::Signup;

graphql_object!(EventWS: Context |&self| {
    field id() -> i32 { self.id }
    field title() -> &str { self.title.as_str() }
    field background() -> &str { self.background.as_str() }
    field location() -> &str { self.location.as_str() }
    field start_time() -> NaiveDateTime { self.start_time }
    field end_time() -> NaiveDateTime { self.end_time }
    field price() -> i32 { self.price }
    field published() -> bool { self.published }
    field signup_count(&executor) -> FieldResult<i32> {
        use schema::tables::event_signups::dsl::*;
        let connection = executor.context().pool.get()?;
        let signups: Vec<Signup> = event_signups
            .filter(event.eq(self.id))
            .load(&connection)?;
        Ok(signups.len() as i32)
    }
    field signups(&executor) -> FieldResult<Vec<Signup>> {
        use schema::tables::event_signups::dsl::*;
        executor.context().get_auth("signups")?;
        let connection = executor.context().pool.get()?;
        Ok(event_signups
            .filter(event.eq(self.id))
            .load(&connection)?)
    }
});
