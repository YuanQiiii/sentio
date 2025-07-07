# Sentio 部署指南

## 部署选项

### 1. 本地部署

#### 系统要求
- CPU: 双核心及以上
- 内存: 2GB 及以上
- 磁盘: 1GB 可用空间
- 网络: 互联网连接（访问 LLM API）

#### 部署步骤

1. **准备环境**
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

2. **获取代码**
```bash
git clone <repository-url>
cd sentio
```

3. **配置环境**
```bash
# 复制环境变量模板
cp .env.example .env

# 编辑配置文件
nano .env
```

4. **构建项目**
```bash
cargo build --release
```

5. **运行服务**
```bash
# 前台运行
./target/release/sentio_core

# 后台运行
nohup ./target/release/sentio_core > logs/app.log 2>&1 &
```

### 2. Docker 部署

#### 创建 Dockerfile

```dockerfile
# Multi-stage build for smaller image
FROM rust:1.70-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false sentio

# Create directories
RUN mkdir -p /app/config /app/logs /app/data && \
    chown -R sentio:sentio /app

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/sentio_core ./
COPY --from=builder /app/config ./config/

# Set permissions
RUN chmod +x ./sentio_core

# Switch to non-root user
USER sentio

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["./sentio_core"]
```

#### 构建 Docker 镜像

```bash
# 构建镜像
docker build -t sentio:latest .

# 运行容器
docker run -d \
    --name sentio \
    -p 8080:8080 \
    -v $(pwd)/config:/app/config \
    -v $(pwd)/logs:/app/logs \
    -v $(pwd)/data:/app/data \
    -e DEEPSEEK_API_KEY=your_api_key \
    -e SMTP_HOST=smtp.gmail.com \
    -e SMTP_PORT=587 \
    -e SMTP_USERNAME=your_email@gmail.com \
    -e SMTP_PASSWORD=your_password \
    sentio:latest
```

#### Docker Compose 部署

创建 `docker-compose.yml`：

```yaml
version: '3.8'

services:
  sentio:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./config:/app/config
      - ./logs:/app/logs
      - ./data:/app/data
    environment:
      - RUST_LOG=info
      - DEEPSEEK_API_KEY=${DEEPSEEK_API_KEY}
      - SMTP_HOST=${SMTP_HOST}
      - SMTP_PORT=${SMTP_PORT}
      - SMTP_USERNAME=${SMTP_USERNAME}
      - SMTP_PASSWORD=${SMTP_PASSWORD}
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  # 可选：添加 Redis 作为缓存
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    restart: unless-stopped

  # 可选：添加 PostgreSQL 作为数据库
  postgres:
    image: postgres:15-alpine
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_DB=sentio
      - POSTGRES_USER=sentio
      - POSTGRES_PASSWORD=sentio_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  redis_data:
  postgres_data:
```

部署命令：
```bash
# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f sentio

# 停止服务
docker-compose down
```

### 3. 系统服务部署

#### 创建系统用户

```bash
# 创建专用用户
sudo useradd -r -s /bin/false sentio
sudo mkdir -p /opt/sentio
sudo chown sentio:sentio /opt/sentio
```

#### 创建 systemd 服务

创建 `/etc/systemd/system/sentio.service`：

```ini
[Unit]
Description=Sentio Email Processing Service
After=network.target network-online.target
Requires=network-online.target

[Service]
Type=simple
User=sentio
Group=sentio
WorkingDirectory=/opt/sentio
ExecStart=/opt/sentio/sentio_core
ExecReload=/bin/kill -HUP $MAINPID

# 环境变量
Environment=RUST_LOG=info
Environment=DEEPSEEK_API_KEY=your_api_key
Environment=SMTP_HOST=smtp.gmail.com
Environment=SMTP_PORT=587
Environment=SMTP_USERNAME=your_email@gmail.com
Environment=SMTP_PASSWORD=your_password

# 重启策略
Restart=always
RestartSec=10
StartLimitInterval=60
StartLimitBurst=3

# 安全设置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/sentio/logs /opt/sentio/data

# 资源限制
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

#### 启用和启动服务

```bash
# 重新加载 systemd 配置
sudo systemctl daemon-reload

# 启用服务（开机自启）
sudo systemctl enable sentio

# 启动服务
sudo systemctl start sentio

# 查看服务状态
sudo systemctl status sentio

# 查看日志
sudo journalctl -u sentio -f
```

### 4. 云平台部署

#### AWS 部署

1. **使用 ECS**

创建 ECS 任务定义：
```json
{
  "family": "sentio",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "256",
  "memory": "512",
  "executionRoleArn": "arn:aws:iam::account:role/ecsTaskExecutionRole",
  "containerDefinitions": [
    {
      "name": "sentio",
      "image": "your-account.dkr.ecr.region.amazonaws.com/sentio:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "DEEPSEEK_API_KEY",
          "value": "your_api_key"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/sentio",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

2. **使用 EC2**

用户数据脚本：
```bash
#!/bin/bash
yum update -y
yum install -y docker
service docker start
usermod -a -G docker ec2-user

# 拉取并运行容器
docker run -d \
    --name sentio \
    -p 8080:8080 \
    -e DEEPSEEK_API_KEY=your_api_key \
    your-account.dkr.ecr.region.amazonaws.com/sentio:latest
```

#### Google Cloud 部署

1. **使用 Cloud Run**

```bash
# 构建并推送镜像
gcloud builds submit --tag gcr.io/PROJECT_ID/sentio

# 部署到 Cloud Run
gcloud run deploy sentio \
    --image gcr.io/PROJECT_ID/sentio \
    --platform managed \
    --region us-central1 \
    --allow-unauthenticated \
    --set-env-vars DEEPSEEK_API_KEY=your_api_key
```

2. **使用 GKE**

Kubernetes 部署文件：
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sentio
spec:
  replicas: 2
  selector:
    matchLabels:
      app: sentio
  template:
    metadata:
      labels:
        app: sentio
    spec:
      containers:
      - name: sentio
        image: gcr.io/PROJECT_ID/sentio:latest
        ports:
        - containerPort: 8080
        env:
        - name: DEEPSEEK_API_KEY
          valueFrom:
            secretKeyRef:
              name: sentio-secrets
              key: deepseek-api-key
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
---
apiVersion: v1
kind: Service
metadata:
  name: sentio-service
spec:
  selector:
    app: sentio
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

## 配置管理

### 环境变量

创建 `.env` 文件：
```env
# 应用配置
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
WORKERS=4

# LLM 配置
DEEPSEEK_API_KEY=your_api_key_here
LLM_MODEL=deepseek-chat
LLM_TIMEOUT=120
LLM_MAX_RETRIES=3

# 邮件配置
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_app_password
SMTP_USE_TLS=true

# 数据存储
MEMORY_FILE_PATH=/app/data/memory.json
LOG_DIR=/app/logs
```

### 配置文件

更新 `config/default.toml`：
```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[llm]
provider = "deepseek"
api_key = "${DEEPSEEK_API_KEY}"
base_url = "https://api.deepseek.com"
model = "${LLM_MODEL:-deepseek-chat}"
timeout = 120
max_retries = 3

[email.smtp]
host = "${SMTP_HOST}"
port = "${SMTP_PORT}"
username = "${SMTP_USERNAME}"
password = "${SMTP_PASSWORD}"
use_tls = true

[telemetry]
level = "${RUST_LOG:-info}"
```

## 监控和日志

### 日志配置

1. **结构化日志**

```rust
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sentio=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

2. **日志轮转**

使用 `logrotate` 配置：
```
/opt/sentio/logs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 sentio sentio
    postrotate
        systemctl reload sentio
    endscript
}
```

### 监控指标

1. **Prometheus 集成**

在 `Cargo.toml` 中添加：
```toml
[dependencies]
prometheus = "0.13"
tokio-metrics = "0.1"
```

2. **健康检查端点**

```rust
use axum::{response::Json, routing::get, Router};
use serde_json::json;

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

let app = Router::new()
    .route("/health", get(health_check));
```

### 性能监控

1. **系统指标**

```bash
# CPU 使用率
top -p $(pgrep sentio_core)

# 内存使用
ps aux | grep sentio_core

# 网络连接
netstat -tulpn | grep :8080

# 磁盘使用
df -h /opt/sentio
```

2. **应用指标**

```rust
use prometheus::{Counter, Histogram, IntGauge};

lazy_static! {
    static ref EMAIL_PROCESSED_TOTAL: Counter = Counter::new(
        "sentio_emails_processed_total",
        "Total number of emails processed"
    ).unwrap();
    
    static ref EMAIL_PROCESSING_DURATION: Histogram = Histogram::new(
        "sentio_email_processing_duration_seconds",
        "Email processing duration in seconds"
    ).unwrap();
    
    static ref ACTIVE_CONNECTIONS: IntGauge = IntGauge::new(
        "sentio_active_connections",
        "Number of active connections"
    ).unwrap();
}
```

## 安全配置

### 1. 网络安全

```bash
# 防火墙配置
sudo ufw allow 8080/tcp
sudo ufw enable

# 反向代理（Nginx）
server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### 2. SSL/TLS 配置

```bash
# 使用 Let's Encrypt
sudo certbot --nginx -d your-domain.com

# 或手动配置
server {
    listen 443 ssl;
    server_name your-domain.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://localhost:8080;
        # ... 其他配置
    }
}
```

### 3. 密钥管理

```bash
# 使用 HashiCorp Vault
vault kv put secret/sentio \
    deepseek_api_key=your_api_key \
    smtp_password=your_smtp_password

# 或使用 AWS Secrets Manager
aws secretsmanager create-secret \
    --name sentio/credentials \
    --secret-string '{"deepseek_api_key":"your_api_key"}'
```

## 备份和恢复

### 1. 数据备份

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backups/sentio"
DATE=$(date +%Y%m%d_%H%M%S)

# 创建备份目录
mkdir -p "$BACKUP_DIR/$DATE"

# 备份配置文件
cp -r /opt/sentio/config "$BACKUP_DIR/$DATE/"

# 备份数据文件
cp -r /opt/sentio/data "$BACKUP_DIR/$DATE/"

# 备份日志（最近7天）
find /opt/sentio/logs -name "*.log" -mtime -7 -exec cp {} "$BACKUP_DIR/$DATE/" \;

# 压缩备份
tar -czf "$BACKUP_DIR/sentio_backup_$DATE.tar.gz" -C "$BACKUP_DIR" "$DATE"
rm -rf "$BACKUP_DIR/$DATE"

echo "Backup completed: $BACKUP_DIR/sentio_backup_$DATE.tar.gz"
```

### 2. 自动化备份

```bash
# 添加到 crontab
0 2 * * * /opt/sentio/scripts/backup.sh
```

## 故障排除

### 常见问题

1. **服务启动失败**
```bash
# 检查日志
sudo journalctl -u sentio -n 50

# 检查配置
sudo -u sentio /opt/sentio/sentio_core --check-config
```

2. **性能问题**
```bash
# 检查资源使用
htop
iotop
free -h

# 分析日志
grep -i "error\|warning" /opt/sentio/logs/*.log
```

3. **网络问题**
```bash
# 检查端口
netstat -tlnp | grep :8080

# 测试连接
curl -I http://localhost:8080/health
```

### 性能优化

1. **调整 Rust 编译选项**

```toml
# Cargo.toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
```

2. **系统优化**

```bash
# 调整文件句柄限制
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf

# 调整内核参数
echo "net.core.somaxconn = 65536" >> /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 65536" >> /etc/sysctl.conf
sysctl -p
```

---

*部署指南版本：v1.0.0*
*最后更新时间：2025-07-07*