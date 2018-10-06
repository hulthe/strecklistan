use schema::tables::event_signups;

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
pub struct Signup {
    pub id: i32,
    pub event: i32,
    pub name: String,
    pub email: String,
}

#[derive(Insertable, Serialize, Deserialize, Debug, PartialEq)]
#[table_name="event_signups"]
pub struct NewSignup {
    pub event: i32,
    pub name: String,
    pub email: String,
}
