#[path = "../../../../crates/client_runner/src/vulkan.rs"]
mod vulkan;

fn main() -> std::process::ExitCode {
    match vulkan::check_vulkan() {
        Ok(()) => {
            println!("Vulkan 1.1 instance and a graphics queue are available.");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
