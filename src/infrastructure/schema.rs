// @generated automatically by Diesel CLI.

diesel::table! {
    companies (id) {
        id -> Int8,
        name -> Varchar,
        description -> Text,
        address -> Varchar,
        logo_url -> Varchar,
    }
}

diesel::table! {
    employees (id) {
        id -> Int8,
        full_name -> Varchar,
        date_of_birth -> Varchar,
        gender -> Nullable<Varchar>,
        email -> Varchar,
        phone -> Varchar,
        residential_address -> Varchar,
        is_available -> Bool,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
        rating -> Float8,
    }
}

diesel::table! {
    job_opportunities (id) {
        id -> Int8,
        title -> Varchar,
        description -> Text,
        company_name -> Varchar,
        company_logo_url -> Nullable<Varchar>,
        category -> Varchar,
        address -> Varchar,
        latitude -> Float8,
        longitude -> Float8,
        start_date_time -> Varchar,
        duration_in_hours -> Int4,
        pay_rate -> Float8,
        status -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    companies,
    employees,
    job_opportunities,
);
