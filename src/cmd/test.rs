use crate::core::models::script_model::ScriptModel;
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
            .output()?;

        if !output.status.success() {
            eprintln!("{}", "Failed to list Docker images".red());
            return Err(anyhow!("Failed to list Docker images"));
        }

        let output_str = str::from_utf8(&output.stdout)?;
        let image_exists = output_str
            .lines()
            .filter(|line| !line.starts_with("REPOSITORY")) // Skip the header line
            .any(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                parts.get(0).map_or(false, |repo| *repo == image.split(':').next().unwrap_or(""))
                    && parts.get(1).map_or(false, |tag| *tag == image.split(':').nth(1).unwrap_or("latest"))
            });

        if image_exists {
            println!("Image already exists locally: {}", image);
        } else {
            println!("Pulling Docker image: {}", image);
            // Logic to pull the image if it doesn't exist
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
    fn run(&self) -> Vec<(String, i32, String)> {
        println!("{}", "Running tests...".green());
        let mut results = Vec::new();

        // Create a new Tokio runtime
        let rt = Runtime::new().unwrap();

        for os in &self.os_list {
            let display_os = match os.as_str() {
                "sickcodes/docker-osx" => "OSX (macOS)",
                "ubuntu:latest" => "Linux",
                "microsoft-windows" => "Windows",
                _ => os, // Default case, if no mapping is found
            };

            println!("{} on OS: {}", "Testing".cyan(), display_os.yellow());

            // Correctly call the associated function and handle the Future
            rt.block_on(async {
                TestRunner::pull_docker_image(os).expect("Failed to pull Docker image");
            });

            // Here you would typically create a Docker container and get its ID
            let container_id = "dummy_container_id"; // Placeholder for actual container ID

            // Mount the playbook into the container
            self.mount_playbook(container_id);

            // Run the playbook inside the container
            let (exit_code, output) = self.run_playbook(container_id);

            // Collect the results
            results.push((os.clone(), exit_code, output));
        }

        println!("{}", "Tests completed.".green());
        results
    }


}
// Function to handle the 'test' subcommand
pub fn test_playbook_command(args: Vec<String>, packages_manager: &PackagesManager) -> Result<(), Error> {

    // Initialize the default images HashMap
    let mut default_images = HashMap::new();
    default_images.insert("linux", "ubuntu:latest");
    default_images.insert("osx", "sickcodes/docker-osx");
    default_images.insert("windows", "microsoft-windows");

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

         //let os = determine_os_from_playbook(&playbook)?; // Determine OS

        // Create a new TestRunner instance with packages_manager reference
       // let test_runner = TestRunner::new(playbook.to_string(), vec![os], packages_manager);

        // ... [Run tests and collect results]
    }

    match args.len() {
        0 => {
            // No arguments provided, open TUI for browsing packages
            // Implement TUI logic here
            // After package selection, prompt for OS or Docker image
            // Example: let selected_package = "example_package";
            // Example: let selected_os = "linux"; // or user-selected OS
        },
        1 => {

            // One argument provided, which is the playbook name
            let playbook = &args[0];

            // Check if the playbook is installed
            if !is_playbook_installed(playbook) {
                eprintln!("{}", format!("No playbook found with the name '{}'", playbook).red());
                return Err(anyhow!("No playbook found with the name '{}'", playbook));

            }

            // Determine OS from playbook's .yaml file or prompt user
            let os = determine_os_from_playbook(&playbook)?; // Implement this function
            let test_runner = TestRunner::new(playbook.to_string(), vec![os], packages_manager);

        },
        _ => {
            // More than one argument provided or unexpected argument
            eprintln!("{}", "Error: unexpected argument found".red());
            eprintln!("{}", "Usage: cocmd test [PLAYBOOK_NAME]".green());
            return Err(anyhow!("Invalid number of arguments. Usage: cocmd test [PLAYBOOK_NAME]"));
        },    }

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
