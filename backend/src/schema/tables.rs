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
        event -> Int4,
        name -> Varchar,
        email -> Varchar,
    }
}

table! {
    inventory (name) {
        name -> Varchar,
        price -> Nullable<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::inventory::InventoryItemChange;
    transaction_items (id) {
        id -> Int4,
        transaction_id -> Int4,
        item_name -> Varchar,
        item_price -> Nullable<Int4>,
        change -> InventoryItemChange,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        amount -> Int4,
        description -> Nullable<Text>,
        time -> Timestamp,
    }
}

table! {
    users (name) {
        name -> Varchar,
        display_name -> Nullable<Varchar>,
        salted_pass -> Varchar,
        hash_iterations -> Int4,
    }
}

joinable!(event_signups -> events (event));
joinable!(transaction_items -> inventory (item_name));
joinable!(transaction_items -> transactions (transaction_id));

allow_tables_to_appear_in_same_query!(
    events,
    event_signups,
    inventory,
    transaction_items,
    transactions,
    users,
);
