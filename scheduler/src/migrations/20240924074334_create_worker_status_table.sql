-- Create the workers table
CREATE TABLE IF NOT EXISTS workers (
  worker_name VARCHAR(255) PRIMARY KEY,
  status VARCHAR(20) CHECK (status IN ('online', 'offline')),
  job_id UUID,
  last_seen TIMESTAMP WITH TIME ZONE,
  started_at TIMESTAMP WITH TIME ZONE,
  shutdown_at TIMESTAMP WITH TIME ZONE,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add a foreign key constraint to the bms_data table
ALTER TABLE bms_data
ADD CONSTRAINT fk_worker_name
FOREIGN KEY (worker_name)
REFERENCES workers(worker_name);

-- Create the trigger function to update the updated_at column
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Call the trigger function before every update
CREATE TRIGGER update_updated_at_trigger
BEFORE UPDATE ON workers
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();