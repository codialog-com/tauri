-- Initial database schema for Codialog application
-- Author: Tom Sapletta <info@softreck.dev>
-- License: Apache-2.0

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- User sessions table
CREATE TABLE IF NOT EXISTS user_sessions (
    session_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    bitwarden_session TEXT,
    user_data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id)
);

-- Indexes for user_sessions
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_user_sessions_last_activity ON user_sessions(last_activity);

-- User files table for CV, cover letters, and attachments
CREATE TABLE IF NOT EXISTS user_files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES user_sessions(session_id) ON DELETE CASCADE,
    file_type VARCHAR(50) NOT NULL, -- 'cv', 'cover_letter', 'attachment'
    original_filename VARCHAR(500) NOT NULL,
    stored_filename VARCHAR(500) NOT NULL,
    file_path VARCHAR(1000) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100),
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- Indexes for user_files
CREATE INDEX IF NOT EXISTS idx_user_files_session_id ON user_files(session_id);
CREATE INDEX IF NOT EXISTS idx_user_files_type ON user_files(file_type);
CREATE INDEX IF NOT EXISTS idx_user_files_active ON user_files(is_active);

-- Form data cache table
CREATE TABLE IF NOT EXISTS form_data_cache (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES user_sessions(session_id) ON DELETE CASCADE,
    url_pattern VARCHAR(500) NOT NULL,
    form_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(session_id, url_pattern)
);

-- Indexes for form_data_cache
CREATE INDEX IF NOT EXISTS idx_form_data_session_id ON form_data_cache(session_id);
CREATE INDEX IF NOT EXISTS idx_form_data_url ON form_data_cache(url_pattern);
CREATE INDEX IF NOT EXISTS idx_form_data_updated ON form_data_cache(updated_at);

-- DSL scripts cache table
CREATE TABLE IF NOT EXISTS dsl_scripts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES user_sessions(session_id) ON DELETE CASCADE,
    url_pattern VARCHAR(500) NOT NULL,
    html_hash VARCHAR(64) NOT NULL, -- SHA256 hash of the HTML content
    generated_script TEXT NOT NULL,
    script_type VARCHAR(50) NOT NULL DEFAULT 'form_fill',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    used_count INTEGER DEFAULT 0,
    last_used TIMESTAMPTZ
);

-- Indexes for dsl_scripts
CREATE INDEX IF NOT EXISTS idx_dsl_scripts_session_id ON dsl_scripts(session_id);
CREATE INDEX IF NOT EXISTS idx_dsl_scripts_url ON dsl_scripts(url_pattern);
CREATE INDEX IF NOT EXISTS idx_dsl_scripts_hash ON dsl_scripts(html_hash);
CREATE INDEX IF NOT EXISTS idx_dsl_scripts_created ON dsl_scripts(created_at);

-- Application logs table (for advanced querying and analytics)
CREATE TABLE IF NOT EXISTS application_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    level VARCHAR(20) NOT NULL, -- DEBUG, INFO, WARN, ERROR
    target VARCHAR(100),
    module VARCHAR(100),
    message TEXT NOT NULL,
    session_id UUID REFERENCES user_sessions(session_id) ON DELETE SET NULL,
    additional_data JSONB DEFAULT '{}'
);

-- Indexes for application_logs
CREATE INDEX IF NOT EXISTS idx_app_logs_timestamp ON application_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_app_logs_level ON application_logs(level);
CREATE INDEX IF NOT EXISTS idx_app_logs_session_id ON application_logs(session_id);
CREATE INDEX IF NOT EXISTS idx_app_logs_target ON application_logs(target);

-- Bitwarden credentials cache (encrypted)
CREATE TABLE IF NOT EXISTS bitwarden_cache (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES user_sessions(session_id) ON DELETE CASCADE,
    credential_id VARCHAR(255) NOT NULL,
    url_pattern VARCHAR(500),
    username_encrypted TEXT,
    notes_encrypted TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE(session_id, credential_id)
);

-- Indexes for bitwarden_cache
CREATE INDEX IF NOT EXISTS idx_bw_cache_session_id ON bitwarden_cache(session_id);
CREATE INDEX IF NOT EXISTS idx_bw_cache_url ON bitwarden_cache(url_pattern);
CREATE INDEX IF NOT EXISTS idx_bw_cache_expires ON bitwarden_cache(expires_at);

-- Create a function to clean up expired data
CREATE OR REPLACE FUNCTION cleanup_expired_data()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER := 0;
    temp_count INTEGER;
BEGIN
    -- Clean up expired sessions
    DELETE FROM user_sessions WHERE expires_at < NOW();
    GET DIAGNOSTICS temp_count = ROW_COUNT;
    deleted_count := deleted_count + temp_count;
    
    -- Clean up expired Bitwarden cache
    DELETE FROM bitwarden_cache WHERE expires_at < NOW();
    GET DIAGNOSTICS temp_count = ROW_COUNT;
    deleted_count := deleted_count + temp_count;
    
    -- Clean up old application logs (older than 30 days)
    DELETE FROM application_logs WHERE timestamp < NOW() - INTERVAL '30 days';
    GET DIAGNOSTICS temp_count = ROW_COUNT;
    deleted_count := deleted_count + temp_count;
    
    -- Clean up old DSL scripts cache (older than 7 days and not used recently)
    DELETE FROM dsl_scripts 
    WHERE created_at < NOW() - INTERVAL '7 days' 
    AND (last_used IS NULL OR last_used < NOW() - INTERVAL '3 days');
    GET DIAGNOSTICS temp_count = ROW_COUNT;
    deleted_count := deleted_count + temp_count;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Create a trigger to automatically update the updated_at column
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply the trigger to form_data_cache
CREATE TRIGGER update_form_data_cache_updated_at
    BEFORE UPDATE ON form_data_cache
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Insert initial admin session (for development)
INSERT INTO user_sessions (session_id, user_id, user_data, expires_at)
VALUES (
    uuid_generate_v4(),
    'admin@codialog.dev',
    '{"first_name": "Admin", "last_name": "User", "email": "admin@codialog.dev", "preferences": {"theme": "dark", "auto_fill": true}}',
    NOW() + INTERVAL '30 days'
) ON CONFLICT (user_id) DO NOTHING;
