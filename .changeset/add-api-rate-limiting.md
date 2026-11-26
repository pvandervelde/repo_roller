---
"repo_roller_api": minor
---

Add per-user rate limiting to the REST API (#284).

Authenticated `POST` requests are now limited per user (JWT `sub`) using a fixed window, 10
requests per 60 seconds by default. Requests over the limit receive `429 Too Many Requests` with a
`Retry-After` header and error code `RateLimitExceeded`, are logged at WARN, and increment the new
`rate_limit_rejections_total` Prometheus counter. `GET` requests, `/auth/token`, health and
metrics endpoints are not limited.

Configure with `RATE_LIMIT_ENABLED` (`false`/`0` disables), `RATE_LIMIT_REQUESTS` and
`RATE_LIMIT_WINDOW_SECS`; invalid values fall back to the defaults. State is in memory per
process, so effective limits scale with the number of replicas and reset on restart.
