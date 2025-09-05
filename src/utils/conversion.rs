use crate::EpcisKgError;

pub struct FormatConverter;

impl FormatConverter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn turtle_to_jsonld(&self, turtle_data: &str) -> Result<String, EpcisKgError> {
        todo!("Implement Turtle to JSON-LD conversion")
    }
    
    pub fn jsonld_to_turtle(&self, jsonld_data: &str) -> Result<String, EpcisKgError> {
        todo!("Implement JSON-LD to Turtle conversion")
    }
}