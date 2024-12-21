diesel::table! {
    companies (id) {
        id -> Int8,
        name -> Varchar,
        description -> Varchar,
        address -> Varchar,
        logo_url -> Varchar,
    }
}

diesel::table! {
    employees (id) {
        id -> Int8,
        full_name -> Varchar,
        date_of_birth -> Varchar,
        gender -> Varchar,
        email -> Varchar,
        phone -> Varchar,
        residential_address -> Varchar,
        is_available -> Bool,
        latitude -> Float8,
        longitude -> Float8,
        rating -> Float8,
    }
}

diesel::table! {
    job_opportunities (id) {
        id -> Int8,
        title -> Varchar,
        description -> Varchar,
        category -> Varchar,
        address -> Varchar,
        latitude -> Float8,
        longitude -> Float8,
        start_date_time -> Varchar,
        duration_in_hours -> Int4,
        pay_rate -> Float8,
        status -> Varchar,
        company_id -> Nullable<Int8>,
    }
}

diesel::table! {
    users (id) {
        id -> Int8,
        login -> Varchar,
        password -> Varchar,
        companyid -> Nullable<Int8>,
        employeeid -> Nullable<Int8>,
    }
}

diesel::joinable!(job_opportunities -> companies (company_id));
diesel::joinable!(users -> companies (companyid));
diesel::joinable!(users -> employees (employeeid));

diesel::allow_tables_to_appear_in_same_query!(
    companies,
    employees,
    job_opportunities,
    users,
);