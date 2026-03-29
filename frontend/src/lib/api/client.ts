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
  ListTemplatesResponse,
  TemplateSummary,
  ValidateRepositoryNameResponse,
} from './types';

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
 * Fetch full details (metadata + variables) for a single template.
 * GET /api/v1/orgs/:org/templates/:name
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
  return response.json() as Promise<GetTemplateDetailsResponse>;
}

/**
 * Validate a repository name (format + availability) for an organisation.
 * POST /api/v1/repositories/validate-name
 * Always returns 200 — `valid: false` for invalid names, never throws on validation failures.
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
