//! Provides the ability to asynchronously load values from [AWS EC2 Metadata](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instancedata-data-retrieval.html)
//!
//! # Examples
//!
//! ```ignore
//! let mut seed = germinate::Seed::new(String::from("Instance ID: %awsec2metadata:instance-id%"));
//! let output = seed.germinate().await.unwrap();
//! assert_eq!(String::from("Instance ID: i-abcdefgh123456789"), output);
//! ```

// TODO handle different responses (text/json). The metadata service doesn't set the content-type
// header correctly so this would most likely have to be handled on a case by case basis
use anyhow::Result;

pub(crate) const TEMPLATE_KEY: &str = "awsec2metadata";
pub(crate) const METADATA_BASE_URL: &str = "http://169.254.169.254/latest/meta-data";

/// This type provides functionality for loading values from [AWS EC2 Metadata](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instancedata-data-retrieval.html)
pub struct AwsEc2MetadataLoader<'a> {
    metadata_url: &'a str,
}

impl<'a> AwsEc2MetadataLoader<'a> {
    /// Creates a new AwsEc2MetadataLoader with the default AWS EC2 Metadata service endpoint
    pub fn new() -> Self {
        Self::with_base_url(METADATA_BASE_URL)
    }

    pub fn with_base_url(url: &'a str) -> Self {
        Self { metadata_url: url }
    }
}

pub(crate) async fn get_metadata_value(base_url: &str, path: &str) -> Result<String> {
    // The following should handle and combination of trailing slash on the base_url with a /
    // prefix on the path
    let mut url = String::from(base_url);
    if !url.ends_with('/') {
        url.push('/');
    }

    url.push_str(path.trim_start_matches('/'));

    // This seems overly complex, there's probably a better way
    let value = surf::get(url)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e).context("Failed to load metadata value"))?
        .body_string()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e).context("Failed to decode response body"))?;

    Ok(value)
}

#[async_trait::async_trait]
impl crate::Loader for AwsEc2MetadataLoader<'_> {
    /// Loads a value from the AWS EC2 Metadata service and returns it as a `String`
    async fn load(&self, key: &str) -> Result<String> {
        get_metadata_value(self.metadata_url, key).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Loader;
    use mockito::mock;

    #[tokio::test]
    async fn test_aws_ec2_metadata_basic() {
        let expected = "test-id";

        let _m = mock("GET", "/instance-id")
            .with_status(200)
            .with_header("Content-Type", "text/plain")
            .with_body(expected)
            .create();

        let mut url = mockito::server_url();
        url.push('/');
        let loader = AwsEc2MetadataLoader::with_base_url(&url);

        let actual = loader.load("instance-id").await;

        assert_eq!(expected, actual.unwrap());
    }
}
