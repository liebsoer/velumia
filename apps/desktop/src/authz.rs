use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    pub user_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    PromptRead,
    PromptWrite,
    PromptExecute,
    AgentRead,
    AgentWrite,
    AgentExecute,
    SkillRead,
    SkillWrite,
    SkillExport,
    LibraryImport,
    LibraryExport,
    CredentialWrite,
    SettingsRead,
    SettingsWrite,
    Admin,
}

impl Permission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromptRead => "prompt:read",
            Self::PromptWrite => "prompt:write",
            Self::PromptExecute => "prompt:execute",
            Self::AgentRead => "agent:read",
            Self::AgentWrite => "agent:write",
            Self::AgentExecute => "agent:execute",
            Self::SkillRead => "skill:read",
            Self::SkillWrite => "skill:write",
            Self::SkillExport => "skill:export",
            Self::LibraryImport => "library:import",
            Self::LibraryExport => "library:export",
            Self::CredentialWrite => "credential:write",
            Self::SettingsRead => "settings:read",
            Self::SettingsWrite => "settings:write",
            Self::Admin => "admin",
        }
    }

    pub fn from_action(action: &str) -> Option<Self> {
        match action {
            "prompt:read" => Some(Self::PromptRead),
            "prompt:write" => Some(Self::PromptWrite),
            "prompt:execute" => Some(Self::PromptExecute),
            "agent:read" => Some(Self::AgentRead),
            "agent:write" => Some(Self::AgentWrite),
            "agent:execute" => Some(Self::AgentExecute),
            "skill:read" => Some(Self::SkillRead),
            "skill:write" => Some(Self::SkillWrite),
            "skill:export" => Some(Self::SkillExport),
            "library:import" => Some(Self::LibraryImport),
            "library:export" => Some(Self::LibraryExport),
            "credential:write" => Some(Self::CredentialWrite),
            "settings:read" => Some(Self::SettingsRead),
            "settings:write" => Some(Self::SettingsWrite),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "allowed", rename_all = "snake_case")]
pub enum AuthzResult {
    #[serde(rename = "true")]
    Allowed,
    #[serde(rename = "false")]
    Denied {
        reason: String,
        permission: String,
    },
}

pub fn authorize(principal: &Principal, permission: Permission) -> AuthzResult {
    if env::var("VELUMIA_AUTHZ_STUB_DENY").ok().as_deref() == Some("1")
        && permission == Permission::PromptWrite
    {
        return AuthzResult::Denied {
            reason: "permission_denied".into(),
            permission: permission.as_str().into(),
        };
    }

    if env::var("VELUMIA_AUTHZ_STUB_DENY_EXECUTE").ok().as_deref() == Some("1")
        && permission == Permission::PromptExecute
    {
        return AuthzResult::Denied {
            reason: "permission_denied".into(),
            permission: permission.as_str().into(),
        };
    }

    if principal.user_id.is_empty() {
        return AuthzResult::Denied {
            reason: "permission_denied".into(),
            permission: permission.as_str().into(),
        };
    }

    AuthzResult::Allowed
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn owner_allowed_by_default() {
        let p = Principal {
            user_id: "user-1".into(),
        };
        assert!(matches!(
            authorize(&p, Permission::CredentialWrite),
            AuthzResult::Allowed
        ));
    }

    #[test]
    #[serial]
    fn stub_deny_prompt_write() {
        unsafe { env::set_var("VELUMIA_AUTHZ_STUB_DENY", "1") };
        let p = Principal {
            user_id: "user-1".into(),
        };
        assert!(matches!(
            authorize(&p, Permission::PromptWrite),
            AuthzResult::Denied { .. }
        ));
        unsafe { env::remove_var("VELUMIA_AUTHZ_STUB_DENY") };
    }

    #[test]
    #[serial]
    fn stub_deny_prompt_execute() {
        unsafe { env::set_var("VELUMIA_AUTHZ_STUB_DENY_EXECUTE", "1") };
        let p = Principal {
            user_id: "user-1".into(),
        };
        assert!(matches!(
            authorize(&p, Permission::PromptExecute),
            AuthzResult::Denied { .. }
        ));
        unsafe { env::remove_var("VELUMIA_AUTHZ_STUB_DENY_EXECUTE") };
    }
}
