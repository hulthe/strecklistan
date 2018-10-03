table! {
    events (id) {
        id -> Int4,
        title -> Text,
        background -> Text,
        location -> Text,
        start_time -> Timestamp,
        end_time -> Timestamp,
        price -> Int4,
        published -> Bool,
    }
}

table! {
    event_signups (id) {
        id -> Int4,
        event_id -> Nullable<Int4>,
        name -> Varchar,
        email -> Varchar,
    }
}

joinable!(event_signups -> events (event_id));

allow_tables_to_appear_in_same_query!(
    events,
    event_signups,
);
