ALTER TABLE prompt_pack_runs
ADD COLUMN run_label TEXT
CHECK (run_label IS NULL OR length(trim(run_label)) > 0);
