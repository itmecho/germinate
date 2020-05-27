//! This crate provides a method of injecting variables from multiple external sources into a
//! template string. Sources can be anything as long as they implement the
//! [`Loader`](https://docs.rs/germinate/*/germinate/trait.Loader.html) trait which handles the
//! loading of the variables in a standard way.
//!
//! # Sources
//! ## Built In
//! These are the currently implemented sources and their associated template keys
//!
//! | Source | Key | Description |
//! |-|-|-|
//! | [AWS EC2 Instance Tags](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | `awsec2tag` | Load the value of AWS EC2 Instance Tags by their key |
//! | [AWS EC2 Metadata Service](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instancedata-data-retrieval.html) | `awsec2metadata` | Load a value from the AWS EC2 Metadata Service by it's path |
//! | Environment Variables | `env` | Load the value of an environment variable |
//!
//! ### Example
//! ```rust
//! # use germinate::Seed;
//! # use std::error::Error;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! # std::env::set_var("NAME", "John");
//! let mut seed = Seed::new("Hi %env:NAME%!".to_string());
//! let output = seed.germinate().await?;
//!
//! assert_eq!("Hi John!", output);
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Sources
//! You can also include your own sources using the
//! [`Seed::add_custom_loader`](https://docs.rs/germinate/*/germinate/struct.Seed.html#method.add_custom_loader)
//! method. The only requirement is that the custom loader must implement the
//! [`Loader`](https://docs.rs/germinate/*/germinate/trait.Loader.html) trait
//!
//! ### Example
//! ```
//! # use germinate::{Seed, Loader};
//! # use std::error::Error;
//! # struct NameLoader {}
//! # #[async_trait::async_trait]
//! # impl Loader for NameLoader {
//! #     async fn load(&self, key: &str) -> anyhow::Result<String> {
//! #         Ok(String::from("John"))
//! #     }
//! # }
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn Error>>{
//! let mut seed = Seed::new("Hi %name:name%".to_string());
//!
//! // Add a custom loader for the name key. This is the loader that will be used whenever
//! // germinate finds %name:...% in the template string
//! seed.add_custom_loader("name".to_string(), Box::new(NameLoader {}));
//!
//! let output = seed.germinate().await?;
//!
//! assert_eq!("Hi John", output);
//! # Ok(())
//! # }
//! ```
#[deny(missing_docs)]
pub(crate) mod loader;
pub(crate) mod seed;

pub use loader::Loader;
pub use seed::Seed;
