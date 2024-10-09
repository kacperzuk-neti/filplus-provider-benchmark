-- Rename bms_data table to worker_data
ALTER TABLE bms_data RENAME TO worker_data;

-- Add is_success to data table
ALTER TABLE worker_data ADD COLUMN is_success BOOLEAN DEFAULT FALSE;