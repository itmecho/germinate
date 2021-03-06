#[cfg(feature = "aws")]
pub(crate) mod awsec2metadata;

#[cfg(feature = "aws")]
pub(crate) mod awsec2tag;

#[cfg(feature = "aws")]
pub(crate) mod awsssm;

pub(crate) mod env;

use anyhow::Result;

/// A type implementing the Loader trait can be used to load a value from a store by it's key
///
/// For example, the type could load a value from and environment variable, or an external key
/// value store like `etcd`.
///
/// As the value could be loaded from an external network source, it must be done asynchronously to
/// allow non-blocking value loading
#[async_trait::async_trait]
pub trait Loader {
    /// Load takes a key and loads a value from the source using the key. As this could be over a
    /// network, we do this asynchronously
    async fn load(&self, key: &str) -> Result<String>;
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub(crate) enum Source {
    #[cfg(feature = "aws")]
    AwsEc2Tag,

    #[cfg(feature = "aws")]
    AwsEc2Metadata,

    #[cfg(feature = "aws")]
    AwsSsm,
    Environment,
    Custom(String),
}

impl Source {
    pub(crate) fn from<T: AsRef<str>>(key: T) -> Self {
        match key.as_ref() {
            #[cfg(feature = "aws")]
            awsec2tag::TEMPLATE_KEY => Self::AwsEc2Tag,

            #[cfg(feature = "aws")]
            awsec2metadata::TEMPLATE_KEY => Self::AwsEc2Metadata,

            #[cfg(feature = "aws")]
            awsssm::TEMPLATE_KEY => Self::AwsSsm,

            env::TEMPLATE_KEY => Self::Environment,

            key => Self::Custom(key.to_string()),
        }
    }
}
