# Non-Template Repository Support Requirements

## Overview

Non-template repository support provides users with the ability to create new repositories without using any predefined template. This feature encompasses two distinct creation modes: truly empty repositories with no files whatsoever, and custom-initialized repositories that include user-defined files but weren't created from a template. Both modes enable rapid repository creation while still benefiting from organization-specific repository settings and configurations.

## Repository Creation Modes

### Mode 1: Empty Repository Creation

Repositories created without any files, providing a completely blank slate for users to build upon.

### Mode 2: Custom-Initialized Repository Creation

Repositories created without templates but with user-selected initialization files (README, .gitignore, LICENSE, etc.) to provide a customized starting point.

## Functional Requirements

### FR-1: Non-Template Repository Creation

#### FR-1.1: Template-Free Repository Creation

- Users SHALL be able to create repositories without specifying any template
- The system SHALL support two distinct creation modes: empty and custom-initialized
- Users SHALL explicitly choose between empty and custom-initialized creation modes
- The system SHALL support non-template repository creation through all interfaces (CLI, API, MCP)
- Non-template repositories SHALL still respect organization-specific repository settings
- Non-template repositories SHALL still apply repository type configurations if specified
- The system SHALL provide clear indication of the creation mode being used

#### FR-1.2: Empty Repository Mode

- Users SHALL be able to create repositories with zero files
- Empty repositories SHALL have no initial commit when truly empty
- Empty repositories SHALL still apply organization settings (labels, webhooks, branch protection, etc.)
- Empty repositories SHALL respect repository type configurations if specified
- Empty repositories SHALL be clearly identified as empty in creation logs and audit trails
- Users SHALL be able to specify basic repository metadata (name, description, visibility) for empty repositories

#### FR-1.3: Custom-Initialized Repository Mode

- Users SHALL be able to create repositories with user-selected initialization files
- Users SHALL be able to choose specific files to include:
  - README.md file (optional, configurable content)
  - .gitignore file (based on language/framework selection)
  - LICENSE file (from organization-approved licenses)
  - Custom files with user-provided content
- The system SHALL create an initial commit containing only the selected files
- Users SHALL be able to provide custom content for README.md files
- Users SHALL be able to select from predefined .gitignore templates or provide custom content
- Users SHALL be able to select from organization-approved LICENSE files
- All initialization files SHALL be optional and user-controllable
- The system SHALL validate custom file content for security and policy compliance

### FR-2: Configuration Integration

#### FR-2.1: Organization Settings Application

- Non-template repositories SHALL respect all organization-specific repository settings
- Team-specific configurations SHALL apply to non-template repositories when team is specified
- Repository type configurations SHALL apply to non-template repositories when type is specified
- Override policies SHALL be enforced for non-template repositories same as templated repositories
- Security policies SHALL be applied consistently regardless of template usage
- Configuration application SHALL be identical for both empty and custom-initialized repositories

#### FR-2.2: Custom Properties and Labels

- GitHub custom properties SHALL be applied to non-template repositories
- Organization-standard labels SHALL be added to non-template repositories
- Repository type-specific labels SHALL be applied when repository type is specified
- All custom property and label application SHALL follow the same hierarchy as templated repositories
- Custom property and label application SHALL occur regardless of repository creation mode

### FR-3: User Experience

#### FR-3.1: Interface Consistency

- Non-template repository creation SHALL be available through the same interfaces as template-based creation
- Command-line interface SHALL support `--empty` flag for truly empty repositories
- Command-line interface SHALL support `--custom-init` flag for custom-initialized repositories
- API endpoints SHALL support both creation modes through dedicated endpoints or mode parameters
- MCP interface SHALL support both creation modes through appropriate methods
- Web interface (if applicable) SHALL provide clear options for both creation modes
- Users SHALL receive clear confirmation of which creation mode was used

#### FR-3.2: Workflow Integration

- Non-template repository creation SHALL integrate seamlessly with existing repository creation workflows
- Post-creation actions (webhooks, notifications, CI/CD setup) SHALL work for non-template repositories
- Repository validation and compliance checks SHALL apply to non-template repositories
- Audit logging SHALL capture repository creation events with creation mode metadata
- Creation mode SHALL be clearly distinguishable in all logging and reporting systems

### FR-4: Validation and Constraints

#### FR-4.1: Input Validation

- Repository names SHALL be validated according to GitHub requirements and organization policies
- Repository descriptions SHALL be validated for content policies if applicable
- Visibility settings SHALL be validated against organization visibility policies
- Team assignments SHALL be validated against user permissions and team membership

#### FR-4.2: Compliance and Governance

- Empty repositories SHALL be subject to the same compliance checks as templated repositories
- Organization governance policies SHALL apply to empty repositories
- Repository creation quotas and limits SHALL apply to empty repositories
- Security scanning and policy enforcement SHALL be enabled for empty repositories

## Non-Functional Requirements

### NFR-1: Performance

- Empty repository creation SHALL complete within the same time constraints as template-based creation
- The absence of template processing SHALL not negatively impact overall creation performance
- Repository initialization options SHALL not significantly increase creation time
- Bulk empty repository creation SHALL be supported efficiently

### NFR-2: Reliability

- Empty repository creation SHALL have the same reliability standards as template-based creation
- Partial creation failures SHALL be handled gracefully with appropriate rollback
- Retry mechanisms SHALL work consistently for empty repository creation
- Error reporting SHALL be clear and actionable for empty repository creation failures

### NFR-3: Security

- Empty repositories SHALL be subject to the same security controls as templated repositories
- Repository access controls SHALL be applied immediately upon creation
- Security policies SHALL be enforced consistently regardless of template usage
- Audit trails SHALL capture all relevant security events for empty repositories

## Technical Requirements

### TR-1: Implementation Integration

- Empty repository support SHALL integrate with existing repository creation infrastructure
- Configuration management SHALL handle empty repositories without template-specific code paths
- GitHub API interactions SHALL be optimized for empty repository scenarios
- Caching strategies SHALL account for empty repository creation patterns

### TR-2: Data Handling

- Empty repository metadata SHALL be stored consistently with templated repository metadata
- Repository type assignments SHALL be persisted properly for empty repositories
- Configuration applications SHALL be logged and traceable for empty repositories
- Creation history SHALL distinguish between empty and templated repository creation

## Edge Cases

### EC-1: Organization Policy Conflicts

- When organization policies require certain files (e.g., CODE_OF_CONDUCT.md), the system SHALL:
  - Add required files automatically to empty repositories
  - Provide clear indication of policy-driven file additions
  - Allow users to customize policy-required file contents where permitted
  - Reject empty repository creation if required files cannot be satisfied

### EC-2: Repository Type Requirements

- When repository types require specific files or structure, the system SHALL:
  - Add type-required files to empty repositories
  - Provide clear indication of type-driven additions
  - Validate that empty repository creation is compatible with specified repository type
  - Offer alternative repository types if conflicts exist

### EC-3: Team and Permission Scenarios

- When teams have specific repository requirements, the system SHALL:
  - Apply team-specific requirements to empty repositories
  - Validate team permissions for empty repository creation
  - Handle team membership changes after empty repository creation
  - Ensure team-specific security policies are enforced

## Acceptance Criteria

### AC-1: Core Functionality

1. Users can create repositories without specifying any template
2. Empty repositories respect all organization-specific settings
3. Repository type configurations apply correctly to empty repositories
4. All standard repository creation interfaces support empty repository creation
5. Post-creation workflows function correctly for empty repositories

### AC-2: Configuration and Compliance

1. Organization policies are enforced consistently for empty repositories
2. Security controls are applied immediately upon empty repository creation
3. Audit logging captures all relevant events for empty repository creation
4. Repository validation and compliance checks work for empty repositories

### AC-3: User Experience

1. Empty repository creation is intuitive and well-documented
2. Error messages are clear and actionable for empty repository scenarios
3. Creation status and progress are communicated clearly to users
4. Performance meets established benchmarks for repository creation

## Behavioral Assertions

1. Empty repository creation must respect all organization-specific repository settings
2. Repository type configurations must apply to empty repositories when specified
3. Security policies must be enforced consistently regardless of template usage
4. Organization-required files must be added automatically to empty repositories
5. Empty repository creation must complete within established time limits
6. Post-creation actions must execute for empty repositories same as templated repositories
7. Audit trails must capture empty repository creation with complete metadata
8. Team-specific configurations must apply to empty repositories when team is specified
9. Repository validation must occur for empty repositories before creation completion
10. Custom properties and labels must be applied to empty repositories following hierarchy rules
11. Empty repository creation failures must trigger appropriate rollback and cleanup
12. Repository type validation must prevent incompatible empty repository creation
