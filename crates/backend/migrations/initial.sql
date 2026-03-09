-- 001_initial.sql
-- 初始化数据库表结构
--
-- ========== 天气数据表 ==========
CREATE TABLE IF NOT EXISTS weather (
    id INTEGER PRIMARY KEY,
    type INTEGER NOT NULL,
    temperature REAL NOT NULL,
    duration INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

-- ========== 顾客表 ==========
-- CustomerType: 0=Normal, 1=Foodie, 2=Critic
CREATE TABLE IF NOT EXISTS customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    age INTEGER NOT NULL DEFAULT 30,
    occupation TEXT NOT NULL DEFAULT '',
    customer_type INTEGER NOT NULL DEFAULT 0,
    affinity INTEGER NOT NULL DEFAULT 0,
    visit_count INTEGER NOT NULL DEFAULT 0,
    story_background TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- ========== 顾客偏好表（作为顾客的子表）==========
-- FlavorPreference: 0=Light, 1=Medium, 2=Heavy, 3=Spicy, 4=SweetSour
-- DietaryRestriction: 0=None, 1=Vegetarian, 2=Halal, 3=GlutenFree, 4=LowSugar
CREATE TABLE IF NOT EXISTS preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    flavor INTEGER NOT NULL DEFAULT 0,
    dietary INTEGER NOT NULL DEFAULT 0,
    price_sensitivity INTEGER NOT NULL DEFAULT 50,
    patience INTEGER NOT NULL DEFAULT 75,
    favorite_categories TEXT NOT NULL DEFAULT '[]',
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_preferences_customer_id ON preferences(customer_id);
