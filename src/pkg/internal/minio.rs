use aws_sdk_s3::{config::Region, Client};
use standard_error::{Interpolate, StandardError};

use crate::prelude::Result;

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
    let create = client
        .create_bucket()
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
            Err(StandardError::new("ERR-S3-001").interpolate_err(err.to_string()))
        }
    })
}

pub async fn upload_object(
    client: &aws_sdk_s3::Client,
    bucket_name: &str,
    file_name: &str,
    key: &str,
) -> Result<aws_sdk_s3::operation::put_object::PutObjectOutput> {
    let body = aws_sdk_s3::primitives::ByteStream::from_path(std::path::Path::new(file_name)).await;
    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body.unwrap())
        .send()
        .await
        .map_err(|e| StandardError::new("ERR-S3-002").interpolate_err(e.to_string()))
}


