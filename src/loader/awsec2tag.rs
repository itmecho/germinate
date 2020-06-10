//! Provides the ability to asynchronously load values from [AWS EC2 Tags](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html)
use anyhow::{anyhow, Result};
use rusoto_core::Region;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};

pub(crate) const TEMPLATE_KEY: &str = "awsec2tag";

/// This type provides functionality for loading values from [AWS EC2 Tags](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html)
pub struct AwsEc2TagLoader<'a> {
    client: rusoto_ec2::Ec2Client,
    metadata_url: &'a str,
}

impl<'a> AwsEc2TagLoader<'a> {
    /// Creates a new AwsEc2TagLoader with the default region
    pub fn new() -> Self {
        // This will attempt to read AWS_DEFAULT_REGION and AWS_REGION from the environment. If
        // neither are set, it will fallback to us-east-1
        let client = Ec2Client::new(Region::default());
        Self::with_client(client)
    }

    /// Creates a new AwsEc2TagLoader with the provided Ec2Client
    pub fn with_client(client: Ec2Client) -> Self {
        Self::with_client_and_metadata_url(client, crate::loader::awsec2metadata::METADATA_BASE_URL)
    }

    pub fn with_client_and_metadata_url(client: Ec2Client, metadata_url: &'a str) -> Self {
        Self {
            client,
            metadata_url,
        }
    }

    /// Loads a EC2 tag value by it's key and returns it as a `String`
    async fn get_tag_value(&self, key: &str) -> Result<String> {
        let instance_id =
            crate::loader::awsec2metadata::get_metadata_value(self.metadata_url, "instance-id")
                .await?;

        let mut req = DescribeInstancesRequest::default();
        req.instance_ids = Some(vec![instance_id]);

        let response = match self.client.describe_instances(req).await {
            Ok(response) => response,
            Err(e) => return Err(anyhow!("Failed to fetch tag value: {}", e)),
        };

        let value = response
            .reservations
            .ok_or_else(|| anyhow!("Reservations missing from response"))?
            .first()
            .ok_or_else(|| anyhow!("No Reservations found"))?
            .instances
            .as_ref()
            .ok_or_else(|| anyhow!("Instances missing from response"))?
            .first()
            .ok_or_else(|| anyhow!("No Instances found"))?
            .tags
            .as_ref()
            .ok_or_else(|| anyhow!("Tags missing from response"))?
            .iter()
            .filter(|t| t.key.as_ref().unwrap_or(&"".into()).to_lowercase() == key.to_lowercase())
            .collect::<Vec<&rusoto_ec2::Tag>>()
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
impl crate::Loader for AwsEc2TagLoader<'_> {
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
        let loader = AwsEc2TagLoader::with_client_and_metadata_url(mock_client, url);
        let actual = loader.load("TestTag").await.unwrap();

        m.assert();
        assert_eq!(String::from("test value"), actual);
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
        let loader = AwsEc2TagLoader::with_client_and_metadata_url(mock_client, url);
        let actual = loader.load("testtag").await.unwrap();

        m.assert();
        assert_eq!(String::from("test value"), actual);
    }
}
