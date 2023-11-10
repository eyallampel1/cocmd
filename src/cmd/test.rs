use crate::core::models::script_model::ScriptModel;
use crate::core::utils::sys::OS;

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
    fn pull_docker_image(&self, os: &str) {
        // Implement logic to pull Docker image based on `os`
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

    // Function to handle the 'test' subcommand
    pub fn test_playbook_command(playbook: String) {
        let test_runner = TestRunner::new(playbook, Vec::new()); // os_list is now determined inside run()
        let results = test_runner.run();

        for (os, exit_code, output) in results {
            println!("OS: {}, Exit Code: {}, Output: {}", os, exit_code, output);
            // Logic to determine pass/fail based on exit code
        }
    }
}
