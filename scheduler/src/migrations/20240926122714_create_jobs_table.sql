-- Create the jobs table
CREATE TABLE IF NOT EXISTS jobs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  url VARCHAR(255),
  routing_key VARCHAR(255),
  details JSONB,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Delete bms_data where job_id does not exist in jobs table (foreign key constraint)
DELETE FROM bms_data WHERE job_id NOT IN (SELECT id FROM jobs);

-- Add a foreign key constraint to the bms_data table
ALTER TABLE bms_data
ADD CONSTRAINT fk_job_id
FOREIGN KEY (job_id)
REFERENCES jobs(id);

-- Call the trigger function before every update on jobs
CREATE TRIGGER update_updated_at_trigger
BEFORE UPDATE ON jobs
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Call the trigger function before every update on bms_data
CREATE TRIGGER update_updated_at_trigger
BEFORE UPDATE ON bms_data
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

