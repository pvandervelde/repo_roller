# UX Assertions

Testable behavioural specifications for the RepoRoller web UI. These assertions are inputs for
the Tester to generate UI test specifications.

Each assertion follows Given / When / Then structure.

---

## Authentication

### UX-ASSERT-001: Unauthenticated users cannot access protected routes

- **Given**: A user has no active session
- **When**: They navigate to `/create`
- **Then**: They are redirected to `/sign-in`
- **And**: No content from `/create` is visible before the redirect

### UX-ASSERT-002: GitHub identity is captured from OAuth, not user input

- **Given**: A user completes the GitHub OAuth flow
- **When**: Their session is created
- **Then**: Their GitHub login is stored as the authenticated identity
- **And**: There is no form field on any screen where the user can supply or override their username

### UX-ASSERT-003: OAuth denial navigates to Access Denied, not Sign In

- **Given**: A user clicks "Authorize" and then "Cancel" on the GitHub authorization page
- **When**: GitHub redirects back to `/auth/callback` with `error=access_denied`
- **Then**: The user is shown the Access Denied screen (SCR-003) with the `access_denied` reason copy
- **And**: The user is NOT shown the Sign In screen directly

### UX-ASSERT-004: Sign-out destroys session and returns to sign-in

- **Given**: A user is authenticated and on any page with the AppShell
- **When**: They click "Sign out" in the UserBadge dropdown
- **Then**: Their session is destroyed
- **And**: They are redirected to `/sign-in`
- **And**: Navigating back to `/create` requires re-authentication

---

## Step 1: Template Selection

### UX-ASSERT-005: Next button is disabled until a template is selected

- **Given**: The user is on Step 1 of the creation wizard
- **When**: No template card has been selected
- **Then**: The "Next: Repository settings" button is disabled

### UX-ASSERT-006: Template details are fetched on card selection, not on Next click

- **Given**: The user clicks a template card in Step 1
- **When**: The selection is registered
- **Then**: A `GET /orgs/{org}/templates/{template}` request is immediately initiated
- **And**: The Next button is not enabled until the details fetch completes successfully

### UX-ASSERT-007: Returning to Step 1 from Step 2 preserves the selected template

- **Given**: The user has selected a template in Step 1 and advanced to Step 2
- **When**: They click "← Back"
- **Then**: The previously selected template card is still shown as selected in Step 1

### UX-ASSERT-008: Template list API failure shows an error state with retry

- **Given**: The template list API call fails
- **When**: The Step 1 view renders
- **Then**: An error message is shown with a "Try again" button
- **And**: No template cards are shown
- **And**: The Next button remains disabled

---

## Step 2: Repository Settings

### UX-ASSERT-009: Repository name format is validated client-side on type

- **Given**: The user is typing in the repository name field
- **When**: The name contains characters not matching the GitHub name format (e.g., uppercase, spaces, special characters)
- **Then**: An inline error message appears below the field describing the format constraint
- **And**: No API call is made for uniqueness checking

### UX-ASSERT-010: Uniqueness check fires on blur, not on type

- **Given**: The user has entered a correctly-formatted repository name
- **When**: The name field loses focus (blur event)
- **Then**: A `POST /api/v1/repositories/validate-name` API call is made
- **And**: The field shows a "Checking availability…" indicator during the call

### UX-ASSERT-011: Next / Create button is disabled while uniqueness check is in-flight

- **Given**: A uniqueness check API call is in progress
- **When**: The user attempts to click the Next or Create button
- **Then**: The button is disabled and the click has no effect

### UX-ASSERT-012: Next / Create button is disabled when name is already taken

- **Given**: The uniqueness check API returns that the name is already taken
- **When**: The result is shown to the user
- **Then**: The name field shows an "already taken" error
- **And**: The Next / Create button remains disabled until the name is changed

### UX-ASSERT-028: Next / Create button is enabled when the uniqueness check could not be completed

- **Given**: The uniqueness check API call fails (network error or server error)
- **When**: The `check_failed` result is shown below the name field
- **Then**: The Next / Create button is **enabled** (not disabled)
- **And**: A warning indicator below the field advises the user that availability could not be confirmed
- **And**: The user can proceed with the understanding that the name may already exist (a race-condition 422 on creation will handle the conflict)

### UX-ASSERT-013: Changing the name after a "taken" result clears the error immediately

- **Given**: The name field shows an "already taken" error
- **When**: The user changes the value in the name field (any keystroke)
- **Then**: The "already taken" error is immediately cleared (without waiting for blur)

### UX-ASSERT-014: Step 2 values are preserved when returning from Step 3

- **Given**: The user has advanced from Step 2 to Step 3 with a valid name and other settings
- **When**: They click "← Back" from Step 3
- **Then**: The repository name, type, team, and visibility fields in Step 2 still show the values they entered
- **And**: No additional API calls are made (uniqueness is not re-checked on step return)

---

## Step 3: Template Variables

### UX-ASSERT-015: Step 3 is skipped when the template has no variables

- **Given**: The user selects a template that defines no template variables
- **When**: They complete Step 2 with a valid repository name
- **Then**: The "Create Repository" button appears in Step 2 (not a "Next: Variables" button)
- **And**: No Step 3 is shown; clicking Create initiates the creation directly

### UX-ASSERT-016: Create button disabled until all required variables are filled

- **Given**: The user is on Step 3 with at least one required variable
- **When**: Any required variable field is empty
- **Then**: The "Create Repository" button is disabled

### UX-ASSERT-017: Optional variables with defaults are pre-filled

- **Given**: The selected template has a variable with a `default_value`
- **When**: Step 3 renders
- **Then**: The corresponding field shows the default value pre-populated
- **And**: The user can change it

---

## Creation Submission

### UX-ASSERT-018: Double submission is prevented

- **Given**: The user clicks "Create Repository"
- **When**: The creation API call is in flight
- **Then**: The creation overlay is shown
- **And**: The wizard buttons are not accessible
- **And**: A second API call cannot be initiated

### UX-ASSERT-019: Race-condition name conflict returns user to Step 2

- **Given**: The user submits a valid creation request
- **When**: The API returns 422 with a "name already taken" error (race condition)
- **Then**: The creation overlay is dismissed
- **And**: The user is returned to Step 2
- **And**: The name field shows an "already taken" error
- **And**: An InlineAlert explains What happened above the name field

### UX-ASSERT-020: Template-not-found error during creation returns user to Step 1

- **Given**: The user submits a valid creation request
- **When**: The API returns 422 with a "template not found" error
- **Then**: The creation overlay is dismissed
- **And**: The user is returned to Step 1
- **And**: The template list is refreshed
- **And**: An InlineAlert explains that the template is no longer available

### UX-ASSERT-021: GitHub API errors are shown inline without losing form data

- **Given**: The user submits a creation request
- **When**: The API returns a 5xx error
- **Then**: The creation overlay is dismissed
- **And**: An InlineAlert with the error message is shown on the current step
- **And**: All form data (name, template selection, variables) is still present
- **And**: The user can retry without re-entering data

---

## Success Screen

### UX-ASSERT-022: Success screen shows the correct repository name and link

- **Given**: Repository creation succeeds
- **When**: The user is redirected to `/create/success`
- **Then**: The full repository name (`{org}/{name}`) is displayed
- **And**: A link to the GitHub repository URL is present and correct

### UX-ASSERT-023: "Create another" navigates to a reset wizard

- **Given**: The user is on the Repository Created screen
- **When**: They click "Create another repository"
- **Then**: They are navigated to `/create`
- **And**: The wizard starts at Step 1 with no template selected and all fields empty

---

## Accessibility

### UX-ASSERT-024: Keyboard-only navigation works through the complete creation wizard

- **Given**: A user navigates using Tab and Enter/Space only
- **When**: Interacting with the creation wizard from Step 1 through to clicking Create
- **Then**: All interactive elements (template cards, form fields, buttons) are reachable
- **And**: Tab order follows the documented reading order (top to bottom)
- **And**: Focus moves to the step `<h1>` heading when advancing to a new step

### UX-ASSERT-025: Error messages are announced to screen readers

- **Given**: A validation error or API error appears on any form field or as an InlineAlert
- **When**: The error is rendered
- **Then**: The error message uses `role="alert"` and is announced by screen readers without requiring user focus movement

---

## Branding

### UX-ASSERT-026: Logo image falls back to text wordmark when not configured

- **Given**: `brand.logo_url` is not set in the deployment branding configuration
- **When**: Any screen with the logo area renders (AppShell header or BrandCard)
- **Then**: The app name text is shown as a styled text wordmark
- **And**: No broken image icon appears
- **And**: The layout does not shift or collapse

### UX-ASSERT-027: Primary colour applies consistently to all interactive elements

- **Given**: A custom `brand.primary_color` is configured at deployment
- **When**: Any interactive element renders — primary buttons, selected template card borders,
  step progress indicators, links, focus rings
- **Then**: Each element uses the configured colour
- **And**: No interactive element uses the default blue (`#0969da`) when a custom colour is set

### UX-ASSERT-029: Dark-mode primary colour overrides the light-mode value under dark scheme

- **Given**: Both `brand.primary_color` and `brand.primary_color_dark` are configured at deployment
- **When**: The operating system / browser `prefers-color-scheme` is set to `dark`
- **Then**: All interactive elements use the dark-mode colour (`brand.primary_color_dark`)
- **And**: When `prefers-color-scheme` is set to `light` (or is not set), the light-mode colour
  is used
- **And**: No JavaScript or client-side toggle is required — switching is handled by CSS alone

### UX-ASSERT-030: Dark-mode logo is used when configured and system dark mode is active

- **Given**: Both `brand.logo_url` and `brand.logo_url_dark` are configured at deployment
- **When**: The operating system / browser `prefers-color-scheme` is set to `dark`
- **Then**: The dark-mode logo image is displayed in the AppShell header and on BrandCard screens
- **And**: When `prefers-color-scheme` is `light` (or not set), the light-mode logo is displayed
- **And**: When only `brand.logo_url` is configured (no dark variant), it is used in both modes

### UX-ASSERT-031: Omitting dark-mode brand tokens does not break the UI

- **Given**: `brand.primary_color_dark` and `brand.logo_url_dark` are both absent from the
  deployment configuration
- **When**: Any screen renders in a system dark mode environment
- **Then**: The light-mode primary colour and light-mode logo (or wordmark) are displayed
- **And**: No missing-image icon, broken layout, or JavaScript error occurs
