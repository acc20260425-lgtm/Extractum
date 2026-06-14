ALTER TABLE prompt_pack_runs
ADD COLUMN client_request_id TEXT CHECK (client_request_id IS NULL OR length(trim(client_request_id)) > 0);

CREATE UNIQUE INDEX IF NOT EXISTS idx_prompt_pack_runs_client_request_id_unique
ON prompt_pack_runs(client_request_id)
WHERE client_request_id IS NOT NULL;
