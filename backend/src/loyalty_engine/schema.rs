// @generated automatically by Diesel CLI (and kept in sync by hand here).

diesel::table! {
    loyalty_programs (id) {
        id -> Text,
        name -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    loyalty_members (id) {
        id -> Text,
        program_id -> Text,
        name -> Text,
        email -> Nullable<Text>,
        external_contact_id -> Nullable<Text>,
        points -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    loyalty_sessions (id) {
        id -> Text,
        member_id -> Text,
        status -> Text,
        created_at -> Timestamptz,
        expires_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(loyalty_members -> loyalty_programs (program_id));
diesel::joinable!(loyalty_sessions -> loyalty_members (member_id));

diesel::allow_tables_to_appear_in_same_query!(loyalty_programs, loyalty_members, loyalty_sessions,);
