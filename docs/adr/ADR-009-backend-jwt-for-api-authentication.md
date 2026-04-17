# ADR-009: Backend JWT for API User Authentication

Status: Accepted
Date: 2026-04-17
Owners: RepoRoller team

## Context

The API must authenticate incoming requests so that the calling user can be
identified for audit attribution.  The initial implementation validated the
GitHub user token on every request by calling `GET /rate_limit` against the
GitHub API.  This creates three problems:

1. **Latency**: One GitHub API round-trip (≈ 100–300 ms) is added to every
   protected request.
2. **Coupling**: A GitHub outage or rate-limit exhaustion blocks all
   authenticated API requests, not just those that actually need GitHub.
3. **Token exposure window**: The GitHub token is transmitted to the backend on
   every call, increasing the attack surface.

The backend already uses its own GitHub App credentials (ADR-006) for all
GitHub operations.  The GitHub user token is therefore only needed for
*identity* — not for permissions — and can be validated once rather than
continuously.

## Decision

Introduce a token exchange endpoint:

```
POST /api/v1/auth/token
Authorization: Bearer <github-user-token>
```

This endpoint:

1. Accepts a GitHub user token (PAT or OAuth token) as the Bearer.
2. Validates it against GitHub **once** (`GET /rate_limit`).
3. Resolves the GitHub login via `GET /user` (best-effort).
4. Issues a short-lived backend-signed JWT (HS256, 8-hour TTL) containing the
   user login as the `sub` claim.
5. Returns the JWT to the client.

All protected API endpoints then validate the backend JWT **locally** using the
shared `JWT_SECRET` — no GitHub API call is made per request.

**Do:**
- Load `JWT_SECRET` from the environment (minimum 32 bytes); panic at startup
  if absent or too short.
- Store `JWT_SECRET` with the same care as `GITHUB_APP_PRIVATE_KEY` (Key Vault
  in cloud deployments, never in configuration files).
- Use `secrecy::SecretString` for in-memory storage; never log the value.

**Don't:**
- Forward a GitHub token to any API endpoint other than `POST /auth/token`.
- Accept raw GitHub tokens on protected endpoints (they are rejected as
  malformed JWTs by `auth_middleware`).

## Consequences

**Enables:**

- GitHub API call for auth reduced from N (# of requests) to 1 per session.
- GitHub outages no longer block authenticated requests (except login).
- Per-request auth latency is now a local signature check (< 1 ms).
- Clear audit trail: the JWT `sub` claim carries the verified GitHub login.

**Forbids:**

- Sharing backend JWTs between users or applications — they are signed for
  this backend only.
- JWT TTL extensions without re-authentication — the client must call
  `/auth/token` again after 8 hours.

**Trade-offs:**

- Adds one mandatory login call before the frontend can use protected endpoints.
- `JWT_SECRET` becomes a critical secret alongside the App private key.
- Stateless design (ADR-003) is preserved: no server-side session store is
  needed because the JWT carries all necessary identity information.
- Logout/revocation is not supported without adding a token denylist; the
  8-hour TTL limits exposure.

## Alternatives considered

### Per-request GitHub token validation (previous implementation)

Validates the GitHub token on every request.  Correct but slow and tightly
coupled to GitHub availability.  Rejected because of latency and coupling.

### OAuth PKCE browser flow

Full OAuth 2.0 redirect flow.  Appropriate for browser-native login but
incompatible with CLI and CI/CD consumers.  Overkill for an internal single-
backend deployment.

### Opaque session tokens with Redis

Stateful token store.  Rejected because it violates ADR-003 (stateless
architecture) and adds an operational dependency on Redis.

### RS256 asymmetric JWT

Public/private key pair allows external parties to verify tokens without the
signing secret.  No external verifier exists for this backend, so the added
complexity is not warranted.  HS256 is sufficient.

## Implementation notes

- `jsonwebtoken = "9"` crate; HS256 algorithm; default `Validation` which
  checks `exp` automatically.
- `auth_middleware` now takes `State<AppState>` and is registered with
  `middleware::from_fn_with_state` so it can read `jwt_secret` without a
  global.
- JWT claims: `sub` (GitHub login), `iat` (issued-at), `exp` (expiry).
- `generate_backend_jwt` and `validate_github_token` are `pub(crate)` for use
  by the exchange handler.
- The exchange endpoint (`/auth/token`) is outside `protected_routes` and does
  not go through `auth_middleware`.

## References

- ADR-003: Stateless Architecture
- ADR-006: GitHub App Authentication
- [jsonwebtoken crate](https://crates.io/crates/jsonwebtoken)
- [secrecy crate](https://crates.io/crates/secrecy)
