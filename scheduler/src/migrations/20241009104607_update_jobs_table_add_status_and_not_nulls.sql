-- Create job_status enum 
CREATE TYPE job_status AS ENUM ('pending', 'running', 'completed', 'failed');

-- Add status to jobs table
ALTER TABLE jobs 
    ADD COLUMN status job_status;

-- Update jobs table to set default values and change columns to NOT NULL
UPDATE jobs 
SET 
    status = COALESCE(status, 'completed'),
    url = COALESCE(url, ''),
    routing_key = COALESCE(routing_key, ''),
    details = COALESCE(details, '{}'::jsonb);

-- Change jobs table columns to NOT NULL
ALTER TABLE jobs 
    ALTER COLUMN url SET NOT NULL,
    ALTER COLUMN routing_key SET NOT NULL,
    ALTER COLUMN details SET NOT NULL
    ALTER COLUMN status SET NOT NULL;
