-- Migration 003: Gamification System
-- Avatar companions, coins, shop, daily rewards, achievements

-- ============================================================================
-- AVATAR - User's companion creature
-- ============================================================================
CREATE TABLE avatar (
    id TEXT PRIMARY KEY DEFAULT 'default',
    creature_type TEXT NOT NULL DEFAULT 'cat',  -- cat, slime, octopus, snail
    name TEXT NOT NULL DEFAULT 'Whiskers',
    last_active_date TEXT,                       -- For mood calculation
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Initialize default avatar
INSERT INTO avatar (id, creature_type, name) VALUES ('default', 'cat', 'Whiskers');

-- ============================================================================
-- WALLET - Coin balance
-- ============================================================================
CREATE TABLE wallet (
    id TEXT PRIMARY KEY DEFAULT 'default',
    coins INTEGER NOT NULL DEFAULT 0,
    lifetime_coins INTEGER NOT NULL DEFAULT 0    -- Total ever earned
);

-- Initialize wallet
INSERT INTO wallet (id, coins, lifetime_coins) VALUES ('default', 0, 0);

-- ============================================================================
-- EQUIPPED - Currently worn accessories
-- ============================================================================
CREATE TABLE equipped (
    id TEXT PRIMARY KEY DEFAULT 'default',
    hat_id TEXT,                                 -- References shop_items.id
    held_id TEXT,                                -- References shop_items.id
    aura_id TEXT,                                -- References shop_items.id
    background_id TEXT DEFAULT 'bg_default'      -- References shop_items.id
);

-- Initialize equipped
INSERT INTO equipped (id) VALUES ('default');

-- ============================================================================
-- INVENTORY - Owned items
-- ============================================================================
CREATE TABLE inventory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id TEXT NOT NULL UNIQUE,                -- References shop_items.id
    acquired_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- SHOP_ITEMS - Item catalog
-- ============================================================================
CREATE TABLE shop_items (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    slot TEXT NOT NULL,                          -- hat, held, aura, background
    price INTEGER NOT NULL,
    rarity TEXT NOT NULL DEFAULT 'common',       -- common, uncommon, rare, legendary
    unlock_level INTEGER DEFAULT 0,              -- Required level to purchase
    sprite_data TEXT                             -- For pixel art (base64 or path)
);

-- Seed shop items
-- HATS
INSERT INTO shop_items (id, name, description, slot, price, rarity, unlock_level) VALUES
    ('hat_tophat', 'Top Hat', 'A distinguished look', 'hat', 50, 'common', 0),
    ('hat_gradcap', 'Grad Cap', 'Scholarly achievement', 'hat', 50, 'common', 0),
    ('hat_sunhat', 'Sun Hat', 'Perfect for sunny study sessions', 'hat', 150, 'uncommon', 5),
    ('hat_cowboy', 'Cowboy Hat', 'Yeehaw, partner!', 'hat', 150, 'uncommon', 5),
    ('hat_crown', 'Crown', 'Rule your knowledge kingdom', 'hat', 400, 'rare', 15),
    ('hat_santa', 'Santa Hat', 'Ho ho ho!', 'hat', 400, 'rare', 10),
    ('hat_halo', 'Halo', 'Angelic intellect', 'hat', 1000, 'legendary', 25),
    ('hat_flame', 'Flame Crown', 'Knowledge burns bright', 'hat', 1000, 'legendary', 30);

-- HELD ITEMS
INSERT INTO shop_items (id, name, description, slot, price, rarity, unlock_level) VALUES
    ('held_book', 'Book', 'Never stop reading', 'held', 50, 'common', 0),
    ('held_coffee', 'Coffee Cup', 'Fuel for the mind', 'held', 50, 'common', 0),
    ('held_controller', 'Controller', 'Learning is a game', 'held', 150, 'uncommon', 5),
    ('held_sword', 'Sword', 'Slice through problems', 'held', 150, 'uncommon', 8),
    ('held_wand', 'Magic Wand', 'Magical understanding', 'held', 400, 'rare', 15),
    ('held_crystal', 'Crystal', 'Clarity of thought', 'held', 1000, 'legendary', 25);

-- AURAS
INSERT INTO shop_items (id, name, description, slot, price, rarity, unlock_level) VALUES
    ('aura_sparkles', 'Sparkles', 'A gentle shimmer', 'aura', 75, 'common', 0),
    ('aura_hearts', 'Hearts', 'Love for learning', 'aura', 75, 'common', 0),
    ('aura_stars', 'Stars', 'Reach for the stars', 'aura', 200, 'uncommon', 5),
    ('aura_bubbles', 'Bubbles', 'Floating ideas', 'aura', 200, 'uncommon', 5),
    ('aura_flames', 'Flames', 'Burning passion', 'aura', 500, 'rare', 15),
    ('aura_snowflakes', 'Snowflakes', 'Cool under pressure', 'aura', 500, 'rare', 15),
    ('aura_rainbow', 'Rainbow', 'Full spectrum knowledge', 'aura', 1200, 'legendary', 25),
    ('aura_lightning', 'Lightning', 'Electric genius', 'aura', 1200, 'legendary', 30);

-- BACKGROUNDS
INSERT INTO shop_items (id, name, description, slot, price, rarity, unlock_level) VALUES
    ('bg_default', 'Cozy Room', 'Home sweet home', 'background', 0, 'common', 0),
    ('bg_forest', 'Forest Glade', 'Study among the trees', 'background', 100, 'common', 0),
    ('bg_beach', 'Beach', 'Sandy study spot', 'background', 250, 'uncommon', 5),
    ('bg_night', 'Night Sky', 'Under the stars', 'background', 250, 'uncommon', 5),
    ('bg_castle', 'Castle', 'Royal learning', 'background', 600, 'rare', 15),
    ('bg_space', 'Space Station', 'Study in orbit', 'background', 600, 'rare', 20),
    ('bg_galaxy', 'Galaxy', 'Universal knowledge', 'background', 1500, 'legendary', 30);

-- Give user the default background
INSERT INTO inventory (item_id) VALUES ('bg_default');

-- ============================================================================
-- DAILY LOGIN - 7-day reward cycle
-- ============================================================================
CREATE TABLE daily_login (
    id TEXT PRIMARY KEY DEFAULT 'default',
    current_day INTEGER NOT NULL DEFAULT 0,      -- 1-7, 0 = never claimed
    last_claim_date TEXT,                        -- YYYY-MM-DD
    total_claims INTEGER NOT NULL DEFAULT 0
);

-- Initialize
INSERT INTO daily_login (id) VALUES ('default');

-- ============================================================================
-- DAILY CHALLENGES - Random daily tasks
-- ============================================================================
CREATE TABLE daily_challenges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,                          -- YYYY-MM-DD
    challenge_type TEXT NOT NULL,                -- complete_sprints, score_80, perfect, review, voice
    description TEXT NOT NULL,
    target INTEGER NOT NULL,                     -- Goal amount
    progress INTEGER NOT NULL DEFAULT 0,         -- Current progress
    reward_coins INTEGER NOT NULL,
    completed INTEGER NOT NULL DEFAULT 0,        -- Boolean
    claimed INTEGER NOT NULL DEFAULT 0           -- Boolean
);

CREATE INDEX idx_daily_challenges_date ON daily_challenges(date);

-- ============================================================================
-- WEEKLY GOALS (replace existing partial table)
-- ============================================================================
-- Drop if exists and recreate with proper structure for gamification
DROP TABLE IF EXISTS weekly_goals;

CREATE TABLE weekly_goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    week_start TEXT NOT NULL,                    -- YYYY-MM-DD (Monday)
    goal_type TEXT NOT NULL,                     -- sprints, streak, perfect, mastery
    description TEXT NOT NULL,
    target INTEGER NOT NULL,
    progress INTEGER NOT NULL DEFAULT 0,
    reward_coins INTEGER NOT NULL,
    completed INTEGER NOT NULL DEFAULT 0,
    claimed INTEGER NOT NULL DEFAULT 0,
    UNIQUE(week_start, goal_type)
);

CREATE INDEX idx_weekly_goals_week ON weekly_goals(week_start);

-- ============================================================================
-- ACHIEVEMENTS - Unlockable badges with coin rewards
-- ============================================================================
CREATE TABLE achievements (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,                      -- learning, streak, collection, secret
    icon TEXT NOT NULL,
    reward_coins INTEGER NOT NULL DEFAULT 0,
    secret INTEGER NOT NULL DEFAULT 0,           -- Hidden until unlocked
    requirement_type TEXT NOT NULL,              -- sprints, perfect, streak, purchase, etc.
    requirement_value INTEGER NOT NULL           -- Threshold to unlock
);

-- LEARNING achievements
INSERT INTO achievements (id, name, description, category, icon, reward_coins, requirement_type, requirement_value) VALUES
    ('ach_first_sprint', 'First Steps', 'Complete your first sprint', 'learning', '📚', 25, 'sprints_total', 1),
    ('ach_sprint_10', 'Getting Started', 'Complete 10 sprints', 'learning', '📖', 50, 'sprints_total', 10),
    ('ach_sprint_50', 'Dedicated Learner', 'Complete 50 sprints', 'learning', '🎯', 150, 'sprints_total', 50),
    ('ach_sprint_100', 'Century', 'Complete 100 sprints', 'learning', '💯', 300, 'sprints_total', 100),
    ('ach_perfect_1', 'Sharpshooter', 'Get 100% on a sprint', 'learning', '🎯', 50, 'perfect_sprints', 1),
    ('ach_perfect_10', 'Perfectionist', 'Get 100% on 10 sprints', 'learning', '💎', 200, 'perfect_sprints', 10),
    ('ach_speed', 'Speed Demon', 'Complete a sprint under 2 minutes', 'learning', '⚡', 75, 'speed_sprint', 1);

-- STREAK achievements
INSERT INTO achievements (id, name, description, category, icon, reward_coins, requirement_type, requirement_value) VALUES
    ('ach_streak_7', 'On Fire', '7-day streak', 'streak', '🔥', 50, 'streak', 7),
    ('ach_streak_30', 'Month Master', '30-day streak', 'streak', '🌟', 200, 'streak', 30),
    ('ach_streak_100', 'Unstoppable', '100-day streak', 'streak', '💫', 500, 'streak', 100),
    ('ach_streak_365', 'Year Legend', '365-day streak', 'streak', '👑', 2000, 'streak', 365);

-- COLLECTION achievements
INSERT INTO achievements (id, name, description, category, icon, reward_coins, requirement_type, requirement_value) VALUES
    ('ach_first_buy', 'Shopper', 'Buy your first item', 'collection', '🛒', 25, 'purchases', 1),
    ('ach_items_5', 'Collector', 'Own 5 items', 'collection', '📦', 75, 'inventory_count', 5),
    ('ach_items_20', 'Hoarder', 'Own 20 items', 'collection', '🏠', 200, 'inventory_count', 20),
    ('ach_full_outfit', 'Fashionista', 'Equip all 3 accessory slots', 'collection', '✨', 100, 'equipped_slots', 3),
    ('ach_legendary', 'Legendary', 'Own a legendary item', 'collection', '🌈', 150, 'legendary_owned', 1);

-- SECRET achievements
INSERT INTO achievements (id, name, description, category, icon, reward_coins, secret, requirement_type, requirement_value) VALUES
    ('ach_night_owl', 'Night Owl', 'Complete a sprint between 2-5 AM', 'secret', '🦉', 100, 1, 'night_sprint', 1),
    ('ach_early_bird', 'Early Bird', 'Complete a sprint before 6 AM', 'secret', '🐦', 100, 1, 'early_sprint', 1),
    ('ach_comeback', 'Comeback', 'Return after 30+ days away', 'secret', '💪', 75, 1, 'comeback', 1),
    ('ach_holiday', 'Dedicated', 'Study on a holiday', 'secret', '🎄', 50, 1, 'holiday_sprint', 1);

-- ============================================================================
-- UNLOCKED ACHIEVEMENTS - Track what user has earned
-- ============================================================================
CREATE TABLE unlocked_achievements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    achievement_id TEXT NOT NULL UNIQUE REFERENCES achievements(id),
    unlocked_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- PURCHASE HISTORY - Track all purchases
-- ============================================================================
CREATE TABLE purchase_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id TEXT NOT NULL,
    price_paid INTEGER NOT NULL,
    purchased_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- COIN TRANSACTIONS - Audit log for coin changes
-- ============================================================================
CREATE TABLE coin_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    amount INTEGER NOT NULL,                     -- Positive = earned, negative = spent
    reason TEXT NOT NULL,                        -- sprint_pass, daily_reward, achievement, purchase, etc.
    reference_id TEXT,                           -- ID of related entity
    balance_after INTEGER NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_coin_tx_timestamp ON coin_transactions(timestamp);
