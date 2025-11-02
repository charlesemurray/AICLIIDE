-- Add session metadata columns to conversations table
ALTER TABLE conversations ADD COLUMN session_name TEXT;
ALTER TABLE conversations ADD COLUMN session_type TEXT;
ALTER TABLE conversations ADD COLUMN session_status TEXT;
ALTER TABLE conversations ADD COLUMN last_active INTEGER;

-- Create index for faster session queries
CREATE INDEX IF NOT EXISTS idx_conversations_session_status ON conversations(session_status);
