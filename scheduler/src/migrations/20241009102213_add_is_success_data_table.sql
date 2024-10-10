-- Rename bms_data table to worker_data
ALTER TABLE IF EXISTS bms_data RENAME TO worker_data;

-- Add is_success to data table
ALTER TABLE worker_data ADD COLUMN IF NOT EXISTS is_success BOOLEAN;