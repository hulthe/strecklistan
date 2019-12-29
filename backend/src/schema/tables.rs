table! {
    use diesel::sql_types::*;
    use laggit_api::book_account::BookAccountTypeMapping;
    book_accounts (id) {
        id -> Int4,
        name -> Text,
        account_type -> BookAccountTypeMapping,
        creditor -> Nullable<Int4>,
    }
}

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
        name -> Nullable<Text>,
        price -> Nullable<Int4>,
        image_url -> Nullable<Text>,
    }
}

table! {
    inventory_bundle_items (id) {
        id -> Int4,
        bundle_id -> Int4,
        item_id -> Int4,
    }
}

table! {
    inventory_bundles (id) {
        id -> Int4,
        name -> Text,
        price -> Int4,
        image_url -> Nullable<Text>,
    }
}

table! {
    inventory_tags (tag, item_id) {
        tag -> Text,
        item_id -> Int4,
    }
}

table! {
    members (id) {
        id -> Int4,
        first_name -> Text,
        last_name -> Text,
        nickname -> Nullable<Text>,
    }
}

table! {
    transaction_bundles (id) {
        id -> Int4,
        transaction_id -> Int4,
        description -> Nullable<Text>,
        price -> Nullable<Int4>,
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
        description -> Nullable<Text>,
        time -> Timestamp,
        debited_account -> Int4,
        credited_account -> Int4,
        amount -> Int4,
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

joinable!(book_accounts -> members (creditor));
joinable!(event_signups -> events (event));
joinable!(inventory_bundle_items -> inventory (item_id));
joinable!(inventory_bundle_items -> inventory_bundles (bundle_id));
joinable!(inventory_tags -> inventory (item_id));
joinable!(transaction_bundles -> transactions (transaction_id));
joinable!(transaction_items -> inventory (item_id));
joinable!(transaction_items -> transaction_bundles (bundle_id));

allow_tables_to_appear_in_same_query!(
    book_accounts,
    events,
    event_signups,
    inventory,
    inventory_bundle_items,
    inventory_bundles,
    inventory_tags,
    members,
    transaction_bundles,
    transaction_items,
    transactions,
    users,
);
