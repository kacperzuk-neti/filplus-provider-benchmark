-- Create sub_job_status enum
CREATE TYPE sub_job_status AS ENUM ('pending', 'running', 'completed', 'failed');

-- Create sub_job_type enum
CREATE TYPE sub_job_type AS ENUM ('combineddhp');

-- Create sub jobs table
CREATE TABLE IF NOT EXISTS sub_jobs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  job_id UUID NOT NULL,
  status sub_job_status NOT NULL,
  type sub_job_type NOT NULL,
  details JSONB NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
);

-- Create index on job_id
CREATE INDEX IF NOT EXISTS sub_jobs_job_id_index ON sub_jobs(job_id);

-- Call the trigger function before every update on sub_jobs
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_trigger
        WHERE tgname = 'update_updated_at_trigger'
        AND tgrelid = 'sub_jobs'::regclass
    ) THEN
        CREATE TRIGGER update_updated_at_trigger
        BEFORE UPDATE ON sub_jobs
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();
    END IF;
END $$;
