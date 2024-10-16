-- Add sub_job_id to worker_data table
ALTER TABLE worker_data
    ADD COLUMN sub_job_id UUID,
    ADD CONSTRAINT worker_data_sub_job_id_fkey FOREIGN KEY (sub_job_id) REFERENCES sub_jobs(id) ON DELETE CASCADE;

-- Create index on sub_job_id
CREATE INDEX worker_data_sub_job_id_index ON worker_data(sub_job_id);

-- Creeate index on job_id
CREATE INDEX worker_data_job_id_index ON worker_data(job_id);

-- Add sub_job_id to workers table
ALTER TABLE workers
    ADD COLUMN sub_job_id UUID,
    ADD CONSTRAINT workers_sub_job_id_fkey FOREIGN KEY (sub_job_id) REFERENCES sub_jobs(id) ON DELETE CASCADE;

-- Create index on job_id
CREATE INDEX workers_job_id_index ON workers(job_id);

-- Create index on sub_job_id
CREATE INDEX workers_sub_job_id_index ON workers(sub_job_id);
