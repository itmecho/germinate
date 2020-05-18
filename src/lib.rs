//! A templating library which supports injecting variables from multiple sources. Examples of this
//! would be templating a file and injecting variables from both the environment and an external
//! location such as [AWS Systems Manager Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html)
pub mod awsssm;
pub mod env;

use std::collections::HashMap;

use anyhow::{Context, Result};
use regex::Regex;

/// A type implementing the ValueLoader trait can be used to load a value from a store by it's key
///
/// For example, the type could load a value from and environment variable, or an external key
/// value store like `etcd`.
///
/// As the value could be loaded from an external network source, it must be done asynchronously to
/// allow non-blocking value loading
#[async_trait::async_trait]
pub trait ValueLoader {
    async fn load(&self, key: &String) -> Result<String>;
}

/// This type represents the sources where a value can be loaded from
#[derive(Debug, PartialEq)]
pub enum ValueSource {
    /// This variant represents a value that it loaded from an environment variable
    Environment,

    /// This variant represents a value that is loaded from [AWS System Manager Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html)
    AwsSsm,
}

impl ValueSource {
    /// Creates a new `ValueSource` from the given name. The name is the string used in the template
    /// to define which loader to use when loading the value
    ///
    /// # Examples
    /// ```
    /// // template string: "Hello %env:NAME%!"
    /// use germinate::ValueSource;
    /// let source = ValueSource::from("env").unwrap();
    /// assert_eq!(ValueSource::Environment, source);
    /// ```
    pub fn from<T: AsRef<str>>(key: T) -> Result<Self> {
        Ok(match key.as_ref() {
            "env" => Self::Environment,
            "awsssm" => Self::AwsSsm,
            s => return Err(anyhow::anyhow!("unsupported value source: {}", s)),
        })
    }
}

/// This is the main entrypoint to the library. It takes a `String` which contains template strings
/// and individually loads and replaces the template strings with the value they represent
///
/// # Examples
/// ```
/// #[tokio::main]
/// async fn main() {
///     std::env::set_var("ORC_NAME", "Nigel");
///     let input = String::from("Rest in peace, %env:ORC_NAME%.");
///     let output = germinate::process(&input).await.unwrap();
///     assert_eq!("Rest in peace, Nigel.", output);
/// }
/// ```
pub async fn process<T: AsRef<str>>(input: T) -> Result<String> {
    let replacements = parse(&input).await?;

    let mut output = input.as_ref().to_string();

    for (find, replace) in replacements {
        output = output.replace(&find, &replace);
    }

    Ok(output)
}

/// Parses the input string for template strings and returns a `HashMap` of template strings and
/// their associated values
///
/// # Examples
///
/// ```
/// #[tokio::main]
/// async fn main() {
///     std::env::set_var("MY_VAR", "success");
///     let mut s = String::from("The value of MY_VAR is %env:MY_VAR%");
///     let replacements = germinate::parse(&s).await.unwrap();
//
///     for (find, replace) in replacements {
///         s = s.replace(&find, &replace);
///     };
///
///     assert_eq!("The value of MY_VAR is success", s);
/// }
/// ```
pub async fn parse<T: AsRef<str>>(input: T) -> Result<HashMap<String, String>> {
    // TODO look into lazy_static to ensure this is only compiled once
    let pattern = Regex::new(r"(%([a-z]+):([^%]+)%)").unwrap();

    let mut replacements = HashMap::new();

    for capture in pattern.captures_iter(input.as_ref()) {
        // capture[1] will be the find string. If the map contains the key then we have already
        // processed this replacement
        if replacements.contains_key(&capture[1].to_string()) {
            continue;
        }

        let source = ValueSource::from(&capture[2]).context("Failed to parse template file")?;
        let key = &capture[3];

        // TODO store this in a map as we will reinitialise loaders if they have been used in a
        // previous iteration
        let loader: Box<dyn ValueLoader> = match source {
            ValueSource::Environment => Box::new(env::EnvironmentLoader::new()),
            ValueSource::AwsSsm => Box::new(awsssm::AwsSsmLoader::new()),
        };

        let value = loader
            .load(&key.to_string())
            .await
            .context("Failed to load value")?;

        replacements.insert(capture[1].to_string(), value);
    }

    Ok(replacements)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_process_env_only() {
        use super::process;

        std::env::set_var("ORC_TEST_VAR_1", "test 1");
        std::env::set_var("ORC_TEST_VAR_2", "test 2");

        let expected = String::from("var 1: test 1, var 1: test 1, var 2: test 2");

        let input = String::from(
            "var 1: %env:ORC_TEST_VAR_1%, var 1: %env:ORC_TEST_VAR_1%, var 2: %env:ORC_TEST_VAR_2%",
        );

        let actual = process(input).await.unwrap();

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_parse() {
        use super::parse;

        std::env::set_var("ORC_TEST_VAR_1", "test 1");
        std::env::set_var("ORC_TEST_VAR_2", "test 2");

        let mut expected: HashMap<String, String> = HashMap::new();
        expected.insert("%env:ORC_TEST_VAR_1%".into(), "test 1".into());
        expected.insert("%env:ORC_TEST_VAR_2%".into(), "test 2".into());

        let input = String::from(
            "var 1: %env:ORC_TEST_VAR_1%, var 1: %env:ORC_TEST_VAR_1%, var 2: %env:ORC_TEST_VAR_2%",
        );

        let actual = parse(input).await.unwrap();

        assert_eq!(expected, actual);
    }
}
