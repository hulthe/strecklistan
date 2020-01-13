use crate::database::{create_pool, DatabasePool};
use crate::models::user::{generate_salted_hash, JWTConfig, JWT};
use crate::models::{NewEvent, NewSignup, User};
use crate::schema::tables::event_signups;
use crate::schema::tables::events;
use crate::schema::tables::users;
use chrono::{Duration, Utc};
use diesel::RunQueryDsl;
use dotenv::dotenv;

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

    let now = Utc::now();

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
        let connection = self
            .db_pool
            .get()
            .expect("Could not get database connection");
        diesel::delete(events::table)
            .execute(&connection)
            .expect("Could not truncate testing database table");
        diesel::delete(event_signups::table)
            .execute(&connection)
            .expect("Could not truncate testing database table");
        diesel::delete(users::table)
            .execute(&connection)
            .expect("Could not truncate testing database table");
    }
}

pub struct UserSession {
    pub user: User,
    pub bearer: String,
}

impl UserSession {
    pub fn new(db_pool: &DatabasePool, jwt_config: &JWTConfig) -> UserSession {
        let user = User {
            name: "test_user".into(),
            display_name: Some("Test User".into()),
            salted_pass: generate_salted_hash("password").expect("Could not create password hash"),
            hash_iterations: 10,
        };
        let connection = db_pool.get().expect("Could not get database connection");
        diesel::insert_into(users::table)
            .values(&user)
            .execute(&connection)
            .expect("Could not add new user for testing");

        let jwt = JWT::new(&user, jwt_config);
        UserSession {
            bearer: format!("Bearer {}", jwt.encode_jwt(jwt_config).unwrap()),
            user,
        }
    }
}
