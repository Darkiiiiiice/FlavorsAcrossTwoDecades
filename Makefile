# Makefile for Flavors Across Two Decades
# 味延廿载

# 变量定义
CARGO := cargo
CONFIG := crates/backend/config/default.toml
DATABASE := sqlite:temp/flavors_game.db
BACKEND_BIN := target/debug/flavors-backend
RELEASE_BIN := target/release/flavors-backend

# 颜色输出
BLUE := \033[34m
GREEN := \033[32m
YELLOW := \033[33m
RED := \033[31m
RESET := \033[0m

.PHONY: all build run release dev clean test lint fmt check help init status swagger openapi

# 默认目标
all: build

# 初始化项目（首次运行）
init: build
	@mkdir -p temp
	@printf "$(GREEN)✓ 项目初始化完成$(RESET)\n"
	@printf "$(BLUE)运行 'make dev' 启动开发服务器$(RESET)\n"

# 开发构建
build:
	@printf "$(BLUE)构建项目...$(RESET)\n"
	$(CARGO) build

# 发布构建
release:
	@printf "$(BLUE)构建发布版本...$(RESET)\n"
	$(CARGO) build --release

# 运行开发服务器
run: build
	@mkdir -p temp
	@printf "$(GREEN)启动后端服务...$(RESET)\n"
	./$(BACKEND_BIN) -c $(CONFIG)

# 开发模式（带日志）
dev: build
	@mkdir -p temp
	@printf "$(GREEN)启动开发服务器 (DEBUG 模式)...$(RESET)\n"
	RUST_LOG=debug ./$(BACKEND_BIN) -c $(CONFIG)

# 使用自定义数据库运行
run-db: build
	@printf "$(GREEN)启动后端服务 (自定义数据库)...$(RESET)\n"
	./$(BACKEND_BIN) -c $(CONFIG) -d "$(DATABASE)"

# 内存数据库运行（测试用）
run-memory: build
	@printf "$(GREEN)启动后端服务 (内存数据库)...$(RESET)\n"
	./$(BACKEND_BIN) -c $(CONFIG) -d "sqlite::memory:"

# 运行测试
test:
	@printf "$(BLUE)运行测试...$(RESET)\n"
	$(CARGO) test

# 运行特定测试
test-%:
	@printf "$(BLUE)运行测试: $*...$(RESET)\n"
	$(CARGO) test $*

# 代码检查 (clippy)
lint:
	@printf "$(BLUE)运行 clippy...$(RESET)\n"
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# 代码格式化
fmt:
	@printf "$(BLUE)格式化代码...$(RESET)\n"
	$(CARGO) fmt

# 检查格式
fmt-check:
	@printf "$(BLUE)检查代码格式...$(RESET)\n"
	$(CARGO) fmt --check

# 完整检查（格式 + clippy + 测试）
check: fmt-check lint test
	@printf "$(GREEN)✓ 所有检查通过$(RESET)\n"

# 清理构建产物
clean:
	@printf "$(YELLOW)清理构建产物...$(RESET)\n"
	$(CARGO) clean
	@rm -rf temp/
	@printf "$(GREEN)✓ 清理完成$(RESET)\n"

# 清理数据库
clean-db:
	@printf "$(YELLOW)清理数据库...$(RESET)\n"
	@rm -rf temp/*.db
	@printf "$(GREEN)✓ 数据库已清理$(RESET)\n"

# 安装依赖（用于 CI）
ci-setup:
	@printf "$(BLUE)安装依赖...$(RESET)\n"
	$(CARGO) fetch

# CI 完整流程
ci: ci-setup fmt-check lint test build
	@printf "$(GREEN)✓ CI 检查完成$(RESET)\n"

# 构建文档
doc:
	@printf "$(BLUE)构建文档...$(RESET)\n"
	$(CARGO) doc --no-deps --open

# 监控模式（需要 cargo-watch）
watch:
	@printf "$(BLUE)启动监控模式...$(RESET)\n"
	@which cargo-watch > /dev/null || $(CARGO) install cargo-watch
	cargo watch -x "build" -x "test"

# 数据库迁移（手动执行）
migrate: build
	@printf "$(BLUE)运行数据库迁移...$(RESET)\n"
	./$(BACKEND_BIN) -c $(CONFIG) -d "$(DATABASE)"

# 生成 OpenAPI 文档
openapi: build
	@printf "$(BLUE)生成 OpenAPI 文档...$(RESET)\n"
	@printf "$(GREEN)启动服务器后访问:$(RESET)\n"
	@printf "  Swagger UI:     http://localhost:3000/swagger-ui/\n"
	@printf "  OpenAPI JSON:   http://localhost:3000/api-docs/openapi.json\n"
	@printf "\n"
	@printf "$(YELLOW)提示: 运行 'make run' 启动服务器后即可访问上述地址$(RESET)\n"

# 打开 Swagger UI（需要服务器运行）
swagger:
	@printf "$(BLUE)打开 Swagger UI...$(RESET)\n"
	@curl -s http://localhost:3000/swagger-ui/ > /dev/null 2>&1 && \
		(xdg-open http://localhost:3000/swagger-ui/ 2>/dev/null || open http://localhost:3000/swagger-ui/ 2>/dev/null || printf "$(YELLOW)请在浏览器中打开: http://localhost:3000/swagger-ui/$(RESET)\n") || \
		printf "$(RED)✗ 服务器未运行，请先执行 'make run'$(RESET)\n"

# 导出 OpenAPI JSON 文件
export-openapi: build
	@printf "$(BLUE)导出 OpenAPI JSON 文件...$(RESET)\n"
	@mkdir -p docs/api
	@./$(BACKEND_BIN) -c $(CONFIG) -d "sqlite::memory:" &
	@PID=$$!; sleep 3; \
	curl -s http://localhost:3000/api-docs/openapi.json > docs/api/openapi.json; \
	kill $$PID 2>/dev/null
	@test -s docs/api/openapi.json && printf "$(GREEN)✓ 已导出到 docs/api/openapi.json$(RESET)\n" || printf "$(RED)✗ 导出失败$(RESET)\n"

# 显示项目状态
status:
	@printf "$(BLUE)=== 项目状态 ===$(RESET)\n"
	@printf "\n"
	@printf "后端二进制: $(BACKEND_BIN)\n"
	@test -f $(BACKEND_BIN) && printf "  $(GREEN)✓ 已构建$(RESET)\n" || printf "  $(RED)✗ 未构建$(RESET)\n"
	@printf "\n"
	@printf "数据库文件: temp/flavors_game.db\n"
	@test -f temp/flavors_game.db && printf "  $(GREEN)✓ 存在$(RESET)\n" || printf "  $(YELLOW)○ 不存在 (首次运行时自动创建)$(RESET)\n"
	@printf "\n"
	@printf "Ollama 服务:\n"
	@curl -s http://localhost:11434/api/tags > /dev/null 2>&1 && printf "  $(GREEN)✓ 运行中$(RESET)\n" || printf "  $(RED)✗ 未运行$(RESET)\n"

# 帮助
help:
	@printf "\n"
	@printf "$(BLUE)味延廿载 - Makefile 帮助$(RESET)\n"
	@printf "\n"
	@printf "$(GREEN)构建命令:$(RESET)\n"
	@printf "  make build        - 构建开发版本\n"
	@printf "  make release      - 构建发布版本\n"
	@printf "  make clean        - 清理构建产物\n"
	@printf "\n"
	@printf "$(GREEN)运行命令:$(RESET)\n"
	@printf "  make run          - 运行后端服务\n"
	@printf "  make dev          - 运行开发服务器 (DEBUG 日志)\n"
	@printf "  make run-db       - 使用自定义数据库运行\n"
	@printf "  make run-memory   - 使用内存数据库运行\n"
	@printf "\n"
	@printf "$(GREEN)测试命令:$(RESET)\n"
	@printf "  make test         - 运行所有测试\n"
	@printf "  make test-<name>  - 运行特定测试\n"
	@printf "\n"
	@printf "$(GREEN)代码质量:$(RESET)\n"
	@printf "  make lint         - 运行 clippy 检查\n"
	@printf "  make fmt          - 格式化代码\n"
	@printf "  make fmt-check    - 检查代码格式\n"
	@printf "  make check        - 完整检查 (格式 + clippy + 测试)\n"
	@printf "\n"
	@printf "$(GREEN)其他命令:$(RESET)\n"
	@printf "  make init         - 初始化项目\n"
	@printf "  make status       - 显示项目状态\n"
	@printf "  make doc          - 构建并打开文档\n"
	@printf "  make watch        - 监控模式 (自动构建)\n"
	@printf "  make ci           - CI 完整流程\n"
	@printf "\n"
	@printf "$(GREEN)API 文档:$(RESET)\n"
	@printf "  make swagger      - 打开 Swagger UI (需先运行服务)\n"
	@printf "  make openapi      - 显示 OpenAPI 访问地址\n"
	@printf "  make export-openapi - 导出 OpenAPI JSON 文件\n"
	@printf "\n"
