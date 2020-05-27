//! Main module of the library that handles parsing of the input string and loading values from the
//! relevant loaders.
//!
//! Allows for custom loaders to be used via the `add_custom_loader` method
use crate::loader::{Loader, Source};
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::collections::HashMap;

use crate::loader::awsec2metadata::AwsEc2MetadataLoader;
use crate::loader::awsec2tag::AwsEc2TagLoader;
use crate::loader::awsssm::AwsSsmLoader;
use crate::loader::env::EnvironmentLoader;

/// A `Seed` is responsible for parsing the template string, loading the values, and optionally
/// making the replacements via the germinate method
pub struct Seed<'a> {
    template: &'a str,
    loaders: HashMap<Source, Box<dyn Loader>>,
}

impl<'a> Seed<'a> {
    /// Create a new `Seed` with the given template string
    pub fn new(template: &'a str) -> Self {
        Self {
            template,
            loaders: HashMap::new(),
        }
    }

    /// Adds a custom loader to allow users of the library to add their own value sources
    ///
    /// # Example
    /// ```
    /// use germinate::{Seed, Loader};
    ///
    /// struct LanguageLoader {}
    ///
    /// #[async_trait::async_trait]
    /// impl Loader for LanguageLoader {
    ///     async fn load(&self, key: &str) -> anyhow::Result<String> {
    ///         // Add your logic for loading the value here
    ///
    ///         Ok(match key {
    ///             "go" => String::from("Go"),
    ///             _ => String::from("Rust"),
    ///         })
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     std::env::set_var("NAME", "John");
    ///
    ///     let mut seed = Seed::new("Hi %env:NAME%, Welcome to %language:rust%! Say goodbye to %language:go%...");
    ///     seed.add_custom_loader("language".into(), Box::new(LanguageLoader{}));
    ///     let output = seed.germinate().await.unwrap();
    ///
    ///     assert_eq!(String::from("Hi John, Welcome to Rust! Say goodbye to Go..."), output);
    /// }
    /// ```
    pub fn add_custom_loader(&mut self, key: String, loader: Box<dyn Loader>) {
        self.loaders.insert(Source::Custom(key), loader);
    }

    fn get_loader(&mut self, source: &Source) -> Result<&dyn Loader> {
        // If a loader with the given key exists, return it
        if self.loaders.contains_key(source) {
            // Unwrap should be safe here as we know the key exists
            return Ok(self.loaders.get(source).unwrap().as_ref());
        }

        // Instantiate a new loader for the given key. If the key is for a custom source, we return
        // an error as that should have been set using the add_custom_loader function before
        // parsing
        let loader: Box<dyn Loader> = match source {
            Source::AwsEc2Tag => Box::new(AwsEc2TagLoader::new()),
            Source::AwsEc2Metadata => Box::new(AwsEc2MetadataLoader::new()),
            Source::AwsSsm => Box::new(AwsSsmLoader::new()),
            Source::Environment => Box::new(EnvironmentLoader::new()),
            Source::Custom(key) => return Err(
                anyhow!(
                    "Unsupported value source: {}. If you're using a custom source, make sure you added the loader before parsing",
                    key
                    )
                ),
        };

        // Store the new loader
        self.loaders.insert(source.clone(), loader);

        // Return a reference to the newly created loader
        Ok(self.loaders.get(source).unwrap().as_ref())
    }

    /// Parses the template string and generates a `HashMap` of key value replacements, loading the
    /// value for each replacement as it goes. If it finds a template string with a custom source
    /// without a related loader, it will return an error. It will also return an error if a value
    /// fails to load
    ///
    /// # Examples
    /// ```
    /// #[tokio::main]
    /// async fn main() {
    ///     std::env::set_var("NAME", "John");
    ///
    ///     let mut seed = germinate::Seed::new("Hi %env:NAME%, Welcome to Rust!");
    ///     let replacements = seed.parse().await.unwrap();
    ///
    ///     assert_eq!(replacements.get("%env:NAME%").unwrap(), &String::from("John"));
    /// }
    /// ```
    pub async fn parse(&mut self) -> Result<HashMap<String, String>> {
        let mut replacements = HashMap::new();

        let pattern = Regex::new(r"(%([a-z0-9]+):([^%]+)%)").unwrap();

        for capture in pattern.captures_iter(self.template.clone().as_ref()) {
            // capture[1] will be the find string. If the map contains the key then we have already
            // processed this replacement
            if replacements.contains_key(&capture[1].to_string()) {
                continue;
            }

            let source = Source::from(&capture[2]);
            let loader = self
                .get_loader(&source)
                .context("Failed to parse template string")?;

            // This is the key to use when loading the value
            let key = &capture[3];

            let value = loader
                .load(&key.to_string())
                .await
                .context("Failed to load value")?;

            replacements.insert(capture[1].to_string(), value);
        }

        Ok(replacements)
    }

    /// The germinate is a wrapper around the parse function which follows up by actually making
    /// the replacements in the template string and returning the result.
    ///
    /// # Examples
    /// ```
    /// #[tokio::main]
    /// async fn main() {
    ///     std::env::set_var("NAME", "John");
    ///
    ///     let mut seed = germinate::Seed::new("Hi %env:NAME%, Welcome to Rust!");
    ///     let output = seed.germinate().await.unwrap();
    ///
    ///     assert_eq!(String::from("Hi John, Welcome to Rust!"), output);
    /// }
    ///
    /// ```
    pub async fn germinate(&mut self) -> Result<String> {
        let mut output = self.template.to_string();

        for (k, v) in self.parse().await? {
            output = output.replace(&k, &v);
        }

        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use super::Seed;
    use crate::Loader;
    use anyhow::Result;

    struct TestLoader {
        value: String,
    }

    impl TestLoader {
        pub fn with_value(value: String) -> Self {
            Self { value }
        }
    }

    #[async_trait::async_trait]
    impl Loader for TestLoader {
        async fn load(&self, _: &str) -> Result<String> {
            Ok(self.value.clone())
        }
    }

    #[tokio::test]
    async fn test_germinate_basic() {
        std::env::set_var("TEST_VAR", "Test");

        let mut seed = Seed::new("Test %env:TEST_VAR% Test");
        let output = seed.germinate().await.unwrap();

        assert_eq!(String::from("Test Test Test"), output);
    }

    #[tokio::test]
    async fn test_geminate_with_custom_loader() {
        let mut seed = Seed::new("Test %custom:test% Test");
        seed.add_custom_loader(
            "custom".into(),
            Box::new(TestLoader::with_value("Test".into())),
        );
        let output = seed.germinate().await.unwrap();

        assert_eq!(String::from("Test Test Test"), output);
    }
}
