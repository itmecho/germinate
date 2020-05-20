// TODO handle different responses (text/json). The metadata service doesn't set the content-type
// header correctly so this would most likely have to be handled on a case by case basis
use anyhow::Result;

pub(crate) const TEMPLATE_KEY: &str = "awsec2metadata";

pub struct AwsEc2MetadataLoader<'a> {
    metadata_url: &'a str,
}

impl<'a> AwsEc2MetadataLoader<'a> {
    pub fn new() -> Self {
        Self::with_base_url("http://169.254.169.254/latest/meta-data/")
    }

    pub fn with_base_url(url: &'a str) -> Self {
        Self { metadata_url: url }
    }

    async fn get_value(&self, path: &str) -> Result<String> {
        let mut url = String::from(self.metadata_url);
        url.push_str(path);

        // This seems overly complex, there's probably a better way
        let value = surf::get(url)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e).context("Failed to load metadata value"))?
            .body_string()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e).context("Failed to decode response body"))?;

        Ok(value)
    }
}

#[async_trait::async_trait]
impl<'a> crate::ValueLoader for AwsEc2MetadataLoader<'a> {
    async fn load(&self, key: &str) -> Result<String> {
        self.get_value(key).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ValueLoader;
    use mockito::mock;

    #[tokio::test]
    async fn test_aws_ec2_metadata_basic() {
        let expected = "test-id";

        let _m = mock("GET", "/instance-id")
            .with_status(200)
            .with_header("Content-Type", "text/plain")
            .with_body(expected)
            .create();

        let mut url = String::from(mockito::server_url());
        // We have to append a slash to the URL to mimic the default value
        url.push('/');
        let loader = AwsEc2MetadataLoader::with_base_url(&url);

        let actual = loader.load("instance-id").await;

        assert_eq!(expected, actual.unwrap());
    }
}
