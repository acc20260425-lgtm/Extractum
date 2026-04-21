-- Add is_member column to sources table
ALTER TABLE sources ADD COLUMN is_member BOOLEAN DEFAULT 0;
