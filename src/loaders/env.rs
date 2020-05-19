//! Provides the ability to load values from the environment
use anyhow::Result;

pub(crate) const TEMPLATE_KEY: &'static str = "env";

/// This type provides functionality to load values from environment variables
pub struct EnvironmentLoader {}

impl EnvironmentLoader {
    pub fn new() -> Self {
        EnvironmentLoader {}
    }
}

#[async_trait::async_trait]
impl crate::ValueLoader for EnvironmentLoader {
    /// Load a value from the environment. The key is the name of the environment variable
    /// containing the value
    async fn load(&self, key: &String) -> Result<String> {
        Ok(std::env::var(&key)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ValueLoader;

    #[tokio::test]
    async fn test_environment_loader() {
        let expected = String::from("success");
        let key = String::from("ORC_TEST_VAR");
        std::env::set_var(&key, &expected);

        let loader = EnvironmentLoader::new();
        let actual = loader.load(&key).await.unwrap();

        assert_eq!(expected, actual);
    }
}
