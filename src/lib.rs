//! A templating library which supports injecting variables from multiple sources. Examples of this
//! would be templating a file and injecting variables from both the environment and an external
//! location such as [AWS Systems Manager Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html)
pub(crate) mod loader;
pub(crate) mod seed;

pub use loader::Loader;
pub use seed::Seed;
