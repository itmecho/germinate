//! Provides the ability to load values from the environment
//!
//! # Examples
//! ```
//! std::env::set_var("TEST_NAME", "John");
//! let mut seed = germinate::Seed::new(String::from("Hi %env:TEST_NAME%"));
//! let output = tokio::runtime::Runtime::new().unwrap().block_on(seed.germinate()).unwrap();
//! assert_eq!("Hi John", output);
//! ```
use anyhow::Result;

pub(crate) const TEMPLATE_KEY: &str = "env";

/// This type provides functionality to load values from environment variables
pub struct EnvironmentLoader {}

impl EnvironmentLoader {
    pub fn new() -> Self {
        EnvironmentLoader {}
    }
}

#[async_trait::async_trait]
impl crate::Loader for EnvironmentLoader {
    /// Load a value from the environment. The key is the name of the environment variable
    /// containing the value
    async fn load(&self, key: &str) -> Result<String> {
        Ok(std::env::var(key)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Loader;

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
