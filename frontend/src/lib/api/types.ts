/**
 * TypeScript types for the RepoRoller REST API.
 *
 * Field names mirror the JSON shapes produced by the Rust API server.
 * Most structs use snake_case (no serde rename_all); ErrorDetails is camelCase
 * because the Rust ErrorResponse uses #[serde(rename_all = "camelCase")].
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
    full_name: string;
    url: string;
    visibility: string;
    repository_type?: string;
    created_at: string;
}

export interface AppliedConfiguration {
    applied_settings: Record<string, unknown>;
    sources: Record<string, string>;
}

/** 201 response for POST /api/v1/repositories */
export interface CreateRepositoryResponse {
    repository: RepositoryInfo;
    configuration: AppliedConfiguration;
}

export interface ValidationIssue {
    field: string;
    message: string;
    constraint: string;
}

/** 200 response for POST /api/v1/repositories/validate-name (always 200) */
export interface ValidateRepositoryNameResponse {
    valid: boolean;
    name: string;
    errors?: ValidationIssue[];
}

export interface RepositoryTypePolicy {
    policy: string;
    type_name?: string;
}

export interface TemplateSummary {
    name: string;
    description: string;
    author?: string;
    tags: string[];
    repository_type?: RepositoryTypePolicy;
}

/** 200 response for GET /api/v1/orgs/:org/templates */
export interface ListTemplatesResponse {
    templates: TemplateSummary[];
}

export interface TemplateMetadata {
    author?: string;
    description: string;
    tags: string[];
}

export interface TemplateVariable {
    name: string;
    description?: string;
    required: boolean;
    default_value?: string;
}

/** 200 response for GET /api/v1/orgs/:org/templates/:name */
export interface GetTemplateDetailsResponse {
    name: string;
    metadata: TemplateMetadata;
    repository_type?: RepositoryTypePolicy;
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
