// Bindings to database views aren't automatically generated by diesel.
// This file has to be updated manually.

table! {
    events_with_signups (id) {
        id -> Int4,
        title -> Text,
        background -> Text,
        location -> Text,
        start_time -> Timestamptz,
        end_time -> Timestamptz,
        price -> Int4,
        published -> Bool,
        signups -> Int8,
    }
}

table! {
    inventory_stock (name) {
        id -> Int4,
        name -> Text,
        price -> Nullable<Int4>,
        image_url -> Nullable<Text>,
        deleted_at -> Nullable<Timestamptz>,
        stock -> Int4,
    }
}
