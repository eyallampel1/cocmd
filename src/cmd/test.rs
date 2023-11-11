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
pub fn test_playbook_command(args: Vec<String>) -> Result<(), Error> {
    // Initialize the default images HashMap
    let mut default_images = HashMap::new();
    default_images.insert("linux", "ubuntu:latest");
    default_images.insert("osx", "sickcodes/docker-osx");
    default_images.insert("windows", "microsoft-windows");

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

            // Create a new TestRunner instance
            let test_runner = TestRunner::new(playbook.to_string(), vec![os]);

            // Run the tests and collect the results
            let results = test_runner.run();

            // Process and display the results
            for (os, exit_code, output) in results {
                println!("OS: {}, Exit Code: {}, Output: {}", os, exit_code, output);
                // Logic to determine pass/fail based on exit code
            }
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
    // Example logic to read the playbook's .yaml file
    // You'll need to replace this with your actual file reading and parsing logic

    // Placeholder for the path to the playbook's .yaml file
    let yaml_file_path = format!("/path/to/{}.yaml", playbook);

    let contents = std::fs::read_to_string(yaml_file_path)
        .map_err(|err| {
            // Transform the error into a custom format, but don't print it here
            anyhow!("Error reading YAML file: {}", err)
        })?;


// Parse the YAML content to find the 'env' field
// This is a simplified example, adjust according to your YAML structure
    let yaml: serde_yaml::Value = serde_yaml::from_str(&contents)
        .map_err(|err| {
            eprintln!("{}", format!("Error parsing YAML: {}", err).red());
            anyhow!("Error parsing YAML: {}", err)
        })?;


    if let Some(env) = yaml.get("env").and_then(|v| v.as_str()) {
        // Return the OS specified in the 'env' field
        Ok(env.to_string())
    } else {
        // If 'env' field is not found, prompt the user to select an OS
        // Implement the logic for prompting the user
        // Example: Ok("linux".to_string()) // Return the user-selected OS
        Err(anyhow!("'env' field not found in the playbook's YAML file"))
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