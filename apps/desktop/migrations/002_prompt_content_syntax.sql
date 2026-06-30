ALTER TABLE prompts ADD COLUMN content_syntax TEXT NOT NULL DEFAULT 'auto'
  CHECK (content_syntax IN ('auto', 'plaintext', 'markdown', 'xml', 'json'));
