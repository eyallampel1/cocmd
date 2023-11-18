use std::fs;
use serde::{Deserialize, Serialize};
use bollard::Docker;
use bollard::container::{CreateContainerOptions, Config, StartContainerOptions, LogOutput};
use bollard::exec::{CreateExecOptions, StartExecOptions};
use bollard::models::HostConfig;
use std::default::Default;
use std::env;
use futures::stream::StreamExt;
use bollard::container::LogsOptions;

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


pub async fn create_and_start_container(docker_image: &str, container_name: &str) -> Result<String, anyhow::Error> {
    let docker = Docker::connect_with_unix_defaults()?;
    let container_options = CreateContainerOptions { name: container_name.to_string(), ..Default::default() };
    let config = Config { image: Some(docker_image.to_string()), ..Default::default() };

    let container = docker.create_container(Some(container_options), config);
    docker.start_container(container_name, None::<StartContainerOptions<String>>).await?;

    Ok(container_name.to_string())
}

pub async fn run_playbook_on_docker(yaml_path: &str, container_id: &str,docker_image: &str) -> Result<(), anyhow::Error> {
    println!("Inside run_playbook_on_docker");

    let yaml_content = fs::read_to_string(yaml_path)?;
    let playbook: Playbook = serde_yaml::from_str(&yaml_content)?;

    let docker = Docker::connect_with_unix_defaults()?;

    for automation in playbook.automations {
        if let Some(content) = automation.content {
            for step in content.steps {
                println!("Executing step: {}", step.title);

                let container_options = CreateContainerOptions { name: container_id.to_string(), ..Default::default() };
                let config = Config { image: Some(docker_image.to_string()), ..Default::default() };
                docker.create_container(Some(container_options), config).await?;
                docker.start_container(container_id, None::<StartContainerOptions<String>>).await?;
                println!("Container created and started.");

                let exec_options = CreateExecOptions { cmd: Some(vec!["/bin/sh".to_string(), "-c".to_string(), step.content.clone()]), attach_stdout: Some(true), attach_stderr: Some(true), ..Default::default() };
                println!("Command to execute: {:?}", exec_options.cmd);

                let exec_creation = docker.create_exec(container_id, exec_options).await?;
                let exec_id = exec_creation.id;

                println!("Starting exec process: {}", exec_id);
                docker.start_exec(&exec_id, None::<StartExecOptions>).await?;
                println!("Command execution started: {}", exec_id);

                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await; // Delay to allow command execution

                let mut logs_stream = docker.logs(container_id, Some(LogsOptions::<String> { stdout: true, stderr: true, follow: true, ..Default::default() }));
                println!("Retrieving logs...");

                while let Some(result) = logs_stream.next().await {
                    match result {
                        Ok(output) => match output {
                            LogOutput::StdOut { message } => println!("STDOUT: {}", String::from_utf8_lossy(&message)),
                            LogOutput::StdErr { message } => eprintln!("STDERR: {}", String::from_utf8_lossy(&message)),
                            _ => {}
                        },
                        Err(e) => {
                            println!("Error reading logs: {}", e);
                            return Err(anyhow::Error::new(e));
                        },
                    }
                }
                println!("End of step: {}", step.title);
            }
        } else {
            println!("Automation '{}' is missing content.", automation.name);
        }
    }
    println!("End of playbook execution.");

    Ok(())
}