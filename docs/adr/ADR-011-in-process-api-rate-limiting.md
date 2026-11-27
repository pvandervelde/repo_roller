# ADR-011: In-Process API Rate Limiting

Status: Accepted
Date: 2026-11-27
Owners: RepoRoller team

## Context

The REST API had no application-level throttling. Two abuse paths mattered: an authenticated
caller hammering state-changing endpoints (each of which fans out to many GitHub API calls made
with the shared GitHub App credentials, ADR-006), and unauthenticated callers hammering the public
`POST /api/v1/auth/token` exchange (ADR-009), which calls GitHub on every request.

RepoRoller is stateless (ADR-003) and has no database or cache. Whatever we add must work without
new infrastructure, must not let a client choose its own bucket, and must keep memory bounded
under hostile input. The application has never been deployed, so there is no legacy behaviour to
preserve.

## Decision

Two fixed-window limiters live in `repo_roller_api::rate_limit`, held in memory per process.

1. **In-memory, per-process fixed window.** Each limiter is a mutex-guarded map of key to window
   start and count. No shared store.
2. **Client identity is the connection address only.** The token-endpoint limiter keys on the
   peer address from `ConnectInfo<SocketAddr>`. `X-Forwarded-For`, `Forwarded` and `X-Real-IP`
   are never read, because a client can set them freely. IPv4-mapped IPv6 is canonicalised and
   IPv6 clients are keyed by their /64 prefix (one subscriber normally controls a whole /64).
3. **Principal keying with a signed `pid` claim.** The per-user limiter runs after
   `auth_middleware` and keys on `login:<lowercased login>` or `sub:<pid>`. Callers with no
   resolvable login (GitHub App installation tokens) receive a backend JWT with no `sub` and a
   `pid` claim, the SHA-256 hex digest of the GitHub token. Because the JWT is HS256-signed, the
   client cannot choose or alter its key. Precedence is `pid` over `sub`. A token with neither a
   non-empty `pid` nor a non-empty `sub` is rejected with 401 code `MissingPrincipal`; requests
   are never pooled into a shared bucket. For loginless callers "per principal" therefore means
   "per token": the digest changes when the token does.
4. **Method scope.** The per-user limiter applies to every method except `GET`, `HEAD` and
   `OPTIONS` (deny-by-default for unsafe and unknown methods). The token limiter applies to
   `POST /auth/token`. Health and metrics endpoints are outside both.
5. **Bounded memory.** Each limiter tracks at most 10,000 keys, reclaiming expired keys
   incrementally. At the cap the per-user limiter evicts the key with the oldest window (that
   principal gets a fresh budget; keys are authenticated, so flooding them is not free). The
   token-endpoint limiter **fails closed**: new addresses get 429 until a window expires, so an
   attacker rotating source addresses cannot evict honest clients' budgets.

Rejections return `429` with `Retry-After` and error code `RateLimitExceeded`, are logged at WARN
without identity, and increment `rate_limit_rejections_total{scope}`;
`rate_limit_tracked_keys{scope}` exposes occupancy.

## Consequences

- No new infrastructure, no network hop on the request path, no new failure mode.
- Effective limits scale with the number of replicas (roughly `RATE_LIMIT_REQUESTS` times the
  replica count) and reset when a process restarts. Acceptable for abuse mitigation; not a
  precise quota.
- Behind a reverse proxy every client shares the proxy's address, and the frontend is the only
  caller of `/auth/token`, so in the standard deployment all users share a single token-endpoint
  bucket. Operators must raise `RATE_LIMIT_AUTH_REQUESTS`. Honouring forwarded headers safely
  would need a trusted-proxy allow-list and is deferred.
- Loginless callers get a fresh budget each time they exchange a different token.
- The fixed window permits up to 2x the limit in a burst straddling a boundary.
- Fail-closed at the cap means a large address-rotating flood can temporarily lock out new
  (not already tracked) sign-in sources.
- The JWT claim set gains an optional `pid` and `sub` becomes optional.

## Alternatives considered

- **Shared store (Redis or similar):** exact cross-replica limits, but adds infrastructure and a
  new availability dependency to a deliberately stateless service (ADR-003), and the threat
  model does not need exact limits.
- **`tower-governor` / GCRA crates:** well-tested, but its default key extraction leans on
  forwarded headers, and per-key eviction and fail-closed behaviour at a key cap are not
  controllable; the principal namespacing would need custom code anyway. Revisit if requirements
  grow.
- **Trusting `X-Forwarded-For`:** spoofable by any client unless a trusted-proxy list exists;
  rejected for now.
- **Limiting only `POST`:** leaves `PUT`, `PATCH` and `DELETE` unthrottled; rejected in favour of
  limiting everything except safe methods.
- **A shared bucket for loginless callers:** one noisy installation token would starve all
  others; rejected in favour of the signed `pid`.
- **Sliding-window or token-bucket algorithms:** smoother, but more state per key for little
  benefit at these limits.

## Implementation notes

- `principal_key`, `client_key`, `RateLimiter`, `FullPolicy` and the middlewares are in
  `crates/repo_roller_api/src/rate_limit.rs`; `Claims` and `generate_loginless_backend_jwt` in
  `middleware.rs`.
- Configuration: `RATE_LIMIT_ENABLED`, `RATE_LIMIT_REQUESTS`, `RATE_LIMIT_WINDOW_SECS`,
  `RATE_LIMIT_AUTH_REQUESTS`, `RATE_LIMIT_AUTH_WINDOW_SECS`; invalid values fall back to defaults.
- Request headers are never written to trace logs, so the `Authorization` header cannot leak.

## References

- ADR-003: Stateless Architecture with External Storage
- ADR-006: GitHub App Authentication
- ADR-009: Backend JWT for API User Authentication
- Issue #284 and PR #384
