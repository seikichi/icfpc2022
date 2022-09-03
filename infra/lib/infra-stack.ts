import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as dynamodb from "aws-cdk-lib/aws-dynamodb";
import * as s3 from "aws-cdk-lib/aws-s3";
import * as cloudfront from "aws-cdk-lib/aws-cloudfront";
import * as iam from "aws-cdk-lib/aws-iam";

import * as child_process from "child_process";

export class InfraStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // get commit hash
    const commitHash = child_process
      .execSync("git rev-parse --short HEAD")
      .toString()
      .trim();

    // DB
    const STRING = dynamodb.AttributeType.STRING;
    const NUMBER = dynamodb.AttributeType.NUMBER;

    const table = new dynamodb.Table(this, "Table", {
      partitionKey: { name: "PK", type: STRING },
      sortKey: { name: "SK", type: STRING },
    });

    table.addGlobalSecondaryIndex({
      indexName: "GSI1",
      partitionKey: { name: "GSI1PK", type: STRING },
      sortKey: { name: "GSI1SK", type: NUMBER },
    });

    // Lambda
    const solver = new lambda.DockerImageFunction(this, "Solver", {
      code: lambda.DockerImageCode.fromImageAsset("../solver"),
      timeout: cdk.Duration.minutes(15),
      memorySize: 1024,
      environment: { COMMIT: commitHash },
    });
    table.grantReadWriteData(solver);

    // File
    const bucket = new s3.Bucket(this, "Bucket", {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
    });

    const oai = new cloudfront.OriginAccessIdentity(this, "OAI");
    bucket.addToResourcePolicy(
      new iam.PolicyStatement({
        effect: iam.Effect.ALLOW,
        actions: ["s3:GetObject"],
        resources: [`${bucket.bucketArn}/*`],
        principals: [
          new iam.CanonicalUserPrincipal(
            oai.cloudFrontOriginAccessIdentityS3CanonicalUserId
          ),
        ],
      })
    );

    new cloudfront.CloudFrontWebDistribution(this, "Distribution", {
      viewerCertificate: {
        aliases: [],
        props: {
          cloudFrontDefaultCertificate: true,
        },
      },
      priceClass: cloudfront.PriceClass.PRICE_CLASS_200,
      originConfigs: [
        {
          s3OriginSource: {
            s3BucketSource: bucket,
            originAccessIdentity: oai,
          },
          behaviors: [
            {
              isDefaultBehavior: true,
              minTtl: cdk.Duration.seconds(0),
              maxTtl: cdk.Duration.days(365),
              defaultTtl: cdk.Duration.days(1),
            },
          ],
        },
      ],
    });
  }
}
