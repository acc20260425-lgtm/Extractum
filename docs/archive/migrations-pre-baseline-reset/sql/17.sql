ALTER TABLE analysis_runs ADD COLUMN youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description'
CHECK (youtube_corpus_mode IN (
    'transcript_only',
    'transcript_description',
    'transcript_description_comments'
));
