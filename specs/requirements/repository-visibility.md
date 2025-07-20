# Repository Visibility Requirements

## Overview

Repository visibility is a critical aspect of repository creation that determines who can access and interact with a repository. The RepoRoller system must handle repository visibility in a way that respects organizational policies, user preferences, and GitHub's constraints while providing clear, predictable behavior.

## Functional Requirements

### FR-1: Visibility Policy Hierarchy

The system SHALL implement a hierarchical visibility decision process with the following precedence (highest to lowest):

1. **Organization Security Policies** - Mandatory visibility constraints that cannot be overridden
2. **User/Team Preferences** - User-specified visibility settings within policy bounds
3. **Template Defaults** - Template-specified default visibility
4. **System Defaults** - Fallback visibility when no other preference is specified

### FR-2: Organization-Level Visibility Policies

#### FR-2.1: Policy Definition

- Organizations SHALL be able to define mandatory visibility policies through configuration
- Policies SHALL support three enforcement levels:
  - `required` - Forces a specific visibility (public/private/internal)
  - `restricted` - Prohibits specific visibility options
  - `unrestricted` - Allows any visibility choice

#### FR-2.2: Policy Application

- Organization policies SHALL be enforced regardless of user or template preferences
- Attempts to violate organization policies SHALL result in clear error messages
- The system SHALL provide policy validation before repository creation

### FR-3: User Visibility Preferences

#### FR-3.1: Preference Specification

- Users SHALL be able to specify repository visibility in creation requests
- Valid visibility options are: `public`, `private`, `internal` (GitHub Enterprise)
- Preference SHALL be validated against organization policies before application

#### FR-3.2: Default Behavior

- When no user preference is specified, the system SHALL fall back to template defaults
- When both user preference and template defaults are missing, system defaults apply

### FR-4: Template Visibility Defaults

#### FR-4.1: Template Configuration

- Templates SHALL be able to specify default visibility settings
- Template visibility defaults SHALL be documented in template metadata
- Template defaults SHALL be overridable by user preferences (unless blocked by org policy)

### FR-5: GitHub Enterprise Internal Repositories

#### FR-5.1: Internal Repository Support

- The system SHALL detect GitHub Enterprise environments
- Internal repository visibility SHALL only be offered in Enterprise environments
- Attempts to create internal repositories on github.com SHALL be rejected with clear error

### FR-6: Visibility Validation

#### FR-6.1: Pre-Creation Validation

- Repository visibility SHALL be validated before any API calls to GitHub
- Validation SHALL check organization policies, GitHub constraints, and user permissions
- Invalid visibility configurations SHALL prevent repository creation

#### FR-6.2: Real-Time Policy Checking

- The system SHALL validate visibility against current organization settings
- Cached organization policies SHALL be refreshed when validation fails
- Stale policy information SHALL not cause creation failures

## Non-Functional Requirements

### NFR-1: Performance

- Visibility determination SHALL complete within 200ms for cached policies
- Policy cache misses SHALL resolve within 2 seconds
- Organization policy validation SHALL not significantly impact creation performance

### NFR-2: Reliability

- Visibility validation SHALL have 99.9% success rate for valid configurations
- The system SHALL gracefully handle GitHub API rate limits during policy checking
- Temporary GitHub API failures SHALL not prevent visibility determination

### NFR-3: Security

- Organization visibility policies SHALL be enforced at all times
- Policy bypass attempts SHALL be logged and audited
- Sensitive repository visibility decisions SHALL be recorded in audit logs

### NFR-4: Usability

- Visibility validation errors SHALL provide actionable guidance
- Available visibility options SHALL be clearly communicated to users
- Policy conflicts SHALL be explained in user-friendly terms

## Edge Cases

### EC-1: Organization Policy Changes

- Handle cases where organization policies change between user request and repository creation
- Detect and handle policy conflicts during multi-step creation process
- Provide clear error messages when policies change mid-creation

### EC-2: GitHub API Limitations

- Handle rate limiting during visibility validation
- Manage API errors when checking organization settings
- Fallback behavior when GitHub Enterprise detection fails

### EC-3: Cross-Organization Templates

- Handle visibility constraints when templates are in different organizations
- Validate visibility policies across organization boundaries
- Clear error messages for unsupported cross-org visibility scenarios

### EC-4: Repository Type Constraints

- Handle visibility limitations for specific repository types (e.g., GitHub Pages)
- Validate visibility against repository features and GitHub plan limitations
- Error handling for plan-specific visibility restrictions

## Acceptance Criteria

### AC-1: Policy Enforcement

- Organization policies are enforced in 100% of repository creation attempts
- Policy violations are detected before any GitHub API calls are made
- Clear error messages are provided for all policy violations

### AC-2: User Experience

- Users receive immediate feedback on visibility option availability
- Error messages clearly explain visibility constraints and alternatives
- Documentation clearly explains visibility behavior and policies

### AC-3: Template Integration

- Template visibility defaults are respected when no user preference is specified
- Template defaults are properly overridden by user preferences (when allowed)
- Template visibility configuration is validated during template registration

### AC-4: GitHub Integration

- Repository visibility is correctly set via GitHub API
- GitHub Enterprise internal repositories are properly supported
- GitHub plan limitations are respected and communicated

## Behavioral Assertions

1. Repository visibility must never violate organization security policies regardless of user or template preferences
2. Visibility validation must complete before any repository creation API calls are made
3. Internal repository visibility must only be available in GitHub Enterprise environments
4. Organization policy changes must not affect repositories already in the creation process
5. Visibility determination must be deterministic - same inputs always produce same outputs
6. Cache invalidation must not cause repository creation failures
7. All visibility policy violations must be logged for security auditing
8. GitHub API rate limits must not prevent visibility validation for legitimate requests
9. Template visibility defaults must be validated against organization policies at template registration time
10. Cross-organization visibility conflicts must be detected and reported clearly
