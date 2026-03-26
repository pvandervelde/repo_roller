# Repository Creation Flow

## Overview

The repository creation flow is the primary function of the web UI. It walks the user through a
2–3 step wizard to select a template, configure the repository, supply template variables, and
submit the creation request. Step 3 (variables) is skipped when the selected template defines
no variables.

---

## User Goal: Create a fully-configured GitHub repository

### Happy Path Flow

```mermaid
flowchart TD
    A([User arrives at /create]) --> B[Load template list from API]
    B --> C{Template list result}
    C -->|Success| D[Step 1: Choose a Template]
    C -->|Empty list| E[Empty state: No templates configured]
    C -->|API error| F[Error state: Could not load templates]

    F --> G{User clicks Retry}
    G --> B

    D --> H{User selects a template}
    H --> I[Fetch template details from API]
    I --> J{Template details result}
    J -->|Success| K[Next button activates]
    J -->|Error| L[Error banner: Could not load template details. Try again.]
    L --> M{User retries fetch}
    M --> I

    K --> N{User clicks Next}
    N --> O[Step 2: Repository Settings]

    O --> P{User fills in repository name}
    P --> Q{Name format valid? client-side}
    Q -->|Invalid format| R[Inline error: format message]
    Q -->|Valid format| S{User leaves name field}
    S --> T[API: validate name uniqueness]
    T --> U{Validation result}
    U -->|Available| V[Name field shows green 'Available' indicator]
    U -->|Already taken| W[Name field shows 'already taken' error]
    U -->|API error| X[Name field shows 'Could not check availability' warning]

    V --> Y{User completes Step 2}

    Y --> Z{Template has variables?}
    Z -->|No variables| AA[Show inline summary + Create button in Step 2]
    Z -->|Has variables| AB[Next button: 'Next: Variables']

    AA --> AC{User clicks Create Repository}
    AB --> AD{User clicks Next}
    AD --> AE[Step 3: Template Variables]
    AE --> AF{User fills in required variables}
    AF --> AG{All required variables filled?}
    AG -->|No| AH[Create button remains disabled]
    AG -->|Yes| AI[Show repository summary + Create button active]
    AI --> AC

    AC --> AJ[Creating overlay: spinner + message]
    AJ --> AK[POST /api/v1/repositories]
    AK --> AL{Creation result}

    AL -->|201 Created| AM[Redirect to /create/success]
    AM --> AN([Repository Created screen])

    AL -->|422 Name taken - race condition| AO[Return to Step 2: name-taken error highlighted]
    AL -->|422 Template not found| AP[Return to Step 1: template removed error banner]
    AL -->|403 Permission denied| AQ[Inline error on Step 3 or 2: permission message]
    AL -->|5xx GitHub error| AR[Inline error: GitHub unavailable]
    AL -->|Network error| AS[Inline error: Could not reach server]
```

---

## Step Structure

| Step | Name | Required for all flows |
|---|---|---|
| Step 1 | Choose a Template | Always |
| Step 2 | Repository Settings | Always |
| Step 3 | Template Variables | Only when selected template has ≥1 variable |

---

## Back Navigation Within the Wizard

| From | Back action | State preserved |
|---|---|---|
| Step 2 | Returns to Step 1 | Previously selected template card remains selected |
| Step 3 | Returns to Step 2 | All Step 2 field values preserved (name, type, team, visibility) |
| Step 1 | Back goes to browser history (leaves /create) | Unsaved-data warning dialog shown if name was entered |

---

## Leaving the Page Mid-Flow

If the user has entered data in the creation form (at minimum: selected a template) and attempts
to navigate away (browser back, closing tab, following a link), the browser's native unload event
shows a confirmation: "Are you sure you want to leave? Your repository settings will be lost."

This guard is removed once creation succeeds (navigation to `/create/success` is intentional) and
while the creation overlay is active (to prevent partial interruption).

---

## Template Details Loading Strategy

Template details (including variable definitions) are fetched when the user **selects** a template
card in Step 1 (not when they click Next). This means by the time the user clicks Next, the detail
fetch is either already complete or nearly so, avoiding a visible loading delay on the Step 2
transition.

If the details fetch fails, an error banner is shown below the template card and the Next button
remains disabled until either the user retries the fetch or selects a different template.
