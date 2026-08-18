# Feature Flagging

LaunchLightly controls which behavior an application serves without requiring a new deployment.

## Language

**Feature flag**:
A project-scoped Boolean decision identified by a stable key. Its identity and descriptive information are shared by every environment.
_Avoid_: Environment flag, toggle

**Environment flag configuration**:
The settings that determine how one feature flag behaves in one environment.
_Avoid_: Variation, flag state

**Context**:
The subject for which a feature flag is evaluated, identified by a stable key.
_Avoid_: User, when the subject may be something else

**Off value**:
The value served when an environment flag configuration is disabled.

**Fallthrough value**:
The value served when a configuration is enabled and the context is not explicitly targeted.
