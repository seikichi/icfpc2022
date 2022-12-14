use core::isl;

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
    ai: &str,
    commit: &str,
    elapsed: u64,
    now: u64,
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
        .item("PK", AttributeValue::S(pk.clone()))
        .item("SK", AttributeValue::S(sk))
        .item("GSI1PK", AttributeValue::S(gsi1pk))
        .item("GSI1SK", AttributeValue::N(gsi1sk))
        .item("AI", AttributeValue::S(ai.to_string()))
        .item("Commit", AttributeValue::S(commit.to_string()))
        .item("ExecTime", AttributeValue::N(elapsed.to_string()))
        .item("ExecDate", AttributeValue::N(now.to_string()))
        .send()
        .await?;

    // 親のレコードにもスコアを追加しておく
    client
        .update_item()
        .table_name(table_name)
        .key("PK", AttributeValue::S(pk.clone()))
        .key("SK", AttributeValue::S(pk))
        .update_expression("SET #key = :score")
        .expression_attribute_names("#key", format!("S#{}", problem_id))
        .expression_attribute_values(":score", AttributeValue::N(score.to_string()))
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
