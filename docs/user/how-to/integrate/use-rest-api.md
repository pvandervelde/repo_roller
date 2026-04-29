---
title: "Authenticate and call the REST API"
description: "Obtain a token, set the Authorization header, and make calls to the RepoRoller REST API."
audience: "repository-creator"
type: "how-to"
---

# Authenticate and call the REST API

This guide shows how to obtain authentication credentials and make calls to the RepoRoller REST API from scripts or external tools.

## Obtain a token

The API accepts a **GitHub App installation token** in the `Authorization` header. Obtain one using the GitHub API:

```bash
# Install gh CLI if needed, then:
GITHUB_TOKEN=$(gh auth token)

# Or generate an installation token directly from a GitHub App private key:
# Use the GitHub Apps API: POST /app/installations/{installation_id}/access_tokens
```

> **Note:** Personal access tokens also work for development. For production systems, use a GitHub App installation token — it is scoped to the organisation, has an explicit expiry, and appears in audit logs as the App.

## Base URL and headers

All endpoints are under `/api/v1`. Every request requires:

```
Content-Type: application/json
Authorization: Bearer <token>
```

```bash
BASE_URL="https://reporoller.myorg.example/api/v1"
TOKEN="${GITHUB_TOKEN}"
```

## Check the health endpoint (no auth required)

```bash
curl -s "${BASE_URL}/health"
# → 200 OK
```

## Validate a repository name before creating

Use the validate-name endpoint to check availability before committing to creation:

```bash
curl -s -X POST "${BASE_URL}/repositories/validate-name" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"organization":"myorg","name":"payment-service"}' | jq .
```

Response on success:

```json
{
  "valid": true,
  "available": true,
  "messages": null
}
```

## Create a repository

```bash
curl -s -X POST "${BASE_URL}/repositories" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "payment-service",
    "organization": "myorg",
    "template": "rust-service",
    "description": "Payment processing microservice",
    "visibility": "private",
    "repositoryType": "service",
    "team": "payments-team",
    "variables": {
      "service_name": "payment-service",
      "service_port": "8080"
    }
  }' | jq .
```

Response on success (201 Created):

```json
{
  "repository": {
    "name": "payment-service",
    "fullName": "myorg/payment-service",
    "url": "https://github.com/myorg/payment-service",
    "visibility": "private"
  },
  "appliedConfiguration": {
    "features": {},
    "branchProtection": {},
    "labels": []
  }
}
```

## Handle error responses

The API returns structured JSON errors:

```json
{
  "error": "Repository already exists",
  "code": "REPOSITORY_ALREADY_EXISTS",
  "details": "myorg/payment-service"
}
```

Common HTTP status codes:

| Status | Meaning |
|---|---|
| `201` | Repository created |
| `200` | Validation passed |
| `400` | Invalid request (missing field, bad name format) |
| `401` | Invalid or expired token |
| `409` | Repository with that name already exists |
| `502` | GitHub API error |

In bash, detect failures by checking the HTTP status:

```bash
response=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/repositories" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"name":"my-repo","organization":"myorg","template":"rust-service"}')

http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

if [ "$http_code" -ne 201 ]; then
  echo "Failed with $http_code: $(echo "$body" | jq -r '.error')" >&2
  exit 1
fi

echo "Created: $(echo "$body" | jq -r '.repository.url')"
```
