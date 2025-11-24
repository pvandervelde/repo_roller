//! Docker container management for E2E tests
//!
//! Provides utilities for starting/stopping the repo_roller_api
//! container for end-to-end integration testing.

use anyhow::{Context, Result};
use bollard::container::{
    Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions,
    StopContainerOptions,
};
use bollard::image::BuildImageOptions;
use bollard::Docker;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for running API container in tests
pub struct ApiContainerConfig {
    /// GitHub App ID for authentication
    pub github_app_id: String,

    /// GitHub App private key
    pub github_app_private_key: String,

    /// Test organization name
    pub test_org: String,

    /// Metadata repository name
    pub metadata_repo: String,

    /// Container port (host port)
    pub port: u16,
}

impl ApiContainerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            github_app_id: env::var("GITHUB_APP_ID").context("GITHUB_APP_ID not set")?,
            github_app_private_key: env::var("GITHUB_APP_PRIVATE_KEY")
                .context("GITHUB_APP_PRIVATE_KEY not set")?,
            test_org: env::var("TEST_ORG").context("TEST_ORG not set")?,
            metadata_repo: env::var("METADATA_REPOSITORY_NAME")
                .unwrap_or_else(|_| ".reporoller".to_string()),
            port: env::var("TEST_API_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("Invalid TEST_API_PORT")?,
        })
    }
}

/// Manages API container lifecycle for testing
pub struct ApiContainer {
    docker: Docker,
    container_id: Option<String>,
    config: ApiContainerConfig,
}

impl ApiContainer {
    /// Create new container manager
    ///
    /// Note: This does not build the Docker image. Use `build_image()` if you need
    /// to build the image, or ensure it's already built by CI/CD before running tests.
    pub async fn new(config: ApiContainerConfig) -> Result<Self> {
        let docker =
            Docker::connect_with_local_defaults().context("Failed to connect to Docker daemon")?;

        Ok(Self {
            docker,
            container_id: None,
            config,
        })
    }

    /// Build the API Docker image (optional - typically done by CI/CD)
    ///
    /// In CI/CD workflows, the image should already be built before running E2E tests.
    /// This method is provided for local development and testing convenience.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use integration_tests::container::{ApiContainer, ApiContainerConfig};
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = ApiContainerConfig::from_env()?;
    /// let mut container = ApiContainer::new(config).await?;
    ///
    /// // Only needed for local dev if image not already built
    /// container.build_image().await?;
    ///
    /// let base_url = container.start().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build_image(&self) -> Result<String> {
        tracing::info!("Building repo_roller_api Docker image...");

        // Build from workspace root with proper context
        let mut build_options = BuildImageOptions::default();
        build_options.dockerfile = "crates/repo_roller_api/Dockerfile";
        build_options.t = "repo_roller_api:test";
        build_options.rm = true;

        // Use bollard's build_image with the current directory as context
        let mut stream = self.docker.build_image(build_options, None, None);

        while let Some(msg) = stream.next().await {
            let info = msg.context("Build stream error")?;
            if let Some(stream_msg) = info.stream {
                print!("{}", stream_msg);
            }
            if let Some(error_msg) = info.error {
                anyhow::bail!("Docker build failed: {}", error_msg);
            }
        }

        tracing::info!("✓ Docker image built successfully");
        Ok("repo_roller_api:test".to_string())
    }

    /// Start the API container
    ///
    /// Assumes the Docker image "repo_roller_api:test" already exists.
    /// If the image doesn't exist, this will fail. Call `build_image()` first
    /// or ensure your CI/CD pipeline has built it.
    pub async fn start(&mut self) -> Result<String> {
        let image = "repo_roller_api:test";

        // Container configuration
        let env_vars = vec![
            format!("GITHUB_APP_ID={}", self.config.github_app_id),
            format!(
                "GITHUB_APP_PRIVATE_KEY={}",
                self.config.github_app_private_key
            ),
            format!("METADATA_REPOSITORY_NAME={}", self.config.metadata_repo),
            "RUST_LOG=info".to_string(),
            "API_HOST=0.0.0.0".to_string(),
            "API_PORT=8080".to_string(),
        ];

        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            "8080/tcp".to_string(),
            Some(vec![bollard::service::PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(self.config.port.to_string()),
            }]),
        );

        let mut exposed_ports = HashMap::new();
        exposed_ports.insert("8080/tcp".to_string(), HashMap::new());

        let host_config = bollard::service::HostConfig {
            port_bindings: Some(port_bindings),
            ..Default::default()
        };

        let container_config = Config {
            image: Some(image.to_string()),
            env: Some(env_vars),
            exposed_ports: Some(exposed_ports),
            host_config: Some(host_config),
            ..Default::default()
        };

        // Create container
        let container = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: "repo_roller_api_test",
                    ..Default::default()
                }),
                container_config,
            )
            .await
            .context("Failed to create container")?;

        let container_id = container.id;
        self.container_id = Some(container_id.clone());

        // Start container
        self.docker
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
            .context("Failed to start container")?;

        tracing::info!("Container started: {}", container_id);

        // Wait for container to be healthy
        self.wait_for_health().await?;

        Ok(format!("http://localhost:{}", self.config.port))
    }

    /// Wait for container to become healthy
    async fn wait_for_health(&self) -> Result<()> {
        let base_url = format!("http://localhost:{}", self.config.port);
        let client = reqwest::Client::new();

        tracing::info!("Waiting for API server to be ready...");

        for attempt in 1..=30 {
            sleep(Duration::from_secs(1)).await;

            match client.get(&format!("{}/health", base_url)).send().await {
                Ok(response) if response.status().is_success() => {
                    tracing::info!("✓ API server is ready (attempt {})", attempt);
                    return Ok(());
                }
                Ok(response) => {
                    tracing::debug!("Health check returned {}", response.status());
                }
                Err(e) => {
                    tracing::debug!("Health check failed: {} (attempt {})", e, attempt);
                }
            }
        }

        anyhow::bail!("Container failed to become healthy after 30 seconds")
    }

    /// Stop and remove the container
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(container_id) = &self.container_id {
            tracing::info!("Stopping container: {}", container_id);

            self.docker
                .stop_container(container_id, Some(StopContainerOptions { t: 5 }))
                .await
                .context("Failed to stop container")?;

            self.docker
                .remove_container(
                    container_id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await
                .context("Failed to remove container")?;

            tracing::info!("✓ Container stopped and removed");
            self.container_id = None;
        }

        Ok(())
    }
}

impl Drop for ApiContainer {
    fn drop(&mut self) {
        // Best-effort cleanup on drop
        if let Some(container_id) = &self.container_id {
            let docker = self.docker.clone();
            let id = container_id.clone();

            tokio::spawn(async move {
                let _ = docker.stop_container(&id, None).await;
                let _ = docker.remove_container(&id, None).await;
            });
        }
    }
}
