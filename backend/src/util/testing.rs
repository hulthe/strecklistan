use chrono::{Duration, Local};
use diesel::RunQueryDsl;
use dotenv::dotenv;
use models::{NewEvent, NewSignup};
use schema::tables::event_signups;
use schema::tables::events;
use schema::tables::users;
use database::{DatabasePool, create_pool};

pub fn generate_new_events(old: usize, new: usize) -> Vec<NewEvent> {
    let mut events = vec![];

    let new_event = |time, p| -> NewEvent {
        NewEvent {
            title: "My Event".into(),
            background: "http://site/image.png".into(),
            location: "Somewhere".into(),
            start_time: time,
            end_time: time + Duration::hours(2),
            price: Some(p),
        }
    };

    let now = Local::now().naive_local();

    for i in 0..old {
        let time = now - Duration::weeks(2 * (i + 1) as i64);
        events.push(new_event(time, -(i as i32) - 1));
    }

    for i in 0..new {
        let time = now + Duration::weeks(2 * (i + 1) as i64);
        events.push(new_event(time, (i as i32) + 1));
    }

    events
}

pub fn generate_event_signups(count: usize, event: i32) -> Vec<NewSignup> {
    let mut signups = vec![];
    for i in 0..count {
        signups.push(NewSignup {
            event,
            name: format!("Alice Bobsson the {}nthdst", i),
            email: "alice.bob@nsa.gov".into(),
        });
    }
    signups
}

pub struct DatabaseState {
    db_pool: DatabasePool,
}

impl DatabaseState {
    pub fn new() -> (DatabaseState, DatabasePool) {
        dotenv().ok();
        let db_pool = create_pool().expect("Could not create database pool");
        let state = DatabaseState {
            db_pool: db_pool.clone(),
        };
        (state, db_pool)
    }
}

impl Drop for DatabaseState {
    fn drop(&mut self) {
        let connection = self.db_pool.get().expect(
            "Could not get database connection",
        );
        diesel::delete(events::table).execute(&connection).expect(
            "Could not truncate testing database table",
        );
        diesel::delete(event_signups::table)
            .execute(&connection)
            .expect("Could not truncate testing database table");
        diesel::delete(users::table).execute(&connection).expect(
            "Could not truncate testing database table",
        );
    }
}
