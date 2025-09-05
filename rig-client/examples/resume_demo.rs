use anyhow::Result;
use baml_client::apis::configuration::Configuration;
use baml_client::apis::default_api;
use baml_client::models::ExtractResumeRequest;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Verify environment variables are loaded
    match env::var("OPENAI_API_KEY") {
        Ok(_) => println!("Successfully loaded .env file"),
        Err(e) => println!("Error loading .env file: {}", e),
    }

    // Print API key presence (not the actual keys)
    println!("OpenAI API key present: {}", std::env::var("OPENAI_API_KEY").is_ok());
    println!("Anthropic API key present: {}", std::env::var("ANTHROPIC_API_KEY").is_ok());

    // Create configuration with default settings
    let config = Configuration::new();

    // Sample resume text
    let resume_text = r#"
        John Doe
        john.doe@example.com

        Experience:
        - Senior Developer at Tech Corp
        - Lead Engineer at Startup Inc
        - Software Engineer at First Job Ltd

        Skills:
        - Rust
        - Python
        - Cloud Architecture
    "#;

    // Create the request object
    let request = ExtractResumeRequest::new(resume_text.to_string());

    // Call the BAML-generated ExtractResume function
    let result = default_api::extract_resume(&config, request).await?;

    // Print the extracted resume information
    println!("Extracted Resume Information:");
    println!("Name: {}", result.name);
    println!("Email: {}", result.email);
    println!("\nExperience:");
    for exp in result.experience {
        println!("- {}", exp);
    }
    println!("\nSkills:");
    for skill in result.skills {
        println!("- {}", skill);
    }

    Ok(())
}
