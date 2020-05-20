use anyhow::{anyhow, Result};
use rusoto_core::Region;
use rusoto_ssm::{GetParameterRequest, Ssm, SsmClient};

pub(crate) const TEMPLATE_KEY: &str = "awsssm";

pub struct AwsSsmLoader {
    client: rusoto_ssm::SsmClient,
}

impl AwsSsmLoader {
    pub fn new() -> Self {
        // TODO hard coded region - should be configurable
        let client = SsmClient::new(Region::default());
        Self::with_client(client)
    }

    pub fn with_client(client: SsmClient) -> Self {
        Self { client }
    }

    async fn get_parameter(&self, name: &str, decrypt: bool) -> Result<String> {
        let req = GetParameterRequest {
            name: name.to_string(),
            with_decryption: Some(decrypt),
        };

        let response = match self.client.get_parameter(req).await {
            Ok(response) => response,
            Err(rusoto_core::RusotoError::Service(
                rusoto_ssm::GetParameterError::ParameterNotFound(_),
            )) => {
                return Err(anyhow!("Parameter not found '{}'", name)
                    .context("Failed to fetch parameter from AWS SSM"))
            }
            Err(e) => return Err(anyhow!("Failed to fetch parameter: {}", e)),
        };

        let parameter = response
            .parameter
            .ok_or_else(|| anyhow!("Failed to get parameter"))?;

        let value = parameter
            .value
            .ok_or_else(|| anyhow!("Parameter has no value"))?;

        Ok(value)
    }
}

#[async_trait::async_trait]
impl crate::ValueLoader for AwsSsmLoader {
    async fn load(&self, key: &str) -> Result<String> {
        // TODO hard coded decrypt value
        // Options:
        //   flag --awsssm-decrypt - will only work if all values are encrypted
        //   separate template strings: (ValueSource)
        //      %awsssm:my.value% - instantiate an AwsSsmLoader with decrypt set to false
        //      %awsssm_decrypt:my.value% - instantiate an AwsSsmLoader with decrypt true
        self.get_parameter(key, true).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ValueLoader;
    use rusoto_mock::{
        MockCredentialsProvider, MockRequestDispatcher, MockResponseReader, ReadMockResponse,
    };

    #[tokio::test]
    async fn test_ssm_load_parameter() {
        let mock_client = rusoto_ssm::SsmClient::new_with(
            MockRequestDispatcher::default().with_body(&MockResponseReader::read_response(
                "testdata/awsssm",
                "get-parameter-response.json",
            )),
            MockCredentialsProvider,
            Default::default(),
        );

        let loader = AwsSsmLoader::with_client(mock_client);
        let actual = loader.load("test.param").await.unwrap();

        assert_eq!(String::from("ssm value"), actual);
    }

    #[tokio::test]
    async fn test_ssm_load_parameter_not_found() {
        let mock_client = rusoto_ssm::SsmClient::new_with(
            MockRequestDispatcher::with_status(400).with_body(&MockResponseReader::read_response(
                "testdata/awsssm",
                "get-parameter-not-found-response.json",
            )),
            MockCredentialsProvider,
            Default::default(),
        );

        let loader = AwsSsmLoader::with_client(mock_client);
        let actual = loader.load("test.param").await;

        assert!(actual.is_err());

        match actual {
            Err(err) => assert!(format!("{:?}", err).contains("Parameter not found")),
            _ => assert!(false),
        }
    }
}
