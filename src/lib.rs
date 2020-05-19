//! A templating library which supports injecting variables from multiple sources. Examples of this
//! would be templating a file and injecting variables from both the environment and an external
//! location such as [AWS Systems Manager Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html)
pub(crate) mod loaders;
pub mod seed;

pub use seed::Seed;

use loaders::awsssm;
use loaders::env;

use anyhow::Result;

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
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub(crate) enum ValueSource {
    /// This variant represents a value that is loaded from [AWS System Manager Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html)
    AwsSsm,

    /// This variant represents a value that it loaded from an environment variable
    Environment,

    /// This variant allows for custom sources to be used that aren't managed by this library.
    Custom(String),
}

impl ValueSource {
    pub(crate) fn from<T: AsRef<str>>(key: T) -> Self {
        match key.as_ref() {
            awsssm::TEMPLATE_KEY => Self::AwsSsm,
            env::TEMPLATE_KEY => Self::Environment,
            key => Self::Custom(key.to_string()),
        }
    }
}
