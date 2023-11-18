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

    // Create an instance of HostConfig with privileged set to true
    let host_config = HostConfig {
        privileged: Some(true),
        ..Default::default()
    };

    let config = Config {
        image: Some(docker_image.to_string()),
        host_config: Some(host_config), // Include the HostConfig here
        ..Default::default()
    };

    let container = docker.create_container(Some(container_options), config);
    docker.start_container(container_name, None::<StartContainerOptions<String>>);

    Ok(container_name.to_string())
}


pub async fn run_playbook_on_docker(yaml_path: &str, container_id: &str, x: &String) -> Result<(), anyhow::Error> {
    println!("Inside run_playbook_on_docker");

    // Read the YAML content from the given path
    let yaml_content = fs::read_to_string(yaml_path)?;
    let playbook: Playbook = serde_yaml::from_str(&yaml_content)?;

    // Connect to Docker
    let docker = Docker::connect_with_unix_defaults()?;

    // Iterate through the automations and execute each step
    for automation in playbook.automations {
        if let Some(content) = automation.content {
            for step in content.steps {
                println!("Executing step: {}", step.title);

                // Configure the execution command
                let exec_options = CreateExecOptions {
                    cmd: Some(vec!["/bin/sh".to_string(), "-c".to_string(), step.content]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                };

                // Create the execution process in the container
                let exec_creation = docker.create_exec(container_id, exec_options).await?;
                let exec_id = exec_creation.id;

                // Start the execution process
                println!("Starting exec process: {}", exec_id);
                docker.start_exec(&exec_id, None::<StartExecOptions>);

                // Optionally, add a delay or logic to wait for the command to complete
                 tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;

                // Retrieve and print logs
                let mut logs_stream = docker.logs(container_id, Some(LogsOptions::<String> {
                    stdout: true,
                    stderr: true,
                    follow: true,
                    ..Default::default()
                }));

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
