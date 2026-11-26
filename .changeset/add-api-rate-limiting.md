---
"repo_roller_api": minor
---

Add per-user rate limiting to the REST API (#284).

Authenticated `POST` requests are now limited per user using a fixed window, 10 requests per 60
seconds by default. Per-user keys are namespaced: the lowercased login, or a stable principal id
for tokens whose login could not be resolved; tokens with no identifier get `401`. The public
`POST /api/v1/auth/token` is also limited per client IP (connection address only, `X-Forwarded-*`
ignored, so behind a reverse proxy all clients share the proxy IP's bucket and operators should
raise the limit), 20 requests per 60 seconds by default. Requests over a limit receive `429 Too
Many Requests` with a `Retry-After` header and error code `RateLimitExceeded`, are logged at WARN,
and increment the new `rate_limit_rejections_total` Prometheus counter, labelled `scope` (`user`
or `auth_ip`). `GET` requests, health and metrics endpoints are not limited.

Each limiter tracks at most 10,000 keys (new `rate_limit_tracked_keys{scope}` gauge). At the cap the
per-user limiter evicts the key with the oldest window; the token-endpoint limiter fails closed,
rejecting new client addresses with `429`. IPv6 clients are keyed by /64 prefix. Request headers
(`Authorization`, `Cookie`, ...) are no longer written to trace logs.

Configure with `RATE_LIMIT_ENABLED` (`false`/`0` disables both limiters), `RATE_LIMIT_REQUESTS`,
`RATE_LIMIT_WINDOW_SECS`, `RATE_LIMIT_AUTH_REQUESTS` and `RATE_LIMIT_AUTH_WINDOW_SECS`; invalid
values fall back to the defaults. State is in memory per process, so effective limits scale with
the number of replicas and reset on restart.
