use std::fs;
use serde::{Deserialize, Serialize};
use bollard::Docker;
use bollard::container::{CreateContainerOptions, Config, StartContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecOptions};
use bollard::models::HostConfig;
use std::default::Default;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Playbook {
    name: String,
    automations: Vec<Automation>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Automation {
    name: String,
    content: Option<AutomationContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AutomationContent {
    description: String,
    env: Option<String>,
    steps: Vec<Step>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Step {
    title: String,
    description: Option<String>,
    runner: String,
    approval_message: Option<String>,
    content: String,
}

pub async fn run_playbook_on_docker(yaml_path: &str) -> Result<(), anyhow::Error> {
    println!("I am inside docker.rs file");
    let yaml_content = fs::read_to_string(yaml_path)?;
    let playbook: Playbook = serde_yaml::from_str(&yaml_content)?;

    let docker = Docker::connect_with_unix_defaults()?;

    for automation in playbook.automations {
        if let Some(content) = automation.content {
            for step in content.steps {
                // Check for user approval if needed
                if let Some(approval_message) = step.approval_message {
                    println!("{}", approval_message);
                    // Add logic to capture user input and proceed based on that
                }

                // Create or find a container to run the command
                let container_name = "your_container_name";
                let container_options = CreateContainerOptions {
                    name: container_name,
                    platform: None,
                };
                let config = Config {
                    image: Some("your_docker_image"),
                    cmd: Some(vec!["/bin/sh", "-c", &step.content]),
                    ..Default::default()
                };



                // Execute command inside container
                let exec_options = CreateExecOptions {
                    cmd: Some(vec!["/bin/sh", "-c", &step.content]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                };

                let container_creation = docker.create_container(Some(container_options), config).await?;
                let _container = container_creation; // No need for rt.block_on, just use .await

                let container_start = docker.start_container(&container_name, None::<StartContainerOptions<String>>).await?;
                let exec_creation = docker.create_exec(&container_name, exec_options).await?;
                let exec_start = docker.start_exec(&exec_creation.id, None::<StartExecOptions>).await?;

                // Handle command output and errors
            // ...

            // Stop or remove container if necessary
            // ...
            }
        } else {
            println!("Automation '{}' is missing content.", automation.name);
        }
    }

    Ok(())
}