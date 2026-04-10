/**
 * TypeScript types for the RepoRoller REST API.
 *
 * Field names mirror the JSON shapes produced by the Rust API server.
 * Top-level response structs use camelCase (serde rename_all = "camelCase").
 * ErrorDetails is also camelCase (Rust ErrorResponse uses rename_all = "camelCase").
 *
 * See: docs/spec/interfaces/api-request-types.md
 *      docs/spec/interfaces/api-response-types.md
 *      docs/spec/interfaces/api-error-handling.md
 */

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

/** POST /api/v1/repositories */
export interface CreateRepositoryRequest {
  organization: string;
  name: string;
  template: string;
  visibility?: string;
  team?: string;
  repository_type?: string;
  variables?: Record<string, string>;
}

/** POST /api/v1/repositories/validate-name (body) */
export interface ValidateRepositoryNameRequestBody {
  organization: string;
  name: string;
}

// ---------------------------------------------------------------------------
// Responses
// ---------------------------------------------------------------------------

export interface RepositoryInfo {
  name: string;
  /** camelCase: matches Rust #[serde(rename_all = "camelCase")] */
  fullName: string;
  url: string;
  visibility: string;
  description?: string;
}

/** 201 response for POST /api/v1/repositories */
export interface CreateRepositoryResponse {
  repository: RepositoryInfo;
  /** camelCase: matches Rust appliedConfiguration field */
  appliedConfiguration: unknown;
  /** ISO 8601 creation timestamp */
  createdAt: string;
}

/** 200 response for POST /api/v1/repositories/validate-name (always 200) */
export interface ValidateRepositoryNameResponse {
  /** Whether the name passes format validation */
  valid: boolean;
  /** Whether the name is not already taken in the organisation */
  available: boolean;
  /** Validation messages when valid=false or available=false */
  messages?: string[];
}

export interface RepositoryTypePolicy {
  policy: string;
  type_name?: string | null;
}

/** A single template entry returned by GET /api/v1/orgs/:org/templates */
export interface TemplateSummary {
  name: string;
  description: string;
  /** Primary category tag for the template (first tag from backend config) */
  category?: string;
  /** Names of template variables (used to show variable-count badge) */
  variables: string[];
}

/** A single repository type entry. */
export interface RepositoryTypeSummary {
  name: string;
  description: string;
}

/** 200 response for GET /api/v1/orgs/:org/repository-types */
export interface ListRepositoryTypesResponse {
  types: RepositoryTypeSummary[];
}

/** A single organization team entry for the team dropdown. */
export interface TeamSummary {
  slug: string;
  name: string;
}

/** 200 response for GET /api/v1/orgs/:org/teams */
export interface ListTeamsResponse {
  teams: TeamSummary[];
}

/** 200 response for GET /api/v1/orgs/:org/templates */
export interface ListTemplatesResponse {
  templates: TemplateSummary[];
}

/**
 * Normalized template variable for UI consumption.
 * Derived from the backend's Record<string, VariableDefinition> by the API client.
 * Field names match the Rust VariableDefinition struct (camelCase via serde).
 */
export interface TemplateVariable {
  /** Variable key name */
  name: string;
  description?: string;
  required: boolean;
  /** Default value if not supplied (field name matches backend `default`) */
  default?: string;
  /** Optional regex pattern for client-side preview validation */
  pattern?: string;
}

/** 200 response for GET /api/v1/orgs/:org/templates/:name (normalized by client) */
export interface GetTemplateDetailsResponse {
  name: string;
  description: string;
  category?: string;
  /**
   * Repository type guidance from the template configuration.
   * Not guaranteed to be present — the backend may omit it for templates
   * that have no type constraint. The wizard defaults to 'optional' when absent.
   */
  repository_type?: RepositoryTypePolicy | null;
  /** Variables normalized from the backend's Record<name, VariableDefinition> */
  variables: TemplateVariable[];
}

// ---------------------------------------------------------------------------
// Errors  (camelCase — Rust ErrorResponse uses #[serde(rename_all = "camelCase")])
// ---------------------------------------------------------------------------

export interface ErrorDetails {
  code: string;
  message: string;
  details?: unknown;
}

export interface ErrorResponse {
  error: ErrorDetails;
}
