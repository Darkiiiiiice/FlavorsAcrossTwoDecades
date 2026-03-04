# Phase 5: 部署（第11周）

## 开发目标

- [ ] systemd 服务配置
- [ ] 生产环境配置
- [ ] 部署文档

---

## 一、systemd 服务配置

### 1.1 服务文件

```ini
# /etc/systemd/system/flavors-game.service
[Unit]
Description=Flavors Across Two Decades Game Server
Documentation=https://github.com/Darkiiiiiice/FlavorsAcrossTwoDecades
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=notify
User=game
Group=game
WorkingDirectory=/opt/flavors-game

# 主程序
ExecStart=/usr/local/bin/flavors-game --config /opt/flavors-game/config/production.toml

# 重启策略
Restart=always
RestartSec=10

# 资源限制
LimitNOFILE=65535
MemoryMax=512M

# 健康检查
WatchdogSec=30
Environment=RUST_LOG=info

# 安全设置
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/opt/flavors-game/data

[Install]
WantedBy=multi-user.target
```

### 1.2 服务管理命令

```bash
# 安装服务
sudo cp flavors-game.service /etc/systemd/system/
sudo systemctl daemon-reload

# 启动服务
sudo systemctl start flavors-game

# 停止服务
sudo systemctl stop flavors-game

# 重启服务
sudo systemctl restart flavors-game

# 查看状态
sudo systemctl status flavors-game

# 查看日志
sudo journalctl -u flavors-game -f

# 开机自启
sudo systemctl enable flavors-game
```

---

## 二、生产环境配置

### 2.1 配置文件

```toml
# config/production.toml

[server]
host = "127.0.0.1"
port = 8080
workers = 4

[database]
path = "/opt/flavors-game/data/game.db"
pool_size = 10

[llm]
provider = "ollama"

[llm.ollama]
base_url = "http://localhost:11434"
model = "qwen2.5:7b"
temperature = 0.8
max_tokens = 2048
timeout_seconds = 60

[llm.prompt]
system_prompt_path = "/opt/flavors-game/prompts/system_prompt.hbs"
max_context_length = 4096

[time]
acceleration = 1
timezone = "Asia/Shanghai"
business_hours_start = 7
business_hours_end = 24

[communication]
base_delay_seconds = 60
default_module_level = 1

[logging]
level = "info"
format = "json"
output = "/opt/flavors-game/logs/game.log"

[backup]
enabled = true
interval_hours = 24
keep_days = 7
path = "/opt/flavors-game/backups"
```

### 2.2 日志配置

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging(config: &LoggingConfig) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_ansi(false)
                .json()
        )
        .with(
            tracing_subscriber::filter::LevelFilter::from_str(&config.level)
                .unwrap_or(tracing_subscriber::filter::LevelFilter::INFO)
        )
        .init();
}
```

### 2.3 数据库备份

```bash
#!/bin/bash
# /opt/flavors-game/scripts/backup.sh

BACKUP_DIR="/opt/flavors-game/backups"
DB_PATH="/opt/flavors-game/data/game.db"
DATE=$(date +%Y%m%d_%H%M%S)
KEEP_DAYS=7

# 创建备份目录
mkdir -p $BACKUP_DIR

# 备份数据库
sqlite3 $DB_PATH ".backup '$BACKUP_DIR/game_$DATE.db'"

# 压缩备份
gzip "$BACKUP_DIR/game_$DATE.db"

# 删除旧备份
find $BACKUP_DIR -name "*.gz" -mtime +$KEEP_DAYS -delete

echo "Backup completed: game_$DATE.db.gz"
```

---

## 三、项目目录结构

```
flavors-game/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
│
├── config/
│   ├── default.toml           # 默认配置
│   └── production.toml        # 生产配置
│
├── migrations/
│   ├── 001_initial.sql        # 初始化 Schema
│   └── 002_add_dialogues.sql  # 后续迁移
│
├── prompts/
│   ├── system_prompt.hbs      # 系统提示词
│   ├── command_decision.hbs   # 指令决策
│   ├── daily_report.hbs       # 每日简报
│   └── travel_log.hbs         # 旅行日志
│
├── scripts/
│   ├── backup.sh              # 备份脚本
│   ├── install.sh             # 安装脚本
│   └── start.sh               # 启动脚本
│
├── src/
│   ├── main.rs                # 入口
│   ├── config.rs              # 配置管理
│   ├── error.rs               # 错误定义
│   │
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs          # 路由定义
│   │   ├── health.rs          # 健康检查
│   │   ├── saves.rs           # 存档 API
│   │   ├── panpan.rs          # 盼盼 API
│   │   ├── shop.rs            # 小馆 API
│   │   ├── garden.rs          # 种植 API
│   │   ├── travel.rs          # 旅行 API
│   │   └── ws.rs              # WebSocket
│   │
│   ├── db/
│   │   ├── mod.rs
│   │   ├── pool.rs            # 连接池
│   │   └── migrations.rs      # 迁移管理
│   │
│   ├── game/
│   │   ├── mod.rs
│   │   ├── engine.rs          # 游戏引擎
│   │   ├── time.rs            # 时间系统
│   │   └── event.rs           # 事件系统
│   │
│   ├── llm/
│   │   ├── mod.rs
│   │   ├── provider.rs        # LLM 提供者
│   │   ├── templates.rs       # Prompt 模板
│   │   └── context.rs         # 上下文管理
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── save.rs            # 存档模型
│   │   ├── panpan.rs          # 盼盼模型
│   │   ├── shop.rs            # 小馆模型
│   │   ├── garden.rs          # 种植模型
│   │   ├── travel.rs          # 旅行模型
│   │   ├── recipe.rs          # 菜谱模型
│   │   ├── memory.rs          # 记忆模型
│   │   └── customer.rs        # 顾客模型
│   │
│   ├── repositories/
│   │   ├── mod.rs
│   │   ├── save.rs            # 存档仓储
│   │   ├── panpan.rs          # 盼盼仓储
│   │   ├── shop.rs            # 小馆仓储
│   │   ├── garden.rs          # 种植仓储
│   │   └── common.rs          # 通用仓储
│   │
│   └── subsystems/
│       ├── mod.rs
│       ├── panpan.rs          # 盼盼子系统
│       ├── shop.rs            # 小馆子系统
│       ├── garden.rs          # 种植子系统
│       ├── kitchen.rs         # 厨房子系统
│       ├── workshop.rs        # 工坊子系统
│       ├── travel.rs          # 旅行子系统
│       ├── recipe.rs          # 菜谱子系统
│       ├── memory.rs          # 记忆子系统
│       └── customer.rs        # 顾客子系统
│
├── tests/
│   ├── integration/
│   │   ├── api_test.rs        # API 集成测试
│   │   └── game_flow_test.rs  # 游戏流程测试
│   │
│   └── fixtures/
│       └── test_data.json     # 测试数据
│
└── debian/
    ├── flavors-game.service   # systemd 服务
    └── postinst               # 安装后脚本
```

---

## 四、安装脚本

```bash
#!/bin/bash
# scripts/install.sh

set -e

echo "Installing Flavors Across Two Decades Game Server..."

# 检查依赖
command -v rustc >/dev/null 2>&1 || { echo "Rust is required"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Cargo is required"; exit 1; }

# 创建用户
if ! id "game" &>/dev/null; then
    sudo useradd -r -s /bin/false game
fi

# 创建目录
sudo mkdir -p /opt/flavors-game/{data,logs,backups,prompts,config}

# 编译
echo "Building..."
cargo build --release

# 安装二进制
sudo cp target/release/flavors-game /usr/local/bin/

# 安装配置
sudo cp config/production.toml /opt/flavors-game/config/
sudo cp prompts/*.hbs /opt/flavors-game/prompts/

# 设置权限
sudo chown -R game:game /opt/flavors-game

# 安装 systemd 服务
sudo cp debian/flavors-game.service /etc/systemd/system/
sudo systemctl daemon-reload

# 设置备份定时任务
(crontab -l 2>/dev/null; echo "0 3 * * * /opt/flavors-game/scripts/backup.sh") | crontab -

echo "Installation complete!"
echo "Run 'sudo systemctl start flavors-game' to start the server."
```

---

## 五、已确认的设计决策

| 问题 | 决策 |
|------|------|
| 盼盼 AI 决策 | 接入 Ollama（第一版），盼盼所有行为由 AI 控制 |
| 存档机制 | 多存档，后端提供 CRUD API，前端管理存档选择 |
| **数据持久化** | 不使用 SaveState 聚合模型，所有实体独立存储，通过 save_id 关联，实时写入 SQLite |
| 图像资源 | 暂不实现，后续迭代，第一版使用文字描述 |
| 时间控制 | 正常模式 1:1 同步，测试模式可配置加速倍率（默认 10 倍）|
| **模块系统** | 7 个模块，等级 1-10，整合技能和健康度（完好度 0-100）|
| **通信延迟** | 基础延迟（物理）+ 模块延迟（可升级），随盼盼升级而降低 |
| **性格轴** | 3 维，范围 0-100，初始 50 |
| **信任度** | 0-100，初始 50，影响记忆恢复和主动行为 |
| **情绪系统** | 完整实现 7 种情绪（开心/平静/疲惫/困惑/担忧/孤独/兴奋）|
| **能量系统** | 0-100，不同活动消耗不同，可充电 |
| **记忆容量** | 初始 100，最大 500，5 种碎片类型 |
| **小馆区域** | 4 大区域（餐厅/厨房/后院/工坊），每区域独立等级 1-5 |
| **设施数量** | 20+ 子设施，每个有等级（1-5）和完好度（0-100%）|
| **口碑系统** | 5 维度加权计算（菜品 40%、服务 20%、环境 15%、邻里 15%、老顾客 10%）|
| **氛围指数** | 5 子项加权（照明 25%、温度 20%、清洁 20%、装饰 20%、音乐 15%）|
| **客流计算** | 口碑 + 季节 + 氛围 + 座位翻台率综合计算 |
| **满意度计算** | 菜品 50% + 服务 30% + 环境 20% |
| **菜品体系** | 3 种来源（传承/旅行/创新），4 种状态（损坏/模糊/精确/掌握）|
| **升级系统** | 每个设施有独立升级路径，需资金/材料/时间/人员 |
| **里程碑** | 每区域有里程碑系统，完成解锁奖励 |
| **天气系统** | 4 种天气类型（晴/雨/雪/阴），4 季节循环，影响客流和种植 |
| **节假日系统** | 内置中国传统节日，特殊事件和客流加成，顾客行为变化 |
| **邻里系统** | 5+ 邻居角色，好感度 0-100，互助事件，可提供服务和材料 |
| **供应商系统** | 3 类供应商（食材/设备/杂货），品质/价格/配送时间权衡选择 |
| **成就系统** | 5 大类别（经营/探索/社交/烹饪/收集），隐藏成就和里程碑成就 |
| **教程系统** | 5 阶段引导（基础/进阶/高级/专家/隐藏），可跳过，上下文感知提示 |
| **统计系统** | 7 类统计数据（财务/客流/顾客/菜品/运营/里程碑/趋势），支持可视化 |
| **数据库索引** | 为高频查询字段建立索引（save_id、时间戳、类型字段等） |
| **人员管理** | 无员工系统，盼盼独立管理所有功能，体现机器人主角特色 |
| **种植系统** | 后院 5 级等级，5 块菜地，5 类作物（蔬菜/香料/花卉/特殊/异星） |
| **作物生长** | 5 阶段生长（播种/发芽/生长/成熟/枯萎），受季节/天气/肥力影响 |
| **病虫害系统** | 12 种病虫害类型，5 级严重程度，10 种治疗方法 |
| **种植自动化** | 后院 3 级解锁自动浇水，4 级解锁自动虫检，5 级解锁自动收获 |
| **园艺模块** | 盼盼园艺技能 1-10 级，影响播种成功率/生长速度/产量/自留种能力 |

---

## 六、下一步

确认以上设计方案后，将按以下顺序实现：

1. **Phase 1**: 初始化项目，实现基础框架
2. **Phase 2**: 实现核心系统（时间、LLM、指令队列）
3. **Phase 3**: 实现游戏子系统
4. **Phase 4**: 整合测试，优化性能
5. **Phase 5**: 部署上线

---

## 七、常见问题

### Q1: 如何更新服务？

```bash
# 拉取最新代码
git pull

# 重新编译
cargo build --release

# 停止服务
sudo systemctl stop flavors-game

# 更新二进制
sudo cp target/release/flavors-game /usr/local/bin/

# 启动服务
sudo systemctl start flavors-game
```

### Q2: 如何查看日志？

```bash
# 实时日志
sudo journalctl -u flavors-game -f

# 最近100行
sudo journalctl -u flavors-game -n 100

# 今日日志
sudo journalctl -u flavors-game --since today
```

### Q3: 如何备份数据？

```bash
# 手动备份
/opt/flavors-game/scripts/backup.sh

# 备份文件位于
ls /opt/flavors-game/backups/
```

### Q4: 如何重置数据库？

```bash
# 停止服务
sudo systemctl stop flavors-game

# 备份现有数据
mv /opt/flavors-game/data/game.db /opt/flavors-game/data/game.db.bak

# 重启服务（会自动创建新数据库）
sudo systemctl start flavors-game
```
