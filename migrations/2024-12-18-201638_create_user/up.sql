CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    login VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    companyId BIGINT REFERENCES companies(id) ON DELETE SET NULL, 
    employeeId BIGINT REFERENCES employees(id) ON DELETE SET NULL 
);