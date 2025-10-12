use std::sync::Arc;

use standard_error::{Interpolate, StandardError};

use crate::prelude::Result;


#[async_trait::async_trait]
pub trait S3Ops {
    async fn create_new_bucket(
        &self,
        bucket_name: &str,
    ) -> Result<Option<aws_sdk_s3::operation::create_bucket::CreateBucketOutput>>;
    
    async fn upload_object(
        &self,
        bucket_name: &str,
        s3_key: &str,
        file_data: Vec<u8>,
        content_type: &str
    ) -> Result<aws_sdk_s3::operation::put_object::PutObjectOutput>;
    
    async fn retrieve_object(&self, bucket_name: &str, key: &str) -> Result<(Vec<u8>, String)>;
}

#[async_trait::async_trait]
impl S3Ops for Arc<aws_sdk_s3::Client> {
    async fn create_new_bucket(
        &self,
        bucket_name: &str,
    ) -> Result<Option<aws_sdk_s3::operation::create_bucket::CreateBucketOutput>> {
        let create = self.create_bucket().bucket(bucket_name).send().await;
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

    async fn upload_object(
        &self,
        bucket_name: &str,
        s3_key: &str,
        file_data: Vec<u8>,
        content_type: &str
    ) -> Result<aws_sdk_s3::operation::put_object::PutObjectOutput> {
        // Create ByteStream from bytes with proper length
        let body = aws_sdk_s3::primitives::ByteStream::from(file_data);
        
        tracing::debug!("Uploading to S3: bucket={}, key={}, content_type={}", 
            bucket_name, s3_key, content_type);
        
        let result = self
            .put_object()
            .bucket(bucket_name)
            .key(s3_key)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("S3 upload failed: {:?}", e);
                StandardError::new("ERR-S3-002").interpolate_err(e.to_string())
            })?;
        
        tracing::debug!("S3 upload successful");
        Ok(result)
    }

    async fn retrieve_object(&self, bucket: &str, key: &str) -> Result<(Vec<u8>, String)> {
        let response = self
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StandardError::new("ERR-S3-003").interpolate_err(e.to_string()))?;
        let content_type = response.content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());
        let data = response.body.collect().await.map_err(|e| StandardError::new("ERR-S3-004").interpolate_err(e.to_string()))?.into_bytes();
        Ok((data.to_vec(), content_type))
    }
}

