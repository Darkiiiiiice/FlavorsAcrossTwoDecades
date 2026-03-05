//! 集成测试

use chrono::Utc;
use flavors_backend::db::models::{
    command::Command,
    customer::CustomerRecord,
    dialogue::DialogueMessage,
    memory::MemoryFragment,
    panpan::PanpanState,
    recipe::Recipe,
    save::Save,
    shop::ShopState,
    travel::Travel,
};
use flavors_backend::db::repositories::{
    CommandRepository, CustomerRepository, DialogueRepository, GardenRepository,
    MemoryRepository, PanpanRepository, RecipeRepository, SaveRepository,
    ShopRepository, TravelRepository,
};
use flavors_backend::game::recipe::{RecipeCategory, RecipeSource, RecipeStatus};
use flavors_backend::game::travel::TravelStatus;
use sqlx::SqlitePool;
use uuid::Uuid;

/// 创建内存数据库用于测试
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");
    
    // 运行迁移
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS saves (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            player_name TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            play_time_seconds INTEGER NOT NULL,
            chapter INTEGER NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS commands (
            id TEXT PRIMARY KEY,
            save_id TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            arrival_time TEXT NOT NULL,
            status TEXT NOT NULL,
            result TEXT
        );
        
        CREATE TABLE IF NOT EXISTS dialogues (
            id TEXT PRIMARY KEY,
            save_id TEXT NOT NULL,
            sender TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            message_type TEXT NOT NULL,
            status TEXT NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS recipes (
            id TEXT PRIMARY KEY,
            save_id TEXT NOT NULL,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            status TEXT NOT NULL,
            ingredients TEXT NOT NULL,
            source TEXT NOT NULL,
            unlock_condition TEXT
        );
        
        CREATE TABLE IF NOT EXISTS customers (
            id TEXT PRIMARY KEY,
            save_id TEXT NOT NULL,
            customer_type TEXT NOT NULL,
            name TEXT NOT NULL,
            favorability INTEGER NOT NULL,
            visit_count INTEGER NOT NULL,
            last_visit TEXT NOT NULL,
            preferences TEXT NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS memory_fragments (
            id TEXT PRIMARY KEY,
            save_id TEXT NOT NULL,
            fragment_type TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            is_unlocked INTEGER NOT NULL,
            unlocked_at TEXT,
            trigger_condition TEXT NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS panpan_states (
            save_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            model TEXT NOT NULL,
            manufacture_date TEXT NOT NULL,
            personality TEXT NOT NULL,
            trust_level INTEGER NOT NULL,
            emotion TEXT NOT NULL,
            energy_current INTEGER NOT NULL,
            energy_max INTEGER NOT NULL,
            location TEXT NOT NULL,
            current_state TEXT NOT NULL,
            current_task TEXT
        );
        
        CREATE TABLE IF NOT EXISTS garden_plots (
            id TEXT PRIMARY KEY,
            save_id TEXT NOT NULL,
            plot_number INTEGER NOT NULL,
            is_unlocked INTEGER NOT NULL,
            current_crop TEXT,
            fertility INTEGER NOT NULL,
            moisture INTEGER NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS shop_states (
            save_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            funds INTEGER NOT NULL,
            reputation REAL NOT NULL,
            restaurant_level INTEGER NOT NULL,
            kitchen_level INTEGER NOT NULL,
            backyard_level INTEGER NOT NULL,
            workshop_level INTEGER NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS travels (
            id TEXT PRIMARY KEY,
            save_id TEXT NOT NULL,
            destination TEXT NOT NULL,
            started_at TEXT NOT NULL,
            expected_return TEXT NOT NULL,
            status TEXT NOT NULL,
            rewards TEXT
        );
    "#)
    .execute(&pool)
    .await
    .expect("Failed to run migrations");
    
    pool
}

/// 创建测试存档
fn create_test_save() -> Save {
    Save::new("测试存档".to_string(), "测试玩家".to_string())
}

// ========== 存档测试 ==========

#[tokio::test]
async fn test_save_repository() {
    let pool = setup_test_db().await;
    let repo = SaveRepository::new(pool);
    
    // 创建存档
    let save = create_test_save();
    repo.create(&save).await.expect("Failed to create save");
    
    // 查找存档
    let found = repo.find_by_id(save.id).await.expect("Failed to find save");
    assert!(found.is_some());
    assert_eq!(found.unwrap().player_name, "测试玩家");
    
    // 列出所有存档
    let all = repo.find_all().await.expect("Failed to list saves");
    assert_eq!(all.len(), 1);
    
    // 删除存档
    repo.delete(save.id).await.expect("Failed to delete save");
    let found = repo.find_by_id(save.id).await.expect("Failed to find save");
    assert!(found.is_none());
}

// ========== 指令测试 ==========

#[tokio::test]
async fn test_command_repository() {
    let pool = setup_test_db().await;
    let save_repo = SaveRepository::new(pool.clone());
    let cmd_repo = CommandRepository::new(pool);
    
    // 创建存档
    let save = create_test_save();
    save_repo.create(&save).await.expect("Failed to create save");
    
    // 创建指令
    let cmd = Command::new(save.id, "测试指令".to_string(), 60);
    cmd_repo.create(&cmd).await.expect("Failed to create command");
    
    // 查找指令
    let found = cmd_repo.find_by_id(cmd.id).await.expect("Failed to find command");
    assert!(found.is_some());
    assert_eq!(found.unwrap().content, "测试指令");
    
    // 按存档查找
    let cmds = cmd_repo.find_by_save_id(save.id).await.expect("Failed to list commands");
    assert_eq!(cmds.len(), 1);
}

// ========== 对话测试 ==========

#[tokio::test]
async fn test_dialogue_repository() {
    let pool = setup_test_db().await;
    let save_repo = SaveRepository::new(pool.clone());
    let dialogue_repo = DialogueRepository::new(pool);
    
    // 创建存档
    let save = create_test_save();
    save_repo.create(&save).await.expect("Failed to create save");
    
    // 创建对话
    let msg = DialogueMessage::new(save.id, "player".to_string(), "你好".to_string(), "chat".to_string());
    dialogue_repo.create(&msg).await.expect("Failed to create dialogue");
    
    // 查找对话
    let msgs = dialogue_repo.find_by_save_id(save.id).await.expect("Failed to list dialogues");
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].content, "你好");
}

// ========== 菜谱测试 ==========

#[tokio::test]
async fn test_recipe_repository() {
    let pool = setup_test_db().await;
    let save_repo = SaveRepository::new(pool.clone());
    let recipe_repo = RecipeRepository::new(pool);
    
    // 创建存档
    let save = create_test_save();
    save_repo.create(&save).await.expect("Failed to create save");
    
    // 创建菜谱
    let recipe = Recipe {
        id: Uuid::new_v4(),
        save_id: save.id,
        name: "宫保鸡丁".to_string(),
        category: RecipeCategory::Sichuan,
        status: RecipeStatus::Precise,
        ingredients: r#"[{"name":"鸡肉","amount":"300g"}]"#.to_string(),
        source: RecipeSource::Inherited,
        unlock_condition: Some("信任度达到50".to_string()),
    };
    recipe_repo.create(&recipe).await.expect("Failed to create recipe");
    
    // 查找菜谱
    let found = recipe_repo.find_by_id(recipe.id).await.expect("Failed to find recipe");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "宫保鸡丁");
    
    // 更新状态
    recipe_repo.update_status(recipe.id, RecipeStatus::Mastered).await.expect("Failed to update status");
    let found = recipe_repo.find_by_id(recipe.id).await.expect("Failed to find recipe");
    assert_eq!(found.unwrap().status, RecipeStatus::Mastered);
}

// ========== 顾客测试 ==========

#[tokio::test]
async fn test_customer_repository() {
    let pool = setup_test_db().await;
    let save_repo = SaveRepository::new(pool.clone());
    let customer_repo = CustomerRepository::new(pool);
    
    // 创建存档
    let save = create_test_save();
    save_repo.create(&save).await.expect("Failed to create save");
    
    // 创建顾客
    let customer = CustomerRecord {
        id: Uuid::new_v4(),
        save_id: save.id,
        customer_type: "regular".to_string(),
        name: "张三".to_string(),
        favorability: 50,
        visit_count: 5,
        last_visit: Utc::now(),
        preferences: r#"["辣菜"]"#.to_string(),
    };
    customer_repo.create(&customer).await.expect("Failed to create customer");
    
    // 查找顾客
    let found = customer_repo.find_by_id(customer.id).await.expect("Failed to find customer");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "张三");
    
    // 更新顾客
    let mut updated = customer.clone();
    updated.favorability = 80;
    customer_repo.update(&updated).await.expect("Failed to update customer");
    let found = customer_repo.find_by_id(customer.id).await.expect("Failed to find customer");
    assert_eq!(found.unwrap().favorability, 80);
    
    // 删除顾客
    customer_repo.delete(customer.id).await.expect("Failed to delete customer");
    let found = customer_repo.find_by_id(customer.id).await.expect("Failed to find customer");
    assert!(found.is_none());
}

// ========== 记忆碎片测试 ==========

#[tokio::test]
async fn test_memory_repository() {
    let pool = setup_test_db().await;
    let save_repo = SaveRepository::new(pool.clone());
    let memory_repo = MemoryRepository::new(pool);
    
    // 创建存档
    let save = create_test_save();
    save_repo.create(&save).await.expect("Failed to create save");
    
    // 创建记忆碎片
    let fragment = MemoryFragment {
        id: Uuid::new_v4(),
        save_id: save.id,
        fragment_type: "childhood".to_string(),
        title: "童年的味道".to_string(),
        content: "奶奶做的红烧肉...".to_string(),
        is_unlocked: false,
        unlocked_at: None,
        trigger_condition: "品尝红烧肉".to_string(),
    };
    memory_repo.create(&fragment).await.expect("Failed to create memory");
    
    // 查找记忆
    let found = memory_repo.find_by_id(fragment.id).await.expect("Failed to find memory");
    assert!(found.is_some());
    assert_eq!(found.unwrap().title, "童年的味道");
    
    // 解锁记忆
    let mut updated = fragment.clone();
    updated.is_unlocked = true;
    updated.unlocked_at = Some(Utc::now());
    memory_repo.update(&updated).await.expect("Failed to update memory");
    
    // 查找已解锁的记忆
    let unlocked = memory_repo.find_unlocked(save.id).await.expect("Failed to find unlocked memories");
    assert_eq!(unlocked.len(), 1);
}

// ========== 盼盼状态测试 ==========

#[tokio::test]
async fn test_panpan_repository() {
    let pool = setup_test_db().await;
    let save_repo = SaveRepository::new(pool.clone());
    let panpan_repo = PanpanRepository::new(pool);
    
    // 创建存档
    let save = create_test_save();
    save_repo.create(&save).await.expect("Failed to create save");
    
    // 创建盼盼状态
    let panpan = PanpanState::new(save.id);
    panpan_repo.create(&panpan).await.expect("Failed to create panpan state");
    
    // 查找盼盼状态
    let found = panpan_repo.find_by_save_id(save.id).await.expect("Failed to find panpan state");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "盼盼");
    
    // 更新盼盼状态
    let mut updated = panpan.clone();
    updated.trust_level = 80;
    panpan_repo.update(&updated).await.expect("Failed to update panpan state");
    let found = panpan_repo.find_by_save_id(save.id).await.expect("Failed to find panpan state");
    assert_eq!(found.unwrap().trust_level, 80);
}

// ========== 菜园测试 ==========

#[tokio::test]
async fn test_garden_repository() {
    let pool = setup_test_db().await;
    let save_repo = SaveRepository::new(pool.clone());
    let garden_repo = GardenRepository::new(pool);
    
    // 创建存档
    let save = create_test_save();
    save_repo.create(&save).await.expect("Failed to create save");
    
    // 初始化菜园
    let plots = garden_repo.initialize_plots(save.id).await.expect("Failed to initialize plots");
    assert_eq!(plots.len(), 6);
    
    // 验证第一块地已解锁
    let first_plot = plots.iter().find(|p| p.plot_number == 1).unwrap();
    assert!(first_plot.is_unlocked);
    
    // 更新地块状态
    let mut plot = first_plot.clone();
    plot.current_crop = Some(r#"{"type":"tomato"}"#.to_string());
    garden_repo.update(&plot).await.expect("Failed to update plot");
    
    let found = garden_repo.find_by_id(plot.id).await.expect("Failed to find plot");
    assert!(found.unwrap().current_crop.is_some());
}

// ========== 小馆状态测试 ==========

#[tokio::test]
async fn test_shop_repository() {
    let pool = setup_test_db().await;
    let save_repo = SaveRepository::new(pool.clone());
    let shop_repo = ShopRepository::new(pool);
    
    // 创建存档
    let save = create_test_save();
    save_repo.create(&save).await.expect("Failed to create save");
    
    // 创建小馆状态
    let shop = ShopState::new(save.id);
    shop_repo.create(&shop).await.expect("Failed to create shop state");
    
    // 查找小馆状态
    let found = shop_repo.find_by_save_id(save.id).await.expect("Failed to find shop state");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "星夜小馆");
    
    // 更新资金
    let mut updated = shop.clone();
    updated.funds = 20000;
    shop_repo.update(&updated).await.expect("Failed to update shop state");
    let found = shop_repo.find_by_save_id(save.id).await.expect("Failed to find shop state");
    assert_eq!(found.unwrap().funds, 20000);
}

// ========== 旅行测试 ==========

#[tokio::test]
async fn test_travel_repository() {
    let pool = setup_test_db().await;
    let save_repo = SaveRepository::new(pool.clone());
    let travel_repo = TravelRepository::new(pool);
    
    // 创建存档
    let save = create_test_save();
    save_repo.create(&save).await.expect("Failed to create save");
    
    // 创建旅行
    let travel = Travel::new(save.id, "四川成都".to_string(), 7);
    travel_repo.create(&travel).await.expect("Failed to create travel");
    
    // 查找旅行
    let found = travel_repo.find_by_id(travel.id).await.expect("Failed to find travel");
    assert!(found.is_some());
    assert_eq!(found.unwrap().destination, "四川成都");
    
    // 完成旅行
    let mut completed = travel.clone();
    completed.status = TravelStatus::Completed;
    completed.rewards = Some(r#"[{"type":"recipe","name":"麻婆豆腐"}]"#.to_string());
    travel_repo.update(&completed).await.expect("Failed to update travel");
    
    let found = travel_repo.find_by_id(travel.id).await.expect("Failed to find travel");
    assert_eq!(found.unwrap().status, TravelStatus::Completed);
}
