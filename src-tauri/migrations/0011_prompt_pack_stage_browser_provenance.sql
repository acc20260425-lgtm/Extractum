ALTER TABLE prompt_pack_stage_runs
ADD COLUMN browser_run_id TEXT
CHECK (browser_run_id IS NULL OR length(trim(browser_run_id)) > 0);

ALTER TABLE prompt_pack_stage_runs
ADD COLUMN browser_run_status TEXT
CHECK (browser_run_status IS NULL OR length(trim(browser_run_status)) > 0);

ALTER TABLE prompt_pack_stage_runs
ADD COLUMN browser_completion_reason TEXT
CHECK (browser_completion_reason IS NULL OR length(trim(browser_completion_reason)) > 0);

ALTER TABLE prompt_pack_stage_runs
ADD COLUMN browser_provider_mode TEXT
CHECK (browser_provider_mode IS NULL OR length(trim(browser_provider_mode)) > 0);

ALTER TABLE prompt_pack_stage_runs
ADD COLUMN browser_run_message TEXT
CHECK (browser_run_message IS NULL OR length(trim(browser_run_message)) > 0);
