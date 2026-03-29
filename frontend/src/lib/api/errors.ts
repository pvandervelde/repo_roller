/**
 * Typed error classes for API client responses.
 *
 * Error categorisation matches the mapping in docs/spec/interfaces/api-error-handling.md:
 *   400 / 404  → ApiValidationError   (user-correctable input problem)
 *   401 / 403  → ApiAuthError         (authentication / authorisation failure)
 *   409 / 422  → ApiConflictError     (name taken, template not found at creation time)
 *   5xx        → ApiServerError       (server-side failure)
 *   network    → ApiNetworkError      (fetch threw before a response was received)
 */
import type { ErrorDetails } from './types';

export class ApiError extends Error {
    constructor(
        message: string,
        public readonly statusCode: number,
        public readonly errorDetails: ErrorDetails,
    ) {
        super(message);
        this.name = 'ApiError';
    }
}

/** 400 Bad Request or 404 Not Found — invalid or unknown resource in the request. */
export class ApiValidationError extends ApiError {
    constructor(statusCode: number, details: ErrorDetails) {
        super(details.message, statusCode, details);
        this.name = 'ApiValidationError';
    }
}

/** 401 Unauthorized or 403 Forbidden — authentication / authorisation failure. */
export class ApiAuthError extends ApiError {
    constructor(statusCode: number, details: ErrorDetails) {
        super(details.message, statusCode, details);
        this.name = 'ApiAuthError';
    }
}

/** 409 Conflict or 422 Unprocessable Entity — name taken or template not found at creation. */
export class ApiConflictError extends ApiError {
    constructor(statusCode: number, details: ErrorDetails) {
        super(details.message, statusCode, details);
        this.name = 'ApiConflictError';
    }
}

/** 5xx — server-side failure. */
export class ApiServerError extends ApiError {
    constructor(statusCode: number, details: ErrorDetails) {
        super(details.message, statusCode, details);
        this.name = 'ApiServerError';
    }
}

/** fetch() threw before a response was received — network failure. */
export class ApiNetworkError extends Error {
    constructor(cause: unknown) {
        super('Network error communicating with API');
        this.name = 'ApiNetworkError';
        this.cause = cause;
    }
}
