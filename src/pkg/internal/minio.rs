use aws_sdk_s3::{config::Region, Client};
use standard_error::StandardError;

use crate::{conf::settings, prelude::Result};

#[derive(Debug)]
struct DefaultResolver {
 endpoint: String,
}

impl DefaultResolver {
   fn new(endpoint: &str) -> Self {
     DefaultResolver {
     endpoint: endpoint.to_string(),
     }
   }
}

pub async fn create_bucket(
    client: &aws_sdk_s3::Client,
    bucket_name: &str,
) -> Result<Option<aws_sdk_s3::operation::create_bucket::CreateBucketOutput>> {
    let constraint = aws_sdk_s3::types::BucketLocationConstraint::from(settings.s3_region.to_string().as_str());
    let cfg = aws_sdk_s3::types::CreateBucketConfiguration::builder()
        .location_constraint(constraint)
        .build();
    let create = client
        .create_bucket()
        .create_bucket_configuration(cfg)
        .bucket(bucket_name)
        .send()
        .await;
    create.map(Some).or_else(|err| {
        if err
            .as_service_error()
            .map(|se| se.is_bucket_already_exists() || se.is_bucket_already_owned_by_you())
            == Some(true)
        {
            Ok(None)
        } else {
            Err(StandardError::new("ERR-S3-001"))
        }
    })
}


