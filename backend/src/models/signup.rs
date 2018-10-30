use schema::tables::event_signups;

/// Metadata about a guest signup to an event
#[derive(Queryable, GraphQLObject, Serialize, Deserialize, Debug, PartialEq)]
pub struct Signup {
    pub id: i32,
    pub event: i32,
    pub name: String,
    pub email: String,
}

#[derive(Insertable, GraphQLObject, Serialize, Deserialize, Debug, PartialEq)]
#[table_name = "event_signups"]
pub struct NewSignup {
    pub event: i32,
    pub name: String,
    pub email: String,
}
