use crate::core::models::script_model::{ScriptModel, StepModel};
use crate::core::utils::sys::OS;
use std::collections::HashMap;
use anyhow::{Error, anyhow};
use bollard::image::{CreateImageOptions};
use bollard::Docker;
use std::default::Default;
use std::env::args;
use tokio::runtime::Runtime;
use futures::stream::StreamExt;
use colored::*;
use bollard::image::ListImagesOptions;
use std::process::Command;
use std::str;
use log::error;
use crate::core::models::settings::Settings;
// Assuming PackagesManager is properly imported and initialized
use crate::core::packages_manager::PackagesManager;
use dialoguer::{Input, Select};
use std::path::{Path, PathBuf};
//import step_runner
use crate::runner::step_runner::{apply_params_to_content, handle_step};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use crate::runner::test_runner::docker;




#[derive(Deserialize)]
struct Playbook {
    steps: Vec<StepModel>,
}

struct TestRunner<'a> {
    playbook: String,
    os_list: Vec<String>,
    packages_manager: &'a PackagesManager,
}


impl<'a> TestRunner<'a> {
    // Updated constructor with a reference to PackagesManager
    fn new(playbook: String, os_list: Vec<String>, packages_manager: &'a PackagesManager) -> TestRunner<'a> {
        println!("{} with playbook: {}", "Creating new TestRunner".green(), playbook.yellow());
        TestRunner { playbook, os_list, packages_manager }
    }

    fn construct_yaml_path(&self) -> String {
        // Get settings to access the runtime directory
        let settings = Settings::new(None, None);

        // Construct the path to the playbook's YAML file using the runtime_dir
        settings.runtime_dir.join(&self.playbook).join("cocmd.yaml").to_string_lossy().into_owned()
    }

    // Method to pull the appropriate Docker image for the OS
    fn pull_docker_image(image: &str) -> Result<(), Error> {
        let output = Command::new("docker")
            .arg("image")
            .arg("ls")
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("{}", "Failed to list Docker images".red());
                    return Err(anyhow!("Failed to list Docker images"));
                }

                let output_str = str::from_utf8(&output.stdout)?;
                let image_exists = output_str
                    .lines()
                    .filter(|line| !line.starts_with("REPOSITORY"))
                    .any(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        parts.get(0).map_or(false, |repo| *repo == image.split(':').next().unwrap_or(""))
                            && parts.get(1).map_or(false, |tag| *tag == image.split(':').nth(1).unwrap_or("latest"))
                    });

                if image_exists {
                    println!("{} {} {}","Image already exists locally:".green(), image,"No need to pull Docker image".cyan());
                } else {
                    println!("Pulling Docker image: {}", image);
                    let pull_output = Command::new("docker")
                        .arg("pull")
                        .arg(image)
                        .output();

                    match pull_output {
                        Ok(pull_output) => {
                            if !pull_output.status.success() {
                                let error_message = str::from_utf8(&pull_output.stderr)?;
                                println!("{}", format!("Failed to pull Docker image: {}", error_message).red());
                                return Err(anyhow!("Failed to pull Docker image: {}", error_message));
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to execute docker pull command: {}", e);
                            return Err(anyhow!("Failed to execute docker pull command: {}", e));
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to execute docker image ls command: {}", e);
                return Err(anyhow!("Failed to execute docker image ls command: {}", e));
            }
        }

        Ok(())
    }




    // Method to mount the playbook files into the container
    fn mount_playbook(&self, container_id: &str) {
        // Implement logic to mount playbook into the Docker container
    }

    // Method to run the playbook inside the container
    fn run_playbook(&self, container_id: &str) -> (i32, String) {
        let exit_code = 0; // Dummy exit code
        let output = format!("Dummy output for container {}", container_id);
        (exit_code, output)
    }

    // Main method to run the tests
    // Additional method to parse the playbook and return ScriptModel
    fn parse_playbook(&self) -> ScriptModel {
        // Placeholder implementation
        ScriptModel {
            steps: vec![], // Empty steps
            env: Some(OS::Linux), // Dummy environment, adjust as per your OS enum
            description: Some("Dummy description".to_string()),
            params: None, // No parameters
        }
    }

    // Main method to run the tests
// Main method to run the tests
    async fn run(&self) -> Result<Vec<(String, i32, String)>, Error> {
        println!("{}", "Running tests...".green());
        let mut results = Vec::new();

        // Create a new Tokio runtime
        // let rt = Runtime::new().unwrap();

        for os_image in &self.os_list {
            println!("{} on OS Image: {}", "Testing".cyan(), os_image.yellow());

            let docker_image = self.packages_manager.get_docker_image_for_os(os_image);

            let pull_result = TestRunner::pull_docker_image(&docker_image);


            // Handle the result of the pull_docker_image call
            if let Err(e) = pull_result {
                eprintln!("Error pulling Docker image: {}", e);
                return Err(e); // Return the error, stopping the program
            }
            // Create and start the container
            let container_id = docker::create_and_start_container(&os_image, &format!("{}_container", self.playbook)).await?;

            // Construct the path to the YAML file
            let yaml_path = self.construct_yaml_path();
            docker::run_playbook_on_docker(&yaml_path, &container_id, &docker_image).await?;


        }

        println!("{}", "Tests completed.".green());
        Ok(results)
    }



}
// Function to handle the 'test' subcommand
pub async fn test_playbook_command(args: Vec<String>, packages_manager: &PackagesManager) -> Result<(), Error> {
    let default_images = HashMap::from([
        ("Linux", "ubuntu:latest"),
        ("macOS", "sickcodes/docker-osx"),
        ("Windows", "mcr.microsoft.com/windows/servercore:ltsc2019"),
    ]);

    let selected_playbook;
    let selected_os_image;

    if args.is_empty() {
        // Interactive mode
        let playbooks = get_available_playbooks(packages_manager);
        let playbook_selection = Select::new()
            .with_prompt("Select a playbook to test")
            .items(&playbooks)
            .default(0)
            .interact()?;
        selected_playbook = playbooks.get(playbook_selection).unwrap().to_string();

        let os_options = vec!["Linux", "Windows", "macOS", "Custom"];
        let os_selection = Select::new()
            .with_prompt("Select an OS to test on")
            .items(&os_options)
            .default(0)
            .interact()?;

        let selected_os = match os_options.get(os_selection).unwrap() {
            &"Custom" => Input::<String>::new()
                .with_prompt("Enter custom OS or Docker image")
                .allow_empty(true)
                .interact_text()?,
            os => os.to_string(),
        };

        selected_os_image = default_images.get(selected_os.as_str()).unwrap_or(&"ubuntu:latest");
    } else {
        selected_playbook = args.get(0).cloned().unwrap_or_default();
        let os = determine_os_from_playbook(&selected_playbook)?;
        selected_os_image = default_images.get(os.as_str()).unwrap_or(&"ubuntu:latest");
    }

    let test_runner = TestRunner::new(selected_playbook, vec![selected_os_image.to_string()], packages_manager);
    test_runner.run().await?;

    Ok(())
}

// Implement the `determine_os_from_playbook` function to extract the OS from the playbook's .yaml file
// If no OS is specified in the file, prompt the user to select one


fn determine_os_from_playbook(playbook: &str) -> Result<String, Error> {
    // Initialize settings
    let settings = Settings::new(None, None);

    // Construct the path to the playbook's YAML file using the runtime_dir
    let yaml_file_path = settings.runtime_dir.join(playbook).join("cocmd.yaml");

    let contents = std::fs::read_to_string(yaml_file_path)
        .map_err(|err| anyhow!("Error reading YAML file: {}", err))?;

    let yaml: serde_yaml::Value = serde_yaml::from_str(&contents)
        .map_err(|err| anyhow!("Error parsing YAML: {}", err))?;

    // Navigate through the 'automations' array
    if let Some(automations) = yaml.get("automations").and_then(|v| v.as_sequence()) {
        for automation in automations {
            // Access the 'content' field and then the 'env' field
            if let Some(content) = automation.get("content") {
                if let Some(env) = content.get("env").and_then(|v| v.as_str()) {
                    println!("{}: {}", "Detected OS".green(), env.yellow());
                    return Ok(env.to_string());
                }
            }
        }
        Err(anyhow!("'env' field not found in any of the automations"))
    } else {
        Err(anyhow!("'automations' field not found in the playbook's YAML file"))
    }
}

impl PackagesManager {
    pub fn get_docker_image_for_os(&self, os_name: &str) -> String {
        match os_name {
            "Linux" => "ubuntu:latest".to_string(),
            "macOS" => "sickcodes/docker-osx".to_string(),
            "Windows" => "mcr.microsoft.com/windows/servercore:ltsc2019".to_string(),
            // Handle custom or other cases
            _ => "ubuntu:latest".to_string(), // Default or custom handling
        }
    }
}

// Dummy function to represent checking if a playbook is installed
// Replace this with your actual logic to check installed playbooks
fn is_playbook_installed(playbook: &str) -> bool {
    // Implement the logic to check if the playbook is installed
    // This is a placeholder function
    true
}

// Function to get the names of all available playbooks
pub fn get_available_playbooks(packages_manager: &PackagesManager) -> Vec<String> {
    packages_manager
        .packages
        .values()
        .map(|package| package.name().to_string())
        .collect()
}
