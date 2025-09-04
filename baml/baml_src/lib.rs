use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod examples;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resume {
    pub name: String,
    pub email: String,
    pub experience: Vec<String>,
    pub skills: Vec<String>,
}

pub async fn extract_resume(resume_text: &str) -> Result<Resume> {
    // Initialize the BAML runtime
    let runtime = baml_runtime::BamlRuntime::new();
    
    // Create the function call parameters
    let params = serde_json::json!({
        "resume": resume_text
    });

    // Call the BAML function
    let result = runtime.call_function(
        "ExtractResume",
        baml_runtime::BamlValue::from(params),
        None,
    ).await?;

    // Convert the result to our Resume type
    let resume: Resume = serde_json::from_value(result.value.into())?;
    Ok(resume)
}
