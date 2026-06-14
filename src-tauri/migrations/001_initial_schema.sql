-- Velumia V1 initial schema (LIE-11)

CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    display_name TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id TEXT NOT NULL REFERENCES roles(id),
    permission TEXT NOT NULL,
    PRIMARY KEY (role_id, permission)
);

CREATE TABLE IF NOT EXISTS user_role_assignments (
    user_id TEXT NOT NULL REFERENCES users(id),
    role_id TEXT NOT NULL REFERENCES roles(id),
    scope_type TEXT NOT NULL CHECK (scope_type IN ('global', 'org', 'resource')),
    scope_id TEXT,
    PRIMARY KEY (user_id, role_id, scope_type, scope_id)
);

CREATE TABLE IF NOT EXISTS prompt_folders (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    parent_id TEXT REFERENCES prompt_folders(id),
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS prompts (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    folder_id TEXT REFERENCES prompt_folders(id),
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    lifecycle_status TEXT NOT NULL CHECK (lifecycle_status IN ('active', 'archived', 'trashed')),
    head_version_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    trashed_at TEXT
);

CREATE TABLE IF NOT EXISTS prompt_versions (
    id TEXT PRIMARY KEY,
    prompt_id TEXT NOT NULL REFERENCES prompts(id),
    version_number INTEGER NOT NULL,
    content_path TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (prompt_id, version_number)
);

CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    UNIQUE (owner_id, name)
);

CREATE TABLE IF NOT EXISTS prompt_tags (
    prompt_id TEXT NOT NULL REFERENCES prompts(id),
    tag_id TEXT NOT NULL REFERENCES tags(id),
    PRIMARY KEY (prompt_id, tag_id)
);

CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    lifecycle_status TEXT NOT NULL CHECK (lifecycle_status IN ('active', 'archived', 'trashed')),
    langdock_agent_id TEXT UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    trashed_at TEXT
);

CREATE TABLE IF NOT EXISTS agent_subagents (
    parent_agent_id TEXT NOT NULL REFERENCES agents(id),
    child_agent_id TEXT NOT NULL REFERENCES agents(id),
    PRIMARY KEY (parent_agent_id, child_agent_id),
    CHECK (parent_agent_id != child_agent_id)
);

CREATE TABLE IF NOT EXISTS agent_prompts (
    agent_id TEXT NOT NULL REFERENCES agents(id),
    prompt_id TEXT NOT NULL REFERENCES prompts(id),
    sort_order INTEGER NOT NULL,
    UNIQUE (agent_id, sort_order)
);

CREATE TABLE IF NOT EXISTS agent_skills (
    agent_id TEXT NOT NULL REFERENCES agents(id),
    skill_id TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    UNIQUE (agent_id, sort_order)
);

CREATE TABLE IF NOT EXISTS agent_sync_state (
    agent_id TEXT PRIMARY KEY REFERENCES agents(id),
    last_synced_at TEXT,
    sync_status TEXT NOT NULL,
    error_message TEXT
);

CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    lifecycle_status TEXT NOT NULL CHECK (lifecycle_status IN ('active', 'archived', 'trashed')),
    head_version_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    trashed_at TEXT
);

CREATE TABLE IF NOT EXISTS skill_versions (
    id TEXT PRIMARY KEY,
    skill_id TEXT NOT NULL REFERENCES skills(id),
    version_number INTEGER NOT NULL,
    content_path TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (skill_id, version_number)
);

CREATE TABLE IF NOT EXISTS skill_assets (
    id TEXT PRIMARY KEY,
    skill_version_id TEXT NOT NULL REFERENCES skill_versions(id),
    relative_path TEXT NOT NULL,
    checksum_sha256 TEXT NOT NULL,
    UNIQUE (skill_version_id, relative_path)
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    transcript_path TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS favorites (
    user_id TEXT NOT NULL REFERENCES users(id),
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (user_id, entity_type, entity_id)
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS langdock_profiles (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    keychain_ref TEXT NOT NULL,
    last_tested_at TEXT,
    connection_status TEXT NOT NULL DEFAULT 'not_configured',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_prompts_owner_status ON prompts(owner_id, lifecycle_status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_agents_owner_status ON agents(owner_id, lifecycle_status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_skills_owner_status ON skills(owner_id, lifecycle_status, updated_at DESC);
