ALTER TABLE prompt_pack_runs
ADD COLUMN runtime_provider TEXT NOT NULL DEFAULT 'api'
CHECK (runtime_provider IN ('api', 'gemini_browser'));

ALTER TABLE prompt_pack_runs
ADD COLUMN browser_provider_config_json TEXT
CHECK (
    browser_provider_config_json IS NULL
    OR length(trim(browser_provider_config_json)) > 0
);
