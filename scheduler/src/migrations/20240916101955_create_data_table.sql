-- Create the data table
CREATE TABLE IF NOT EXISTS bms_data (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  job_id UUID,
  download JSONB,
  ping JSONB,
  head JSONB,
  worker_name VARCHAR(255), -- TODO: make sure we have a worker name
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
