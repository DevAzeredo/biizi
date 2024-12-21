CREATE TABLE employees (
    id BIGSERIAL PRIMARY KEY,
    full_name VARCHAR NOT NULL,
    date_of_birth VARCHAR NOT NULL,
    gender VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    phone VARCHAR NOT NULL,
    residential_address VARCHAR NOT NULL,
    is_available BOOLEAN NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    rating DOUBLE PRECISION NOT NULL
);
CREATE TABLE companies (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    address VARCHAR NOT NULL,
    logo_url VARCHAR NOT NULL
);
CREATE TABLE job_opportunities (
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    category VARCHAR NOT NULL,
    address VARCHAR NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    start_date_time VARCHAR NOT NULL,
    duration_in_hours INTEGER NOT NULL,
    pay_rate DOUBLE PRECISION NOT NULL,
    status VARCHAR NOT NULL,
    company_id BIGINT REFERENCES companies(id) ON DELETE CASCADE 
);
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    login VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    companyId BIGINT REFERENCES companies(id) ON DELETE SET NULL, 
    employeeId BIGINT REFERENCES employees(id) ON DELETE SET NULL 
);