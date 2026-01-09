//! Tests for visibility resolution logic.

use super::*;
use async_trait::async_trait;
use config_manager::{
    RepositoryVisibility, VisibilityError, VisibilityPolicy, VisibilityPolicyProvider,
};
use std::sync::Arc;

/// Mock policy provider for testing.
struct MockPolicyProvider {
    policy: VisibilityPolicy,
}

impl MockPolicyProvider {
    fn new(policy: VisibilityPolicy) -> Self {
        Self { policy }
    }

    fn unrestricted() -> Self {
        Self::new(VisibilityPolicy::Unrestricted)
    }

    fn required(visibility: RepositoryVisibility) -> Self {
        Self::new(VisibilityPolicy::Required(visibility))
    }

    fn restricted(visibilities: Vec<RepositoryVisibility>) -> Self {
        Self::new(VisibilityPolicy::Restricted(visibilities))
    }
}

#[async_trait]
impl VisibilityPolicyProvider for MockPolicyProvider {
    async fn get_policy(&self, _organization: &str) -> Result<VisibilityPolicy, VisibilityError> {
        Ok(self.policy.clone())
    }

    async fn invalidate_cache(&self, _organization: &str) {
        // No-op for mock
    }
}

/// Mock environment detector for testing.
struct MockEnvironmentDetector {
    limitations: PlanLimitations,
}

impl MockEnvironmentDetector {
    fn new(limitations: PlanLimitations) -> Self {
        Self { limitations }
    }

    fn free_plan() -> Self {
        Self::new(PlanLimitations::free_plan())
    }

    fn paid_plan() -> Self {
        Self::new(PlanLimitations::paid_plan())
    }

    fn enterprise() -> Self {
        Self::new(PlanLimitations::enterprise())
    }
}

#[async_trait]
impl GitHubEnvironmentDetector for MockEnvironmentDetector {
    async fn get_plan_limitations(
        &self,
        _organization: &str,
    ) -> Result<PlanLimitations, github_client::Error> {
        Ok(self.limitations.clone())
    }

    async fn is_enterprise(&self, _organization: &str) -> Result<bool, github_client::Error> {
        Ok(self.limitations.is_enterprise)
    }
}

/// Test that organization Required policy rejects conflicting user preference.
#[tokio::test]
async fn test_required_policy_rejects_conflicting_user_preference() {
    let policy_provider = Arc::new(MockPolicyProvider::required(RepositoryVisibility::Private));
    let env_detector = Arc::new(MockEnvironmentDetector::paid_plan());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: Some(RepositoryVisibility::Public),
        template_default: None,
    };

    let result = resolver.resolve_visibility(request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        VisibilityError::PolicyViolation { requested, .. } => {
            assert_eq!(requested, RepositoryVisibility::Public);
        }
        _ => panic!("Expected PolicyViolation error"),
    }
}

/// Test that organization Required policy allows compliant user preference.
#[tokio::test]
async fn test_required_policy_allows_compliant_preference() {
    let policy_provider = Arc::new(MockPolicyProvider::required(RepositoryVisibility::Private));
    let env_detector = Arc::new(MockEnvironmentDetector::paid_plan());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: Some(RepositoryVisibility::Private),
        template_default: None,
    };

    let decision = resolver.resolve_visibility(request).await.unwrap();

    assert_eq!(decision.visibility, RepositoryVisibility::Private);
    assert_eq!(decision.source, DecisionSource::OrganizationPolicy);
    assert!(decision
        .constraints_applied
        .contains(&PolicyConstraint::OrganizationRequired));
}

/// Test that organization Required policy applies when no preference specified.
#[tokio::test]
async fn test_required_policy_applies_when_no_preference() {
    let policy_provider = Arc::new(MockPolicyProvider::required(RepositoryVisibility::Private));
    let env_detector = Arc::new(MockEnvironmentDetector::paid_plan());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: None,
        template_default: None,
    };

    let decision = resolver.resolve_visibility(request).await.unwrap();

    assert_eq!(decision.visibility, RepositoryVisibility::Private);
    assert_eq!(decision.source, DecisionSource::OrganizationPolicy);
    assert!(decision
        .constraints_applied
        .contains(&PolicyConstraint::OrganizationRequired));
}

/// Test that Restricted policy blocks prohibited visibility.
#[tokio::test]
async fn test_restricted_policy_blocks_public() {
    let policy_provider = Arc::new(MockPolicyProvider::restricted(vec![
        RepositoryVisibility::Public,
    ]));
    let env_detector = Arc::new(MockEnvironmentDetector::paid_plan());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: Some(RepositoryVisibility::Public),
        template_default: None,
    };

    let result = resolver.resolve_visibility(request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        VisibilityError::PolicyViolation { requested, .. } => {
            assert_eq!(requested, RepositoryVisibility::Public);
        }
        _ => panic!("Expected PolicyViolation error"),
    }
}

/// Test that Restricted policy allows non-restricted visibility.
#[tokio::test]
async fn test_restricted_policy_allows_private() {
    let policy_provider = Arc::new(MockPolicyProvider::restricted(vec![
        RepositoryVisibility::Public,
    ]));
    let env_detector = Arc::new(MockEnvironmentDetector::paid_plan());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: Some(RepositoryVisibility::Private),
        template_default: None,
    };

    let decision = resolver.resolve_visibility(request).await.unwrap();

    assert_eq!(decision.visibility, RepositoryVisibility::Private);
    assert_eq!(decision.source, DecisionSource::UserPreference);
}

/// Test that user preference is respected when policy allows.
#[tokio::test]
async fn test_user_preference_respected() {
    let policy_provider = Arc::new(MockPolicyProvider::unrestricted());
    let env_detector = Arc::new(MockEnvironmentDetector::paid_plan());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: Some(RepositoryVisibility::Public),
        template_default: Some(RepositoryVisibility::Private),
    };

    let decision = resolver.resolve_visibility(request).await.unwrap();

    assert_eq!(decision.visibility, RepositoryVisibility::Public);
    assert_eq!(decision.source, DecisionSource::UserPreference);
}

/// Test that template default is used when no user preference.
#[tokio::test]
async fn test_template_default_used() {
    let policy_provider = Arc::new(MockPolicyProvider::unrestricted());
    let env_detector = Arc::new(MockEnvironmentDetector::paid_plan());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: None,
        template_default: Some(RepositoryVisibility::Internal),
    };

    // Should fail because Internal requires Enterprise
    let result = resolver.resolve_visibility(request).await;
    assert!(result.is_err());
}

/// Test that template default is used when it's valid for the plan.
#[tokio::test]
async fn test_template_default_used_with_enterprise() {
    let policy_provider = Arc::new(MockPolicyProvider::unrestricted());
    let env_detector = Arc::new(MockEnvironmentDetector::enterprise());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: None,
        template_default: Some(RepositoryVisibility::Internal),
    };

    let decision = resolver.resolve_visibility(request).await.unwrap();

    assert_eq!(decision.visibility, RepositoryVisibility::Internal);
    assert_eq!(decision.source, DecisionSource::TemplateDefault);
}

/// Test that system default is used when no preferences.
#[tokio::test]
async fn test_system_default_used() {
    let policy_provider = Arc::new(MockPolicyProvider::unrestricted());
    let env_detector = Arc::new(MockEnvironmentDetector::paid_plan());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: None,
        template_default: None,
    };

    let decision = resolver.resolve_visibility(request).await.unwrap();

    assert_eq!(decision.visibility, RepositoryVisibility::Private);
    assert_eq!(decision.source, DecisionSource::SystemDefault);
}

/// Test that Internal visibility is rejected on non-Enterprise.
#[tokio::test]
async fn test_internal_rejected_on_non_enterprise() {
    let policy_provider = Arc::new(MockPolicyProvider::unrestricted());
    let env_detector = Arc::new(MockEnvironmentDetector::paid_plan());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: Some(RepositoryVisibility::Internal),
        template_default: None,
    };

    let result = resolver.resolve_visibility(request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        VisibilityError::GitHubConstraint { requested, reason } => {
            assert_eq!(requested, RepositoryVisibility::Internal);
            assert!(reason.contains("Enterprise"));
        }
        _ => panic!("Expected GitHubConstraint error"),
    }
}

/// Test that Internal visibility is allowed on Enterprise.
#[tokio::test]
async fn test_internal_allowed_on_enterprise() {
    let policy_provider = Arc::new(MockPolicyProvider::unrestricted());
    let env_detector = Arc::new(MockEnvironmentDetector::enterprise());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: Some(RepositoryVisibility::Internal),
        template_default: None,
    };

    let decision = resolver.resolve_visibility(request).await.unwrap();

    assert_eq!(decision.visibility, RepositoryVisibility::Internal);
    assert_eq!(decision.source, DecisionSource::UserPreference);
    assert!(decision
        .constraints_applied
        .contains(&PolicyConstraint::RequiresEnterprise));
}

/// Test that constraints are tracked correctly.
#[tokio::test]
async fn test_constraints_tracked() {
    let policy_provider = Arc::new(MockPolicyProvider::restricted(vec![
        RepositoryVisibility::Public,
    ]));
    let env_detector = Arc::new(MockEnvironmentDetector::enterprise());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: Some(RepositoryVisibility::Internal),
        template_default: None,
    };

    let decision = resolver.resolve_visibility(request).await.unwrap();

    assert_eq!(decision.visibility, RepositoryVisibility::Internal);
    assert!(decision
        .constraints_applied
        .contains(&PolicyConstraint::OrganizationRestricted));
    assert!(decision
        .constraints_applied
        .contains(&PolicyConstraint::RequiresEnterprise));
}

/// Test that Public visibility works on free plan.
#[tokio::test]
async fn test_public_on_free_plan() {
    let policy_provider = Arc::new(MockPolicyProvider::unrestricted());
    let env_detector = Arc::new(MockEnvironmentDetector::free_plan());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: Some(RepositoryVisibility::Public),
        template_default: None,
    };

    let decision = resolver.resolve_visibility(request).await.unwrap();

    assert_eq!(decision.visibility, RepositoryVisibility::Public);
    assert_eq!(decision.source, DecisionSource::UserPreference);
}

/// Test that Private visibility works on free plan.
#[tokio::test]
async fn test_private_on_free_plan() {
    let policy_provider = Arc::new(MockPolicyProvider::unrestricted());
    let env_detector = Arc::new(MockEnvironmentDetector::free_plan());
    let resolver = VisibilityResolver::new(policy_provider, env_detector);

    let request = VisibilityRequest {
        organization: OrganizationName::new("test-org").unwrap(),
        user_preference: Some(RepositoryVisibility::Private),
        template_default: None,
    };

    let decision = resolver.resolve_visibility(request).await.unwrap();

    assert_eq!(decision.visibility, RepositoryVisibility::Private);
    assert_eq!(decision.source, DecisionSource::UserPreference);
}
