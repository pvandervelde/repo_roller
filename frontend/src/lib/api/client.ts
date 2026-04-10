/**
 * RepoRoller API client.
 *
 * All functions use same-origin fetch — the session cookie is sent automatically.
 * No Authorization header is required or set.
 *
 * Errors are thrown as typed subclasses of ApiError (see errors.ts).
 */
import {
  ApiAuthError,
  ApiConflictError,
  ApiNetworkError,
  ApiServerError,
  ApiValidationError,
} from './errors';
import type {
  CreateRepositoryRequest,
  CreateRepositoryResponse,
  ErrorResponse,
  GetTemplateDetailsResponse,
  ListRepositoryTypesResponse,
  ListTeamsResponse,
  ListTemplatesResponse,
  RepositoryTypeSummary,
  TeamSummary,
  TemplateSummary,
  TemplateVariable,
  ValidateRepositoryNameResponse,
} from './types';

// ---------------------------------------------------------------------------
// Internal types: raw shapes returned by the Rust backend
// ---------------------------------------------------------------------------

/** Raw variable definition from the backend (camelCase). */
interface RawVariableDefinition {
  description?: string;
  required: boolean;
  default?: string;
  pattern?: string;
}

/** Raw template details response before variable normalization. */
interface RawTemplateDetailsResponse {
  name: string;
  description: string;
  category?: string;
  repository_type?: { policy: string; type_name?: string | null } | null;
  /** Backend returns variables as a Record<name, RawVariableDefinition>. */
  variables?: Record<string, RawVariableDefinition> | TemplateVariable[];
  configuration?: unknown;
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

async function apiFetch(url: string, init?: RequestInit): Promise<Response> {
  try {
    return await fetch(url, init);
  } catch (err) {
    throw new ApiNetworkError(err);
  }
}

async function throwForStatus(response: Response): Promise<never> {
  let details = { code: 'UnknownError', message: `HTTP ${response.status}` };

  try {
    const body = (await response.json()) as ErrorResponse;
    if (body?.error) details = body.error;
  } catch {
    // Body was not valid JSON — keep the default details above
  }

  const status = response.status;

  if (status === 400 || status === 404) throw new ApiValidationError(status, details);
  if (status === 401 || status === 403) throw new ApiAuthError(status, details);
  if (status === 409 || status === 422) throw new ApiConflictError(status, details);
  throw new ApiServerError(status, details);
}

/**
 * Normalize the variables field from the backend's Record format into an array.
 * Accepts both the backend object format and a legacy array format (dev mocks).
 */
function normalizeVariables(
  raw: Record<string, RawVariableDefinition> | TemplateVariable[] | undefined,
): TemplateVariable[] {
  if (!raw) return [];
  if (Array.isArray(raw)) {
    // Already an array (should not happen with the real backend; guard for safety).
    return raw;
  }
  return Object.entries(raw).map(([name, def]) => ({
    name,
    description: def.description,
    required: def.required ?? false,
    default: def.default,
    pattern: def.pattern,
  }));
}

// ---------------------------------------------------------------------------
// Public API functions
// ---------------------------------------------------------------------------

/**
 * List all templates available for an organisation.
 * GET /api/v1/orgs/:org/templates
 */
export async function listTemplates(org: string): Promise<TemplateSummary[]> {
  const response = await apiFetch(`/api/v1/orgs/${encodeURIComponent(org)}/templates`, {
    method: 'GET',
  });
  if (!response.ok) await throwForStatus(response);
  const body = (await response.json()) as ListTemplatesResponse;
  return body.templates;
}

/**
 * Fetch full details (variables + configuration) for a single template.
 * GET /api/v1/orgs/:org/templates/:name
 *
 * Normalizes the backend's `variables` Record into a typed array before returning.
 */
export async function getTemplateDetails(
  org: string,
  name: string,
): Promise<GetTemplateDetailsResponse> {
  const response = await apiFetch(
    `/api/v1/orgs/${encodeURIComponent(org)}/templates/${encodeURIComponent(name)}`,
    { method: 'GET' },
  );
  if (!response.ok) await throwForStatus(response);
  const raw = (await response.json()) as RawTemplateDetailsResponse;
  return {
    name: raw.name,
    description: raw.description,
    category: raw.category,
    repository_type: raw.repository_type ?? null,
    variables: normalizeVariables(raw.variables),
  };
}

/**
 * Validate a repository name (format + availability) for an organisation.
 * POST /api/v1/repositories/validate-name
 * Always returns 200; `valid` = format check, `available` = uniqueness check.
 */
export async function validateRepositoryName(
  org: string,
  name: string,
): Promise<ValidateRepositoryNameResponse> {
  const response = await apiFetch('/api/v1/repositories/validate-name', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ organization: org, name }),
  });
  if (!response.ok) await throwForStatus(response);
  return response.json() as Promise<ValidateRepositoryNameResponse>;
}

/**
 * List all repository types configured for an organisation.
 * GET /api/v1/orgs/:org/repository-types
 */
export async function listRepositoryTypes(org: string): Promise<RepositoryTypeSummary[]> {
  const response = await apiFetch(`/api/v1/orgs/${encodeURIComponent(org)}/repository-types`, {
    method: 'GET',
  });
  if (!response.ok) await throwForStatus(response);
  const body = (await response.json()) as ListRepositoryTypesResponse;
  return body.types;
}

/**
 * List all GitHub organization teams.
 * GET /api/v1/orgs/:org/teams
 */
export async function listTeams(org: string): Promise<TeamSummary[]> {
  const response = await apiFetch(`/api/v1/orgs/${encodeURIComponent(org)}/teams`, {
    method: 'GET',
  });
  if (!response.ok) await throwForStatus(response);
  const body = (await response.json()) as ListTeamsResponse;
  return body.teams;
}

/**
 * Create a new repository.
 * POST /api/v1/repositories
 * Returns 201 on success; throws typed errors on failure.
 */
export async function createRepository(
  req: CreateRepositoryRequest,
): Promise<CreateRepositoryResponse> {
  const response = await apiFetch('/api/v1/repositories', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  if (!response.ok) await throwForStatus(response);
  return response.json() as Promise<CreateRepositoryResponse>;
}


