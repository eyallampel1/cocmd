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
    fn run(&self) -> Result<Vec<(String, i32, String)>, Error> {
        println!("{}", "Running tests...".green());
        let mut results = Vec::new();

        // Create a new Tokio runtime
        // let rt = Runtime::new().unwrap();

        for os_image in &self.os_list {
            println!("{} on OS Image: {}", "Testing".cyan(), os_image.yellow());

            let pull_result = TestRunner::pull_docker_image(os_image);


            // Handle the result of the pull_docker_image call
            if let Err(e) = pull_result {
                eprintln!("Error pulling Docker image: {}", e);
                return Err(e); // Return the error, stopping the program
            }

            // Here you would typically create a Docker container and get its ID
            let container_id = "dummy_container_id"; // Placeholder for actual container ID

            // Mount the playbook into the container
            self.mount_playbook(container_id);

            // Run the playbook inside the container
            let (exit_code, output) = self.run_playbook(container_id);

            // Collect the results
            results.push((os_image.to_string(), exit_code, output));
        }

        println!("{}", "Tests completed.".green());
        Ok(results)
    }



}
// Function to handle the 'test' subcommand
pub async fn test_playbook_command(args: Vec<String>, packages_manager: &PackagesManager) -> Result<(), Error> {

    let mut selected_playbook = String::new();

    // Initialize the default images HashMap
    let mut default_images = HashMap::new();
    default_images.insert("Linux", "ubuntu:latest");
    default_images.insert("macOS", "sickcodes/docker-osx");
    default_images.insert("Windows", "microsoft-windows");

    if args.is_empty() {
        // Interactive mode
        // Prompt for playbook selection
        let playbooks = get_available_playbooks(packages_manager); // Implement this function to list available playbooks
        let playbook_selection = Select::new()
            .with_prompt("Select a playbook to test")
            .items(&playbooks)
            .default(0)
            .interact()?;

         selected_playbook = playbooks.get(playbook_selection).unwrap().to_string();

        // Prompt for OS selection (optional)
        let os_options = vec!["Linux", "Windows", "macOS", "Custom"];
        let os_selection = Select::new()
            .with_prompt("Select an OS to test on ")
            .items(&os_options)
            .default(0)
            .interact()?;

        let selected_os = match os_options.get(os_selection).unwrap() {
            &"Custom" => {
                let custom_os = Input::<String>::new()
                    .with_prompt("Enter custom OS or Docker image")
                    .allow_empty(true)
                    .interact_text()?;
                custom_os
            }
            os => os.to_string(),
        };

        // Define selected_os_image here
        let selected_os_image: &str = default_images.get(selected_os.as_str()).unwrap_or(&selected_os.as_str());


        // Run the test with the selected options
        let test_runner = TestRunner::new(selected_playbook.clone(), vec![selected_os_image.to_string()], packages_manager);
        let results = test_runner.run()?;
        // Process results as needed

        // Process results as needed
    }

    if args.len() == 1 {
        let playbook = &args[0];

        // Check if the playbook is installed
        let package = match packages_manager.get_package(playbook.to_string()) {
            Some(pkg) => pkg,
            None => {
                eprintln!("Package '{}' is not installed.", playbook);
                return Err(anyhow!("Package '{}' is not installed.", playbook));
            }
        };

        let os = determine_os_from_playbook(&playbook)?;

        // Convert the OS string to a `&str` slice
        let os_slice = os.as_str();

        // Create a reference to the selected OS image
        let selected_os_image: &str = default_images.get(os_slice).unwrap_or(&os_slice);

        let test_runner = TestRunner::new(playbook.to_string(), vec![selected_os_image.to_string()], packages_manager);

        // Run tests and collect results
        let results = test_runner.run();
        // Process results as needed
        // Run tests and collect results

    }

    match args.len() {
        0 => {
            let runtime_dir = &packages_manager.settings.runtime_dir;
            let playbook_path = Path::new(runtime_dir);
            let cocmd_yaml_path = playbook_path.join(&selected_playbook).join("cocmd.yaml");


            let cocmd_yaml_path_str = cocmd_yaml_path.to_str().unwrap_or_default();
            println!("Path to cocmd.yaml: {}", cocmd_yaml_path_str.yellow());

            // Read and parse the YAML file
            let yaml_content = fs::read_to_string(cocmd_yaml_path_str)
                .expect("Failed to read YAML file");

            let rt = Runtime::new()?;

            // Use the runtime to await the async function
            docker::run_playbook_on_docker(cocmd_yaml_path_str).await?;


            // Run interactive shell mode
            // Implement the logic for interactive mode here
            println!("{}","Interactive test mode not implemented yet".red());
        },
        1 => {

            println!("{} {}","Testing playbook:".green(), args[0].yellow());
            // One argument: playbook name
            let playbook_name = &args[0];
            // Implement the logic for testing with the specified playbook on all default OSes

            }
        2 => {

            // print Two arguments: playbook name and OS
            println!("{} {} {} {}","Testing playbook:".green(), args[0].yellow(), "on OS:".green(), args[1].yellow());
            let playbook_name = &args[0];
            let os = &args[1];
            // Implement the logic for testing with the specified playbook on the specified OS
        },
        3 => {
            // Three arguments: playbook name, OS, and Docker image
            println!("{} {} {} {} {} {}","Testing playbook:".green(), args[0].yellow(), "on OS:".green(), args[1].yellow(), "with Docker image:".green(), args[2].yellow());
            let playbook_name = &args[0];
            let os = &args[1];
            let docker_image = &args[2];
            // Implement the logic for testing with the specified playbook, OS, and Docker image
        },
        _ => {
            // More than three arguments or unexpected argument
            return Err(anyhow!("Invalid number of arguments."));
        }
    }

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

// Implement the TUI logic for browsing packages and selecting an OS or Docker image
// This part will depend on your existing TUI implementation


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
