use crate::core::models::script_model::ScriptModel;
use crate::core::utils::sys::OS;
use std::collections::HashMap;
use anyhow::{Error, anyhow};
use bollard::image::{CreateImageOptions};
use bollard::Docker;
use std::default::Default;
use tokio::runtime::Runtime;
use futures::stream::StreamExt;
use colored::*;
use bollard::image::ListImagesOptions;
use std::process::Command;
use std::str;


struct TestRunner {
    playbook: String,
    os_list: Vec<String>,
}

impl TestRunner {
    // Constructor
    fn new(playbook: String, os_list: Vec<String>) -> TestRunner {
        println!("{} with playbook: {}", "Creating new TestRunner".green(), playbook.yellow());
        TestRunner { playbook, os_list }
    }


    // Method to pull the appropriate Docker image for the OS
    fn pull_docker_image(image: &str) -> Result<(), Error> {
        let output = Command::new("docker")
            .arg("image")
            .arg("ls")
            .output()?;

        if !output.status.success() {
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
pub fn test_playbook_command(playbook: String, os: Option<String>, docker_image: Option<String>) -> Result<(), Error> {
    // Check if the OS is valid
    let valid_os = ["linux", "osx", "windows", "all"];
    if let Some(ref os_key) = os {
        if !valid_os.contains(&os_key.as_str()) {
            return Err(anyhow!("Invalid OS specified. Allowed values are: linux, osx, windows, all."));
        }
    }

    // Check if the playbook is installed (assuming a function `is_playbook_installed`)
    if !is_playbook_installed(&playbook) {
        return Err(anyhow!("No playbook found with the name '{}'", playbook));
    }

    println!("{} called with playbook: {}", "test_playbook_command".green(), playbook.yellow());

    // Initialize the default images HashMap
    let mut default_images = HashMap::new();
    default_images.insert("linux", "ubuntu:latest");
    default_images.insert("osx", "sickcodes/docker-osx");
    default_images.insert("windows", "microsoft-windows");

    // Determine the Docker image to use based on the OS and the docker_image argument
    let docker_image_to_use = match (os.as_deref(), docker_image.as_deref()) {
        (Some(os_key), None) => {
            // No custom Docker image provided, use the default for the specified OS
            default_images.get(os_key).ok_or_else(|| anyhow!("No valid OS specified"))?.to_string()
        },
        (_, Some(custom_image)) => {
            // Custom Docker image provided, use it
            custom_image.to_string()
        },
        _ => return Err(anyhow!("OS must be specified if no custom Docker image is provided")),
    };

    let os_identifier = os.unwrap_or_else(|| "Custom".to_string()); // Adjust this identifier as needed

    // Create a new TestRunner instance with the correct OS identifier
    let test_runner = TestRunner::new(playbook, vec![os_identifier]);

    // Run the tests and collect the results
    let results = test_runner.run();

    // Process and display the results
    for (os, exit_code, output) in results {
        println!("OS: {}, Docker Image: {}, Exit Code: {}, Output: {}",
                 os.yellow(),
                 docker_image_to_use.cyan(),
                 exit_code.to_string().magenta(),
                 output.cyan());
        // Logic to determine pass/fail based on exit code
    }

    // Indicate success without an error
    Ok(())
}

// Dummy function to represent checking if a playbook is installed
// Replace this with your actual logic to check installed playbooks
fn is_playbook_installed(playbook: &str) -> bool {
    // Implement the logic to check if the playbook is installed
    // This is a placeholder function
    true
}

// Example usage of the test_playbook_command function:
// 1. Test with a specified playbook on all default OSes:
//    cocmd test rust
// 2. Test with a specified playbook on a specific OS (Linux):
//    cocmd test rust linux
// 3. Test with a specified playbook, specific OS (Linux), and a custom Docker image:
//    cocmd test rust linux custom/docker-image-link
// 4. Test with a specified playbook on a specific OS (Windows):
//    cocmd test rust windows
// 5. Test with a specified playbook on a specific OS (OSX):
//    cocmd test rust osx