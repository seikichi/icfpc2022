FROM public.ecr.aws/lambda/nodejs:16

RUN yum update -y && yum groupinstall -y 'Development Tools'

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH $PATH:/root/.cargo/bin
RUN rustup install stable

WORKDIR /code
COPY Cargo.lock Cargo.toml /code/
COPY core /code/core
COPY cli /code/cli
COPY lambda /code/lambda
RUN cargo build --release

COPY problems /code/problems
COPY lambda.js ${LAMBDA_TASK_ROOT}
RUN npm install @aws-sdk/client-dynamodb

CMD ["lambda.handler"]
