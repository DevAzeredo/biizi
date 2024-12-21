ALTER TABLE job_opportunities
ADD COLUMN company_id BIGINT REFERENCES companies(id) ON DELETE CASCADE;