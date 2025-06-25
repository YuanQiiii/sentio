#!/bin/bash

# Sentio AI MongoDB 基础设施部署脚本
# 这个脚本用于启动和管理 MongoDB 容器

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 函数定义
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    if ! command -v docker &> /dev/null; then
        log_error "Docker 未安装，请先安装 Docker"
        exit 1
    fi
    
    # 检查并设置 Docker Compose 命令
    if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
        DOCKER_COMPOSE="docker compose"
    elif command -v docker compose >/dev/null 2>&1; then
        DOCKER_COMPOSE="docker compose"
    else
        log_error "Docker Compose 未安装，请先安装 Docker Compose"
        exit 1
    fi
    
    log_success "依赖检查通过"
}

# 检查环境变量文件
check_env_file() {
    if [ ! -f ".env" ]; then
        log_warning ".env 文件不存在，将从 .env.example 复制"
        cp .env.example .env
        log_info "请编辑 .env 文件，设置正确的配置值"
    fi
}

# 启动 MongoDB
start_mongodb() {
    log_info "启动 MongoDB 基础设施..."
    
    # 检查是否有正在运行的容器
    if docker ps -q -f name=sentio-mongodb > /dev/null 2>&1; then
        log_warning "MongoDB 容器已在运行"
        return 0
    fi
    
    # 启动 MongoDB
    $DOCKER_COMPOSE up -d mongodb
    
    # 等待 MongoDB 启动
    log_info "等待 MongoDB 启动完成..."
    for i in {1..30}; do
        if $DOCKER_COMPOSE exec mongodb mongosh --eval "db.runCommand('ping')" > /dev/null 2>&1; then
            log_success "MongoDB 启动成功"
            return 0
        fi
        echo -n "."
        sleep 2
    done
    
    log_error "MongoDB 启动超时"
    return 1
}

# 启动 MongoDB Express (可选)
start_mongo_express() {
    log_info "启动 MongoDB Express 管理界面..."
    $DOCKER_COMPOSE --profile tools up -d mongo-express
    log_success "MongoDB Express 已启动，访问地址: http://localhost:8081"
}

# 停止服务
stop_services() {
    log_info "停止 MongoDB 基础设施..."
    $DOCKER_COMPOSE down
    log_success "服务已停止"
}

# 查看状态
show_status() {
    log_info "服务状态:"
    $DOCKER_COMPOSE ps
    
    echo
    log_info "连接信息:"
    echo "MongoDB: mongodb://admin:sentio-secure-password@localhost:27017/sentio?authSource=admin"
    echo "MongoDB Express: http://localhost:8081 (如果已启用)"
}

# 查看日志
show_logs() {
    log_info "显示 MongoDB 日志..."
    $DOCKER_COMPOSE logs -f mongodb
}

# 清理数据
clean_data() {
    log_warning "这将删除所有 MongoDB 数据，是否继续？ (y/N)"
    read -r response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        log_info "停止服务并清理数据..."
        $DOCKER_COMPOSE down -v
        docker volume rm sentio_mongodb_data sentio_mongodb_config 2>/dev/null || true
        log_success "数据清理完成"
    else
        log_info "已取消清理操作"
    fi
}

# 备份数据
backup_data() {
    if [ -z "$1" ]; then
        log_error "请指定备份文件名"
        echo "使用方法: $0 backup <backup-name>"
        exit 1
    fi
    
    local backup_name="$1"
    local backup_dir="backups"
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local backup_file="${backup_dir}/${backup_name}_${timestamp}.gz"
    
    log_info "创建数据库备份..."
    
    # 创建备份目录
    mkdir -p "$backup_dir"
    
    # 执行备份
    $DOCKER_COMPOSE exec mongodb mongodump --authenticationDatabase admin \
        --username admin --password sentio-secure-password \
        --db sentio --gzip --archive | gzip > "$backup_file"
    
    log_success "备份完成: $backup_file"
}

# 恢复数据
restore_data() {
    if [ -z "$1" ]; then
        log_error "请指定备份文件路径"
        echo "使用方法: $0 restore <backup-file>"
        exit 1
    fi
    
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        log_error "备份文件不存在: $backup_file"
        exit 1
    fi
    
    log_warning "这将覆盖现有数据库，是否继续？ (y/N)"
    read -r response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        log_info "恢复数据库..."
        
        gunzip -c "$backup_file" | docker compose exec -T mongodb \
            mongorestore --authenticationDatabase admin \
            --username admin --password sentio-secure-password \
            --db sentio --gzip --archive
        
        log_success "数据恢复完成"
    else
        log_info "已取消恢复操作"
    fi
}

# 帮助信息
show_help() {
    echo "Sentio AI MongoDB 基础设施管理脚本"
    echo
    echo "使用方法:"
    echo "  $0 <command> [options]"
    echo
    echo "命令:"
    echo "  start              启动 MongoDB"
    echo "  start-with-ui      启动 MongoDB 和 MongoDB Express"
    echo "  stop               停止所有服务"
    echo "  status             显示服务状态"
    echo "  logs               显示 MongoDB 日志"
    echo "  clean              清理所有数据 (危险操作)"
    echo "  backup <name>      备份数据库"
    echo "  restore <file>     从备份恢复数据库"
    echo "  help               显示此帮助信息"
    echo
    echo "示例:"
    echo "  $0 start                    # 启动 MongoDB"
    echo "  $0 start-with-ui            # 启动 MongoDB 和管理界面"
    echo "  $0 backup daily             # 创建名为 daily 的备份"
    echo "  $0 restore backups/daily_20240625_120000.gz  # 恢复备份"
}

# 主函数
main() {
    case "${1:-help}" in
        "start")
            check_dependencies
            check_env_file
            start_mongodb
            show_status
            ;;
        "start-with-ui")
            check_dependencies
            check_env_file
            start_mongodb
            start_mongo_express
            show_status
            ;;
        "stop")
            stop_services
            ;;
        "status")
            show_status
            ;;
        "logs")
            show_logs
            ;;
        "clean")
            clean_data
            ;;
        "backup")
            backup_data "$2"
            ;;
        "restore")
            restore_data "$2"
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# 执行主函数
main "$@"
