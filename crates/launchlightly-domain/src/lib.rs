use std::collections::HashMap;

use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FlagId(pub Uuid);

impl FlagId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for FlagId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProjectId(pub Uuid);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EnvironmentId(pub Uuid);

impl EnvironmentId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for EnvironmentId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FlagKey(pub String);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ContextKey(pub String);

impl From<&str> for ContextKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for ContextKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeatureFlag {
    pub id: FlagId,
    pub project_id: ProjectId,
    pub key: FlagKey,
    pub name: String,
    pub description: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub archived_at: Option<OffsetDateTime>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentFeatureFlagConfig {
    pub flag_id: FlagId,
    pub environment_id: EnvironmentId,
    pub enabled: bool,
    pub off_value: bool,
    pub fallthrough_value: bool,
    pub targets: HashMap<ContextKey, bool>,
}

impl EnvironmentFeatureFlagConfig {
    #[must_use]
    pub fn evaluate(&self, context: &ContextKey) -> bool {
        if !self.enabled {
            return self.off_value;
        }

        self.targets
            .get(context)
            .copied()
            .unwrap_or(self.fallthrough_value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn disabled_flag_returns_its_off_value_even_for_a_targeted_context() {
        let context = ContextKey::from("beta-user");
        let config = EnvironmentFeatureFlagConfig {
            flag_id: FlagId::new(),
            environment_id: EnvironmentId::new(),
            enabled: false,
            off_value: true,
            fallthrough_value: false,
            targets: HashMap::from([(context.clone(), false)]),
        };

        assert!(config.evaluate(&context));
    }

    #[test]
    fn enabled_flag_returns_the_value_for_an_exact_target() {
        let context = ContextKey::from("beta-user");
        let config = EnvironmentFeatureFlagConfig {
            flag_id: FlagId::new(),
            environment_id: EnvironmentId::new(),
            enabled: true,
            off_value: false,
            fallthrough_value: false,
            targets: HashMap::from([(context.clone(), true)]),
        };

        assert!(config.evaluate(&context));
    }

    #[test]
    fn enabled_flag_returns_its_fallthrough_value_for_an_unknown_context() {
        let config = EnvironmentFeatureFlagConfig {
            flag_id: FlagId::new(),
            environment_id: EnvironmentId::new(),
            enabled: true,
            off_value: false,
            fallthrough_value: true,
            targets: HashMap::new(),
        };

        assert!(config.evaluate(&ContextKey::from("unknown-user")));
    }
}
