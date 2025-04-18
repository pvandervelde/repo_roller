# CI/CD Pipeline Design (Issue #14)

## Problem Description

The project requires automated build, test, lint, security checks, and deployment pipelines.
This automation is crucial for maintaining code quality, ensuring security standards are met,
and enabling reliable, repeatable delivery, as outlined in
[Issue #14](https://github.com/pvandervelde/RepoRoller/issues/14).

## Surrounding Context

This project is a Rust workspace managed with Cargo. It potentially includes a frontend component
(indicated by `npm run lint` in Issue #14). Deployment targets include Azure/AWS, and infrastructure
is managed using Terraform. The project is hosted on GitHub, with `master` as the default branch.
GitHub Actions is the chosen CI/CD platform.

## Proposed Solution

Implement comprehensive CI/CD pipelines using GitHub Actions, separating concerns into distinct
workflows triggered by relevant events (push, pull request, release creation, timer).

### Design Goals

* Automate build, test, linting, and security scanning processes.
* Ensure high standards of code quality and security via checks on pushes and PRs.
* Enable automated, reproducible deployments triggered by releases.
* Automate release creation and management.
* Detect infrastructure drift proactively.
* Prevent merging of pull requests if checks fail.

### Design Constraints

* Must use GitHub Actions as the CI/CD platform.
* Sensitive information must be managed securely using GitHub Secrets, potentially scoped to
    specific GitHub Environments.
* The default branch is `master`.

### Design Decisions

* Utilize standard GitHub Actions (e.g., `actions/checkout`, `dtolnay/rust-toolchain`) instead
    of custom reusable workflows for common setup tasks.
* Separate workflows for CI checks (`ci.yml`), linting (`lint.yml`), release management
    (`release-plz.yml`), deployment (`deploy.yml`), and drift detection (`tf-drift.yml`).
* Employ distinct jobs within workflows for clarity and parallelism where appropriate.
* Use GitHub Environments (`on-master-push`, `on-release-publish`) to manage secrets and
    permissions for release and deployment steps, including OIDC integration for cloud providers.

### Alternatives Considered

* **Manual build and deployment:** Rejected (error-prone, not scalable).
* **Other CI/CD platforms:** Rejected (GitHub Actions preferred for integration).
* **Monolithic workflow:** Rejected (less maintainable, harder to manage triggers/permissions).

## Design

### Architecture

The CI/CD system will consist of the following GitHub Actions workflows in `.github/workflows/`:

1. **`ci.yml`:** (Continuous Integration Checks)
    * **Trigger:** `push` (all branches), `pull_request` (targeting `master`).
    * **Jobs:**
        * `unit-tests`: Run `cargo test --no-doc` with coverage tracking. Upload coverage to Codecov.io.
        * `doc-tests`: Run `cargo test --doc`.
        * `terraform-checks` (Future): Run `terraform validate`, `terraform fmt --check`.
    * **Goal:** Ensure core logic correctness, documentation examples work, and track test coverage.

2. **`lint.yml`:** (Linting and Static Analysis)
    * **Trigger:** `push` (all branches), `pull_request` (targeting `master`).
    * **Jobs:** (Run as separate jobs for clarity)
        * `audit`: Run `cargo audit`.
        * `check`: Run `cargo check`.
        * `format`: Run `cargo fmt --check`.
        * `clippy`: Run `cargo clippy -- -D warnings`.
        * `deny`: Run `cargo deny check`.
        * `terraform-lint` (Future): Run relevant Terraform linters.
        * `npm-lint` (Future): Run `npm run lint` if frontend exists.
    * **Goal:** Enforce code style, catch potential errors, check dependencies.

3. **`release-plz.yml`:** (Release Management)
    * **Trigger:** `push` (on `master` branch only).
    * **Environment:** `on-master-push` (for necessary secrets).
    * **Jobs:**
        * `release-please`: Run `release-plz` to create/update release PRs. If merged, `release-plz`
          (running again on the merge commit) creates the GitHub release and tag.
    * **Goal:** Automate version bumping, changelog generation, and GitHub release creation.

4. **`deploy.yml`:** (Deployment)
    * **Trigger:** `release` (event type `published`).
    * **Environment:** `on-release-publish` (for cloud credentials via OIDC, other secrets).
    * **Jobs:**
        * `deploy-cloud`: Deploy Azure Function / AWS resources.
        * `attach-binary`: Build CLI binary and attach it to the GitHub release assets.
    * **Goal:** Automate deployment to cloud environments and distribute CLI binaries upon release.

5. **`tf-drift.yml`:** (Terraform Drift Detection - Future Scope)
    * **Trigger:** `schedule` (e.g., daily).
    * **Jobs:**
        * `check-drift`: Run `terraform plan` against production/staging state. If drift is detected,
          create a GitHub issue.
    * **Goal:** Proactively detect and report unauthorized or unexpected infrastructure changes.

### Data Flow / Workflow Diagram (Mermaid)

```mermaid
graph TD
    subgraph Push/PR Events
        A[Push to any branch] --> B(ci.yml);
        A --> C(lint.yml);
        D[PR to master] --> B;
        D --> C;
        B --> E{Tests Pass?};
        C --> F{Linters Pass?};
        E & F -- Yes --> G[Checks OK];
        E -- No --> H[Fail PR];
        F -- No --> H;
    end

    subgraph Master Branch Push
        I[Push to master] --> J(release-plz.yml);
        J -- Uses --> K[Env: on-master-push];
        J --> L{Release PR / GitHub Release};
    end

    subgraph Release Published
        M[GitHub Release Published] --> N(deploy.yml);
        N -- Uses --> O[Env: on-release-publish];
        N --> P[Deploy Cloud Resources];
        N --> Q[Attach CLI Binary];
    end

    subgraph Scheduled Events (Future)
        R[Timer Trigger] --> S(tf-drift.yml);
        S --> T{Drift Detected?};
        T -- Yes --> U[Create GitHub Issue];
    end
```

### Secrets Management

Secrets will be managed using GitHub Secrets, scoped appropriately using Environments:

* **`on-master-push` Environment:**
  * Secrets required by `release-plz` (e.g., `GITHUB_TOKEN` with write permissions, potentially
      a specific PAT if needed).
* **`on-release-publish` Environment:**
  * Cloud credentials (e.g., Azure SPN, AWS Role ARN) configured for OIDC integration.
  * Other secrets needed for deployment tasks.
* **Repository/Organization Secrets:**
  * `CODECOV_TOKEN`: For uploading coverage reports from `ci.yml`.

*Note: No deployment to crates.io is planned, so `CRATES_IO_TOKEN` is not needed.*

## Conclusion

This revised design document outlines a more granular and robust structure for the CI/CD pipelines,
addressing the feedback provided for
[Issue #14](https://github.com/pvandervelde/RepoRoller/issues/14).
The implementation will begin with the `ci.yml` and `lint.yml` workflows to establish foundational
checks for code quality, style, and correctness on every push and pull request. Subsequent workflows
will build upon this foundation for release management and deployment.
