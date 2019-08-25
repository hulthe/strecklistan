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
    inventory (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
        price -> Nullable<Int4>,
    }
}

table! {
    transaction_bundles (id) {
        id -> Int4,
        transaction_id -> Int4,
        bundle_price -> Nullable<Int4>,
        change -> Int4,
    }
}

table! {
    transaction_items (id) {
        id -> Int4,
        bundle_id -> Int4,
        item_id -> Int4,
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
joinable!(transaction_bundles -> transactions (transaction_id));
joinable!(transaction_items -> inventory (item_id));
joinable!(transaction_items -> transaction_bundles (bundle_id));

allow_tables_to_appear_in_same_query!(
    events,
    event_signups,
    inventory,
    transaction_bundles,
    transaction_items,
    transactions,
    users,
);
