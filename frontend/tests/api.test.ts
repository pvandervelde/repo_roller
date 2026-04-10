import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  listTemplates,
  getTemplateDetails,
  validateRepositoryName,
  createRepository,
  listTeams,
} from '../src/lib/api/client';
import {
  ApiAuthError,
  ApiConflictError,
  ApiNetworkError,
  ApiServerError,
  ApiValidationError,
} from '../src/lib/api/errors';
import type {
  CreateRepositoryResponse,
  ListTeamsResponse,
  ListTemplatesResponse,
  ValidateRepositoryNameResponse,
} from '../src/lib/api/types';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function mockFetch(body: unknown, status = 200): void {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify(body), {
        status,
        headers: { 'Content-Type': 'application/json' },
      }),
    ),
  );
}

function mockFetchNetworkError(cause: unknown = new Error('Failed to fetch')): void {
  vi.stubGlobal('fetch', vi.fn().mockRejectedValueOnce(cause));
}

function errorBody(code: string, message: string) {
  return { error: { code, message } };
}

beforeEach(() => {
  vi.unstubAllGlobals();
});

// ---------------------------------------------------------------------------
// listTemplates
// ---------------------------------------------------------------------------

describe('listTemplates()', () => {
  const TEMPLATES_RESPONSE: ListTemplatesResponse = {
    templates: [
      {
        name: 'rust-library',
        description: 'Rust library template',
        category: 'library',
        variables: [],
      },
      {
        name: 'python-service',
        description: 'Python service template',
        variables: [],
      },
    ],
  };

  it('calls GET /api/v1/orgs/:org/templates', async () => {
    mockFetch(TEMPLATES_RESPONSE);
    await listTemplates('myorg');
    expect(vi.mocked(fetch)).toHaveBeenCalledWith(
      '/api/v1/orgs/myorg/templates',
      expect.objectContaining({ method: 'GET' }),
    );
  });

  it('returns the templates array from the response', async () => {
    mockFetch(TEMPLATES_RESPONSE);
    const result = await listTemplates('myorg');
    expect(result).toHaveLength(2);
    expect(result[0].name).toBe('rust-library');
    expect(result[1].name).toBe('python-service');
  });

  it('throws ApiServerError on 500', async () => {
    mockFetch(errorBody('SystemError', 'Internal server error'), 500);
    await expect(listTemplates('myorg')).rejects.toThrow(ApiServerError);
  });

  it('throws ApiAuthError on 401', async () => {
    mockFetch(errorBody('AuthenticationError', 'Session expired'), 401);
    await expect(listTemplates('myorg')).rejects.toThrow(ApiAuthError);
  });

  it('throws ApiNetworkError when fetch rejects', async () => {
    mockFetchNetworkError();
    await expect(listTemplates('myorg')).rejects.toThrow(ApiNetworkError);
  });
});

// ---------------------------------------------------------------------------
// getTemplateDetails
// ---------------------------------------------------------------------------

describe('getTemplateDetails()', () => {
  const DETAILS_RESPONSE = {
    name: 'rust-library',
    description: 'Rust library with CI/CD',
    category: 'library',
    repository_type: { policy: 'fixed', type_name: 'library' },
    // Backend returns variables as a Record; the client normalizes to an array.
    variables: {
      project_name: { description: 'Project name', required: true },
      author: { description: 'Author', required: false, default: 'Engineering Team' },
    },
    configuration: {},
  };

  it('calls GET /api/v1/orgs/:org/templates/:name', async () => {
    mockFetch(DETAILS_RESPONSE);
    await getTemplateDetails('myorg', 'rust-library');
    expect(vi.mocked(fetch)).toHaveBeenCalledWith(
      '/api/v1/orgs/myorg/templates/rust-library',
      expect.objectContaining({ method: 'GET' }),
    );
  });

  it('returns full template details including variables', async () => {
    mockFetch(DETAILS_RESPONSE);
    const result = await getTemplateDetails('myorg', 'rust-library');
    expect(result.name).toBe('rust-library');
    expect(result.description).toBe('Rust library with CI/CD');
    expect(result.variables).toHaveLength(2);
    expect(result.variables[0].required).toBe(true);
    expect(result.variables[1].default).toBe('Engineering Team');
  });

  it('throws ApiValidationError on 404 (template not found)', async () => {
    mockFetch(errorBody('TemplateNotFound', "Template 'rust-library' not found"), 404);
    await expect(getTemplateDetails('myorg', 'rust-library')).rejects.toThrow(ApiValidationError);
  });

  it('throws ApiServerError on 502', async () => {
    mockFetch(errorBody('GitHubError', 'GitHub API request failed'), 502);
    await expect(getTemplateDetails('myorg', 'rust-library')).rejects.toThrow(ApiServerError);
  });

  it('throws ApiNetworkError when fetch rejects', async () => {
    mockFetchNetworkError();
    await expect(getTemplateDetails('myorg', 'rust-library')).rejects.toThrow(ApiNetworkError);
  });
});

// ---------------------------------------------------------------------------
// validateRepositoryName
// ---------------------------------------------------------------------------

describe('validateRepositoryName()', () => {
  it('calls POST /api/v1/repositories/validate-name with correct body', async () => {
    const response: ValidateRepositoryNameResponse = { valid: true, available: true };
    mockFetch(response);
    await validateRepositoryName('myorg', 'my-repo');
    expect(vi.mocked(fetch)).toHaveBeenCalledWith(
      '/api/v1/repositories/validate-name',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({ 'Content-Type': 'application/json' }),
        body: JSON.stringify({ organization: 'myorg', name: 'my-repo' }),
      }),
    );
  });

  it('returns valid:true for an available name', async () => {
    mockFetch({ valid: true, available: true });
    const result = await validateRepositoryName('myorg', 'my-repo');
    expect(result.valid).toBe(true);
    expect(result.available).toBe(true);
  });

  it('returns valid:false with messages for an invalid name (still 200)', async () => {
    const response: ValidateRepositoryNameResponse = {
      valid: false,
      available: false,
      messages: ['Invalid characters in name'],
    };
    mockFetch(response, 200);
    const result = await validateRepositoryName('myorg', 'My@Repo');
    expect(result.valid).toBe(false);
    expect(result.messages).toHaveLength(1);
  });

  it('throws ApiServerError on 500', async () => {
    mockFetch(errorBody('SystemError', 'Internal error'), 500);
    await expect(validateRepositoryName('myorg', 'my-repo')).rejects.toThrow(ApiServerError);
  });
});

// ---------------------------------------------------------------------------
// createRepository
// ---------------------------------------------------------------------------

describe('createRepository()', () => {
  const CREATION_RESPONSE: CreateRepositoryResponse = {
    repository: {
      name: 'my-new-service',
      fullName: 'myorg/my-new-service',
      url: 'https://github.com/myorg/my-new-service',
      visibility: 'private',
    },
    appliedConfiguration: {
      applied_settings: { repository: { has_issues: true } },
      sources: { 'repository.has_issues': 'global' },
    },
    createdAt: '2026-03-29T10:00:00Z',
  };

  const REQUEST = {
    organization: 'myorg',
    name: 'my-new-service',
    template: 'rust-library',
  };

  it('calls POST /api/v1/repositories with correct JSON body', async () => {
    mockFetch(CREATION_RESPONSE, 201);
    await createRepository(REQUEST);
    expect(vi.mocked(fetch)).toHaveBeenCalledWith(
      '/api/v1/repositories',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({ 'Content-Type': 'application/json' }),
        body: JSON.stringify(REQUEST),
      }),
    );
  });

  it('returns the created repository info on 201', async () => {
    mockFetch(CREATION_RESPONSE, 201);
    const result = await createRepository(REQUEST);
    expect(result.repository.fullName).toBe('myorg/my-new-service');
    expect(result.repository.url).toBe('https://github.com/myorg/my-new-service');
    expect((result.appliedConfiguration as Record<string, Record<string, string>>).sources['repository.has_issues']).toBe('global');
  });

  it('serialises optional fields (team, visibility, variables) when provided', async () => {
    mockFetch(CREATION_RESPONSE, 201);
    const req = {
      ...REQUEST,
      visibility: 'public',
      team: 'backend-team',
      variables: { service_name: 'MyService' },
    };
    await createRepository(req);
    const sentBody = JSON.parse(vi.mocked(fetch).mock.calls[0][1]?.body as string);
    expect(sentBody.visibility).toBe('public');
    expect(sentBody.team).toBe('backend-team');
    expect(sentBody.variables.service_name).toBe('MyService');
  });

  it('throws ApiValidationError on 400 (invalid request)', async () => {
    mockFetch(errorBody('ValidationError', 'Repository name contains invalid characters'), 400);
    await expect(createRepository(REQUEST)).rejects.toThrow(ApiValidationError);
  });

  it('ApiValidationError carries the error code and message from the response', async () => {
    mockFetch(errorBody('ValidationError', 'Name invalid'), 400);
    const err = await createRepository(REQUEST).catch((e) => e);
    expect(err).toBeInstanceOf(ApiValidationError);
    expect(err.errorDetails.code).toBe('ValidationError');
    expect(err.errorDetails.message).toBe('Name invalid');
    expect(err.statusCode).toBe(400);
  });

  it('throws ApiConflictError on 409 (name already taken)', async () => {
    mockFetch(
      errorBody('RepositoryAlreadyExists', "Repository 'myorg/my-new-service' already exists"),
      409,
    );
    await expect(createRepository(REQUEST)).rejects.toThrow(ApiConflictError);
  });

  it('throws ApiConflictError on 422 (unprocessable entity)', async () => {
    mockFetch(errorBody('TemplateNotFound', "Template 'rust-library' not found"), 422);
    const err = await createRepository(REQUEST).catch((e) => e);
    expect(err).toBeInstanceOf(ApiConflictError);
    expect(err.statusCode).toBe(422);
  });

  it('throws ApiServerError on 500', async () => {
    mockFetch(errorBody('SystemError', 'An internal error occurred'), 500);
    await expect(createRepository(REQUEST)).rejects.toThrow(ApiServerError);
  });

  it('throws ApiServerError on 502 (gateway error)', async () => {
    mockFetch(errorBody('GitHubError', 'GitHub API request failed'), 502);
    await expect(createRepository(REQUEST)).rejects.toThrow(ApiServerError);
  });

  it('throws ApiNetworkError when fetch rejects', async () => {
    mockFetchNetworkError();
    await expect(createRepository(REQUEST)).rejects.toThrow(ApiNetworkError);
  });

  it('throws ApiNetworkError with the original cause', async () => {
    const cause = new TypeError('Failed to fetch');
    mockFetchNetworkError(cause);
    const err = await createRepository(REQUEST).catch((e) => e);
    expect(err).toBeInstanceOf(ApiNetworkError);
    expect(err.cause).toBe(cause);
  });
});

// ---------------------------------------------------------------------------
// listTeams
// ---------------------------------------------------------------------------

describe('listTeams()', () => {
  const TEAMS_RESPONSE: ListTeamsResponse = {
    teams: [
      { slug: 'backend', name: 'Backend Engineers' },
      { slug: 'platform', name: 'Platform Team' },
    ],
  };

  it('calls GET /api/v1/orgs/:org/teams', async () => {
    mockFetch(TEAMS_RESPONSE);
    await listTeams('myorg');
    expect(vi.mocked(fetch)).toHaveBeenCalledWith(
      '/api/v1/orgs/myorg/teams',
      expect.objectContaining({ method: 'GET' }),
    );
  });

  it('URL-encodes the org name', async () => {
    mockFetch(TEAMS_RESPONSE);
    await listTeams('my org');
    expect(vi.mocked(fetch)).toHaveBeenCalledWith(
      '/api/v1/orgs/my%20org/teams',
      expect.anything(),
    );
  });

  it('returns the teams array with slug and name', async () => {
    mockFetch(TEAMS_RESPONSE);
    const result = await listTeams('myorg');
    expect(result).toHaveLength(2);
    expect(result[0].slug).toBe('backend');
    expect(result[0].name).toBe('Backend Engineers');
    expect(result[1].slug).toBe('platform');
    expect(result[1].name).toBe('Platform Team');
  });

  it('returns an empty array when the org has no teams', async () => {
    mockFetch({ teams: [] });
    const result = await listTeams('myorg');
    expect(result).toEqual([]);
  });

  it('throws ApiAuthError on 401', async () => {
    mockFetch(errorBody('AuthenticationError', 'Unauthorized'), 401);
    await expect(listTeams('myorg')).rejects.toThrow(ApiAuthError);
  });

  it('throws ApiServerError on 500', async () => {
    mockFetch(errorBody('SystemError', 'Internal server error'), 500);
    await expect(listTeams('myorg')).rejects.toThrow(ApiServerError);
  });

  it('throws ApiNetworkError when fetch rejects', async () => {
    mockFetchNetworkError();
    await expect(listTeams('myorg')).rejects.toThrow(ApiNetworkError);
  });
});
