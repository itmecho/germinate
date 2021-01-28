//! Provides the ability to asynchronously load values from [AWS EC2 Tags](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html)
use anyhow::{anyhow, Result};
use rusoto_core::Region;
use rusoto_ec2::{DescribeTagsRequest, Ec2, Ec2Client, Filter, TagDescription};

pub(crate) const TEMPLATE_KEY: &str = "awsec2tag";

/// This type provides functionality for loading values from [AWS EC2 Tags](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html)
pub struct AwsEc2TagLoader {
    tags: Vec<TagDescription>,
}

impl AwsEc2TagLoader {
    /// Creates a new AwsEc2TagLoader with the default region
    pub async fn new() -> Result<Self> {
        // This will attempt to read AWS_DEFAULT_REGION and AWS_REGION from the environment. If
        // neither are set, it will fallback to us-east-1
        let region: Region = crate::loader::awsec2metadata::get_current_region()
            .await?
            .parse()
            .unwrap_or_default();
        let client = Ec2Client::new(region);
        Self::with_client(client).await
    }

    /// Creates a new AwsEc2TagLoader with the provided Ec2Client
    pub async fn with_client(client: Ec2Client) -> Result<Self> {
        Self::with_client_and_metadata_url(client, crate::loader::awsec2metadata::METADATA_BASE_URL)
            .await
    }

    /// Creates a new AwsEc2TagLoader with the provided Ec2Client and metadata URL
    pub async fn with_client_and_metadata_url(
        client: Ec2Client,
        metadata_url: &str,
    ) -> Result<Self> {
        let instance_id =
            crate::loader::awsec2metadata::get_metadata_value(metadata_url, "instance-id").await?;

        let mut req = DescribeTagsRequest::default();
        req.filters = Some(Vec::from([Filter {
            name: Some("resource-id".to_string()),
            values: Some(Vec::from([instance_id])),
        }]));

        let response = match client.describe_tags(req).await {
            Ok(response) => response,
            Err(e) => return Err(anyhow!("Failed to fetch tag value: {}", e)),
        };

        let tags = response
            .tags
            .as_ref()
            .ok_or_else(|| anyhow!("Tags missing from response"))?
            .clone();

        Ok(Self { tags })
    }

    /// Loads an EC2 tag value by it's key and returns it as a `String`
    async fn get_tag_value(&self, key: &str) -> Result<String> {
        let value = self
            .tags
            .iter()
            .filter(|t| {
                t.key.as_ref().unwrap_or(&String::new()).to_lowercase() == key.to_lowercase()
            })
            .collect::<Vec<&rusoto_ec2::TagDescription>>()
            .first()
            .ok_or_else(|| anyhow!("Tag with key '{}' not found", key))?
            .value
            .as_ref()
            .ok_or_else(|| anyhow!("Tag has no value"))?
            .clone();

        Ok(value)
    }
}

#[async_trait::async_trait]
impl crate::Loader for AwsEc2TagLoader {
    /// Loads a value from the EC2 Instance's Tags and returns it as a `String`
    async fn load(&self, key: &str) -> Result<String> {
        self.get_tag_value(key).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Loader;
    use rusoto_mock::{
        MockCredentialsProvider, MockRequestDispatcher, MockResponseReader, ReadMockResponse,
    };

    fn tag_value() -> String {
        String::from("test value")
    }

    #[tokio::test]
    async fn test_aws_ec2_tag_load_basic() {
        let mock_client = rusoto_ec2::Ec2Client::new_with(
            MockRequestDispatcher::default().with_body(&MockResponseReader::read_response(
                "testdata/awsec2tag",
                // Taken from https://github.com/rusoto/rusoto/tree/master/rusoto/services/ec2/test_resources/generated
                "describe-instances-response.xml",
            )),
            MockCredentialsProvider,
            Default::default(),
        );

        let m = mockito::mock("GET", "/instance-id")
            .with_status(200)
            .with_body("i-01234567890123456")
            .expect(1)
            .create();

        let url = &mockito::server_url();
        let loader = AwsEc2TagLoader::with_client_and_metadata_url(mock_client, url)
            .await
            .unwrap();
        let actual = loader.load("TestTag").await.unwrap();

        m.assert();
        assert_eq!(tag_value(), actual);
    }

    #[tokio::test]
    async fn test_aws_ec2_tag_load_is_case_insensitive() {
        let mock_client = rusoto_ec2::Ec2Client::new_with(
            MockRequestDispatcher::default().with_body(&MockResponseReader::read_response(
                "testdata/awsec2tag",
                // Taken from https://github.com/rusoto/rusoto/tree/master/rusoto/services/ec2/test_resources/generated
                "describe-instances-response.xml",
            )),
            MockCredentialsProvider,
            Default::default(),
        );

        let m = mockito::mock("GET", "/instance-id")
            .with_status(200)
            .with_body("i-01234567890123456")
            .expect(1)
            .create();

        let url = &mockito::server_url();
        let loader = AwsEc2TagLoader::with_client_and_metadata_url(mock_client, url)
            .await
            .unwrap();
        let actual = loader.load("testtag").await.unwrap();

        m.assert();
        assert_eq!(tag_value(), actual);
    }

    #[tokio::test]
    async fn test_aws_ec2_tag_load_caches_tags() {
        let mock_client = rusoto_ec2::Ec2Client::new_with(
            MockRequestDispatcher::default().with_body(&MockResponseReader::read_response(
                "testdata/awsec2tag",
                // Taken from https://github.com/rusoto/rusoto/tree/master/rusoto/services/ec2/test_resources/generated
                "describe-instances-response.xml",
            )),
            MockCredentialsProvider,
            Default::default(),
        );

        // By creating a mock server that asserts only one request was made, we can check that
        // after the first load, the cached tags are returned
        let m = mockito::mock("GET", "/instance-id")
            .with_status(200)
            .with_body("i-01234567890123456")
            .expect(1)
            .create();

        let url = &mockito::server_url();
        let loader = AwsEc2TagLoader::with_client_and_metadata_url(mock_client, url)
            .await
            .unwrap();
        assert_eq!(tag_value(), loader.load("TestTag").await.unwrap());
        assert_eq!(tag_value(), loader.load("TestTag").await.unwrap());
        assert_eq!(tag_value(), loader.load("TestTag").await.unwrap());
        assert_eq!(
            String::from("my-instance"),
            loader.load("Name").await.unwrap()
        );

        m.assert();
    }
}
