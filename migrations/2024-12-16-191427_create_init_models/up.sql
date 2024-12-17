CREATE TABLE employees (
    id BIGINT PRIMARY KEY,
    full_name VARCHAR NOT NULL,
    date_of_birth VARCHAR NOT NULL,
    gender VARCHAR,
    email VARCHAR NOT NULL,
    phone VARCHAR NOT NULL,
    residential_address VARCHAR NOT NULL,
    is_available BOOLEAN NOT NULL,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    rating DOUBLE PRECISION NOT NULL
);

CREATE TABLE companies (
    id BIGINT PRIMARY KEY,
    name VARCHAR NOT NULL,
    description TEXT NOT NULL,
    address VARCHAR NOT NULL,
    logo_url VARCHAR NOT NULL
);

CREATE TABLE job_opportunities (
    id BIGINT PRIMARY KEY,
    title VARCHAR NOT NULL,
    description TEXT NOT NULL,
    company_name VARCHAR NOT NULL,
    company_logo_url VARCHAR,
    category VARCHAR NOT NULL,
    address VARCHAR NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    start_date_time VARCHAR NOT NULL,
    duration_in_hours INT NOT NULL,
    pay_rate DOUBLE PRECISION NOT NULL,
    status VARCHAR NOT NULL
);
