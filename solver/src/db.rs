use crate::isl;

use aws_sdk_dynamodb as dynamodb;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_s3 as s3;
use aws_sdk_s3::types::ByteStream;

pub async fn save(
    run_id: &str,
    problem_id: &str,
    program: &isl::Program,
    score: i64,
    image_path: &str,
) -> anyhow::Result<()> {
    let table_name = "InfraStack-TableCD117FA1-1NAQ40LMS0E1G";
    let bucket_name = "infrastack-bucket83908e77-vvxulc74xyib";

    let config = aws_config::load_from_env().await;

    let client = dynamodb::Client::new(&config);

    let pk = format!("R#{}", run_id).to_string();
    let sk = format!("S#{}", problem_id).to_string();
    let gsi1pk = format!("P#{}", problem_id).to_string();
    let gsi1sk = score.to_string();
    client
        .put_item()
        .table_name(table_name)
        .item("PK", AttributeValue::S(pk))
        .item("SK", AttributeValue::S(sk))
        .item("GSI1PK", AttributeValue::S(gsi1pk))
        .item("GSI1SK", AttributeValue::N(gsi1sk))
        .send()
        .await?;

    let client = s3::Client::new(&config);
    client
        .put_object()
        .bucket(bucket_name)
        .key(format!("{}/{}.isl", run_id, problem_id).to_string())
        .body(ByteStream::from(format!("{program}").into_bytes()))
        .send()
        .await?;

    let client = s3::Client::new(&config);
    client
        .put_object()
        .bucket(bucket_name)
        .key(format!("{}/{}.png", run_id, problem_id).to_string())
        .body(ByteStream::from_path(image_path).await.unwrap())
        .send()
        .await?;

    Ok(())
}
