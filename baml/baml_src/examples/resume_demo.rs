use anyhow::Result;
use dotenv::dotenv;
use baml::Resume;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Sample resume text
    let resume_text = r#"
        John Doe
        john@example.com

        Experience:
        - Senior Developer at Tech Corp
        - Software Engineer at Code Inc
        - Junior Developer at Startup Ltd

        Skills:
        - Rust
        - TypeScript
        - Python
    "#;

    // Extract resume using our library function
    let resume = baml::extract_resume(resume_text).await?;

    // Print the extracted information
    println!("Extracted Resume Information:");
    println!("Name: {}", resume.name);
    println!("Email: {}", resume.email);
    println!("\nExperience:");
    for exp in resume.experience {
        println!("- {}", exp);
    }
    println!("\nSkills:");
    for skill in resume.skills {
        println!("- {}", skill);
    }

    Ok(())
}