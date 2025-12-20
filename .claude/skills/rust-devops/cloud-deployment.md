# Cloud Deployment

Deploying Rust applications to major cloud platforms with provider-specific best practices.

## Platform Comparison

| Feature | Google Cloud Run | AWS Lambda | Azure Container Apps |
|---------|------------------|------------|---------------------|
| **Model** | Container | Container/Zip | Container |
| **Cold Start** | 100-500 ms | 100-1000 ms | 100-500 ms |
| **Max Memory** | 32 GB | 10 GB | 16 GB |
| **Scaling** | 0 to N | 0 to N | 0 to N |
| **VPC** | Direct VPC Egress | VPC Connector | VNet Integration |

## Google Cloud Run

Fully managed serverless container platform.

### Deployment

```bash
# Build and push
gcloud builds submit --tag gcr.io/PROJECT/myapp

# Deploy
gcloud run deploy myapp \
  --image gcr.io/PROJECT/myapp \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated
```

### Best Practices

- Use **Direct VPC Egress** for internal services
- Configure health checks at `/health`
- Minimize image size for faster cold starts
- Set appropriate memory/CPU limits

### Dockerfile for Cloud Run

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/myapp /
ENV PORT=8080
EXPOSE 8080
CMD ["/myapp"]
```

## AWS Lambda

Serverless functions with container or zip deployment.

### Container Deployment

```dockerfile
FROM public.ecr.aws/lambda/provided:al2 as runtime

FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM runtime
COPY --from=builder /app/target/release/bootstrap ${LAMBDA_RUNTIME_DIR}
CMD ["bootstrap"]
```

```bash
# Push to ECR
aws ecr get-login-password | docker login --username AWS --password-stdin $ECR_URI
docker build -t myapp .
docker tag myapp:latest $ECR_URI/myapp:latest
docker push $ECR_URI/myapp:latest

# Create/update function
aws lambda create-function \
  --function-name myapp \
  --package-type Image \
  --code ImageUri=$ECR_URI/myapp:latest \
  --role arn:aws:iam::ACCOUNT:role/lambda-role
```

### Best Practices

- Use **provisioned concurrency** for latency-sensitive workloads
- Enable **Lambda Layers** for shared dependencies
- Use AWS-provided base images for compatibility

## Azure Container Apps

Serverless container platform with Kubernetes foundation.

### Deployment

```bash
# Create environment
az containerapp env create \
  --name myenv \
  --resource-group mygroup \
  --location eastus

# Deploy
az containerapp create \
  --name myapp \
  --resource-group mygroup \
  --environment myenv \
  --image myregistry.azurecr.io/myapp:latest \
  --target-port 8080 \
  --ingress external
```

### Best Practices

- Use Azure Container Registry (ACR) for images
- Configure **Dapr** for microservices patterns
- Use Alpine or Chiselled Ubuntu for minimal images

## Other Platforms

### Cloudflare Workers

Edge-deployed Rust via WebAssembly.

```bash
# Install wrangler
npm install -g wrangler

# Create worker
cargo generate --git https://github.com/cloudflare/workers-sdk

# Deploy
wrangler deploy
```

### Vercel

Deploy via WebAssembly or containers.

```bash
# vercel.json
{
  "functions": {
    "api/**/*.rs": {
      "runtime": "vercel-rust@latest"
    }
  }
}
```

### Heroku

Use community buildpack.

```bash
heroku buildpacks:set emk/heroku-buildpack-rust
git push heroku main
```

## Security Best Practices

### Secrets Management

| Provider | Service |
|----------|---------|
| AWS | Secrets Manager, Parameter Store |
| GCP | Secret Manager |
| Azure | Key Vault |

```rust
// Example: AWS Secrets Manager
use aws_sdk_secretsmanager::Client;

async fn get_secret(client: &Client, name: &str) -> String {
    client.get_secret_value()
        .secret_id(name)
        .send()
        .await
        .unwrap()
        .secret_string()
        .unwrap()
        .to_string()
}
```

### Image Scanning

- **AWS**: ECR image scanning
- **GCP**: Container Analysis
- **Azure**: Defender for Containers

### IAM Best Practices

- Apply least-privilege permissions
- Use service accounts/roles, not user credentials
- Rotate credentials regularly

## Cost Optimization

- Rust's low memory footprint reduces compute costs
- Fast cold starts mean lower provisioned concurrency needs
- Static binaries enable smaller images = faster pulls

## Related

- [Container Builds](./container-builds.md)
- [CI/CD Workflows](./cicd-workflows.md)
