use crate::core::models::script_model::ScriptModel;
use crate::core::utils::sys::OS;
use std::collections::HashMap;
use anyhow::{Error, anyhow};
use bollard::image::{CreateImageOptions, ListImagesOptions};
use bollard::Docker;
use std::default::Default;
use tokio::runtime::Runtime;
use futures::stream::StreamExt;

struct TestRunner {
    playbook: String,
    os_list: Vec<String>,
}

impl TestRunner {
    // Constructor
    fn new(playbook: String, os_list: Vec<String>) -> TestRunner {
        TestRunner { playbook, os_list }
    }

    // Method to pull the appropriate Docker image for the OS
    // Method to pull the appropriate Docker image for the OS
    fn pull_docker_image(&self, image: &str) {
        // Create a new Tokio runtime
        let rt = Runtime::new().unwrap();

        // Execute the async block using the runtime
        rt.block_on(async {
            let docker = Docker::connect_with_local_defaults().unwrap();

            // Check if image exists locally
            let images = docker.list_images(Some(ListImagesOptions::<String> {
                all: true,
                ..Default::default()
            })).await.unwrap();

            let image_exists = images.iter().any(|i| {
                i.repo_tags.iter().any(|tag| tag == image)
            });

            // If image does not exist, pull from Docker Hub
            if !image_exists {
                let options = CreateImageOptions {
                    from_image: image,
                    ..Default::default()
                };

                let mut stream = docker.create_image(Some(options), None, None);

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(info) => println!("{:?}", info),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }
        });
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
    fn run(&self) -> Vec<(String, i32, String)> {
        let mut results = Vec::new();
        let script_model = self.parse_playbook(); // Parse the playbook to get ScriptModel

        let os_list = match script_model.env {
            Some(os) => vec![os.to_string()],
            None => vec!["linux".to_string(), "windows".to_string(), "osx".to_string()],
        };

        for os in os_list {
            // Existing logic to run tests
        }

        results
    }
}
// Function to handle the 'test' subcommand
pub fn test_playbook_command(playbook: String, os: Option<String>, docker_image: Option<String>) -> Result<(), Error> {
    // Initialize the default images HashMap
    let mut default_images = HashMap::new();
    default_images.insert("linux", "ubuntu");
    default_images.insert("osx", "sickcodes/docker-osx");
    default_images.insert("windows", "microsoft-windows");

// Determine the Docker image to use
    let image = match (os.as_deref(), docker_image) {
        (Some(os_key), None) => default_images.get(os_key).ok_or_else(|| anyhow!("No valid OS specified"))?.to_string(),
        (_, Some(custom_image)) => custom_image,
        _ => return Err(anyhow!("No valid OS or Docker image specified")),
    };

// Create a new TestRunner instance
    let test_runner = TestRunner::new(playbook, vec![image]);

    // Run the tests and collect the results
    let results = test_runner.run();

    // Process and display the results
    for (os, exit_code, output) in results {
        println!("OS: {}, Exit Code: {}, Output: {}", os, exit_code, output);
        // Logic to determine pass/fail based on exit code
    }

    // Indicate success without an error
    Ok(())
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