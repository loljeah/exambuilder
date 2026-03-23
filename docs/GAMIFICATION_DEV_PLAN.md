# Gamification System — Development Plan

## Final Design Summary

### Creatures (Pick at Start)
```
🐱 Cat      — Default, classic companion
💧 Slime    — Dragon Quest teardrop style
🐙 Octopus  — Tentacled friend
🐌 Snail    — Slow and steady
```

### Mood System (Activity-Based)
```
😸 Happy    — Active today      → +15% XP bonus
😺 Content  — Active yesterday  → +5% XP bonus
😐 Neutral  — 2-3 days inactive → Normal XP
😿 Sad      — 4-6 days inactive → Normal XP (visual only)
😾 Lonely   — 7+ days inactive  → Normal XP (needs attention)
```

### Currency
- **Coins only** — single currency, simple balance
- Earned from: sprints, streaks, achievements, daily rewards

### Shop (Cosmetics Only)
- **3 accessory slots**: Hat, Held Item, Aura
- **Plus**: Background selection
- **Rarity tiers**: Common, Uncommon, Rare, Legendary
- No gameplay-affecting items

### Daily Systems
- **Login rewards**: 7-day cycle with escalating coins
- **Daily challenges**: 3 random tasks for bonus coins
- **Weekly goals**: 4 targets for big rewards

### Achievements
- Unlock coins + display badges
- Categories: learning, streaks, collection, secrets

### Display
- ASCII art in terminal with equipped accessories
- Tray icon shows avatar + mood
- Mood affects XP earning (+15% when happy)

---

## Database Schema

```sql
-- ============================================================
-- GAMIFICATION TABLES
-- ============================================================

-- Avatar (one per user)
CREATE TABLE avatar (
    id TEXT PRIMARY KEY DEFAULT 'default',
    creature_type TEXT NOT NULL DEFAULT 'cat',
    name TEXT NOT NULL DEFAULT 'Whiskers',
    mood TEXT NOT NULL DEFAULT 'neutral',
    last_active_date DATE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Wallet
CREATE TABLE wallet (
    id TEXT PRIMARY KEY DEFAULT 'default',
    coins INTEGER NOT NULL DEFAULT 0,
    lifetime_coins INTEGER NOT NULL DEFAULT 0
);

-- Equipped accessories
CREATE TABLE equipped (
    id TEXT PRIMARY KEY DEFAULT 'default',
    hat_id TEXT,
    held_id TEXT,
    aura_id TEXT,
    background_id TEXT DEFAULT 'default'
);

-- Owned items inventory
CREATE TABLE inventory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id TEXT NOT NULL UNIQUE,
    acquired_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Shop item catalog (seeded data)
CREATE TABLE shop_items (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    slot TEXT NOT NULL,  -- hat, held, aura, background
    price INTEGER NOT NULL,
    rarity TEXT NOT NULL DEFAULT 'common',  -- common, uncommon, rare, legendary
    unlock_level INTEGER DEFAULT 0,
    ascii_art TEXT  -- for rendering
);

-- Daily login tracking
CREATE TABLE daily_login (
    id TEXT PRIMARY KEY DEFAULT 'default',
    current_day INTEGER NOT NULL DEFAULT 0,
    last_claim_date DATE,
    total_claims INTEGER NOT NULL DEFAULT 0
);

-- Daily challenges
CREATE TABLE daily_challenges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date DATE NOT NULL,
    challenge_type TEXT NOT NULL,
    description TEXT NOT NULL,
    target INTEGER NOT NULL,
    progress INTEGER NOT NULL DEFAULT 0,
    reward_coins INTEGER NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    claimed BOOLEAN NOT NULL DEFAULT FALSE
);

-- Weekly goals
CREATE TABLE weekly_goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    week_start DATE NOT NULL,
    goal_type TEXT NOT NULL,
    description TEXT NOT NULL,
    target INTEGER NOT NULL,
    progress INTEGER NOT NULL DEFAULT 0,
    reward_coins INTEGER NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    claimed BOOLEAN NOT NULL DEFAULT FALSE
);

-- Achievements catalog
CREATE TABLE achievements (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,  -- learning, streak, collection, secret
    icon TEXT NOT NULL,
    reward_coins INTEGER NOT NULL DEFAULT 0,
    secret BOOLEAN NOT NULL DEFAULT FALSE,
    requirement_type TEXT NOT NULL,
    requirement_value INTEGER NOT NULL
);

-- Unlocked achievements
CREATE TABLE unlocked_achievements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    achievement_id TEXT NOT NULL UNIQUE,
    unlocked_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## Phase 1: Core Avatar (Week 1)

### Tasks

1. **Create gamification migration**
   - Add all tables above
   - Seed shop items (20-30 items)
   - Seed achievements (15-20 achievements)

2. **Avatar CRUD in db/gamification.go**
   ```go
   GetAvatar() (*Avatar, error)
   SetCreatureType(creatureType string) error
   SetAvatarName(name string) error
   UpdateMood() error  // Called on activity
   GetMood() string    // Calculate from last_active_date
   ```

3. **Wallet operations**
   ```go
   GetWallet() (*Wallet, error)
   AddCoins(amount int, reason string) error
   SpendCoins(amount int) error
   ```

4. **ASCII art rendering**
   ```go
   // internal/gamification/render.go
   RenderAvatar(avatar *Avatar, equipped *Equipped) string
   RenderCreature(creatureType, mood string) string
   RenderWithAccessories(base string, hat, held, aura string) string
   ```

5. **CLI commands**
   ```bash
   kgatectl avatar              # Show avatar with stats
   kgatectl avatar pick <type>  # Choose creature (first time)
   kgatectl avatar name <name>  # Rename
   ```

### Deliverables
- Avatar displays in terminal
- Mood updates based on activity
- Coins shown in status

---

## Phase 2: Shop System (Week 2)

### Tasks

1. **Shop data structures**
   ```go
   type ShopItem struct {
       ID          string
       Name        string
       Description string
       Slot        string  // hat, held, aura, background
       Price       int
       Rarity      string
       UnlockLevel int
       ASCIIArt    string
   }
   ```

2. **Shop operations**
   ```go
   GetShopItems(slot string) ([]ShopItem, error)
   GetShopItem(id string) (*ShopItem, error)
   CanAfford(itemID string) bool
   CanUnlock(itemID string) bool  // Level check
   PurchaseItem(itemID string) error
   ```

3. **Inventory operations**
   ```go
   GetInventory() ([]string, error)  // Item IDs owned
   OwnsItem(itemID string) bool
   ```

4. **Equipment operations**
   ```go
   GetEquipped() (*Equipped, error)
   EquipItem(itemID string) error
   UnequipSlot(slot string) error
   ```

5. **CLI commands**
   ```bash
   kgatectl shop                # Browse all
   kgatectl shop hats           # Filter by slot
   kgatectl shop buy <id>       # Purchase
   kgatectl inventory           # Show owned items
   kgatectl equip <id>          # Equip item
   kgatectl unequip <slot>      # Remove from slot
   ```

6. **Seed shop items**
   - 8 hats (2 per rarity)
   - 6 held items
   - 6 auras
   - 5 backgrounds

### Item Ideas

```
HATS (Common: 50c, Uncommon: 150c, Rare: 400c, Legendary: 1000c)
- 🎩 Top Hat (common)
- 🎓 Grad Cap (common)
- 👒 Sun Hat (uncommon)
- 🤠 Cowboy Hat (uncommon)
- 👑 Crown (rare)
- 🎄 Santa Hat (rare, seasonal)
- 🌟 Halo (legendary)
- 🔥 Flame Crown (legendary)

HELD ITEMS
- 📚 Book (common)
- ☕ Coffee Cup (common)
- 🎮 Controller (uncommon)
- 🗡️ Sword (uncommon)
- 🪄 Magic Wand (rare)
- 💎 Crystal (legendary)

AURAS
- ✨ Sparkles (common)
- 💫 Stars (uncommon)
- 🔥 Flames (rare)
- ❄️ Snowflakes (rare)
- 🌈 Rainbow (legendary)
- ⚡ Lightning (legendary)

BACKGROUNDS
- 🏠 Cozy Room (free, default)
- 🌲 Forest (common)
- 🏖️ Beach (uncommon)
- 🌙 Night Sky (uncommon)
- 🏰 Castle (rare)
- 🌌 Galaxy (legendary)
```

---

## Phase 3: Daily Systems (Week 3)

### Tasks

1. **Daily login rewards**
   ```go
   GetDailyLoginStatus() (*DailyLogin, error)
   ClaimDailyReward() (int, error)  // Returns coins earned
   ```

   Reward schedule:
   ```
   Day 1: 10 coins
   Day 2: 15 coins
   Day 3: 25 coins
   Day 4: 40 coins
   Day 5: 60 coins
   Day 6: 85 coins
   Day 7: 120 coins + random uncommon item
   ```

2. **Daily challenges**
   ```go
   GenerateDailyChallenges(date time.Time) error
   GetDailyChallenges() ([]Challenge, error)
   UpdateChallengeProgress(challengeType string, delta int) error
   ClaimChallengeReward(challengeID int) error
   ```

   Challenge types:
   ```
   - complete_sprints: Complete N sprints (15-30c)
   - score_80_plus: Get 80%+ on a sprint (25c)
   - perfect_sprint: Get 100% on a sprint (50c)
   - review_items: Review N knowledge items (20c)
   - use_voice: Use voice mode (15c)
   - any_activity: Just log in and do something (10c)
   ```

3. **Weekly goals**
   ```go
   GenerateWeeklyGoals(weekStart time.Time) error
   GetWeeklyGoals() ([]Goal, error)
   UpdateGoalProgress(goalType string, delta int) error
   ClaimGoalReward(goalID int) error
   ```

   Goal types:
   ```
   - weekly_sprints: Complete 10 sprints (100c)
   - weekly_streak: Maintain 7-day streak (75c)
   - weekly_perfect: Get 3 perfect scores (150c)
   - weekly_mastery: Master 5 knowledge items (125c)
   Bonus for all 4: +200c
   ```

4. **CLI commands**
   ```bash
   kgatectl daily               # Claim daily + show challenges
   kgatectl challenges          # List daily challenges
   kgatectl goals               # List weekly goals
   ```

5. **Integration hooks**
   - Call `UpdateChallengeProgress` from grade command
   - Call `UpdateGoalProgress` from grade command
   - Auto-generate challenges at midnight or on first command

---

## Phase 4: Achievements (Week 4)

### Tasks

1. **Achievement checking**
   ```go
   CheckAchievements() ([]Achievement, error)  // Returns newly unlocked
   GetAllAchievements() ([]Achievement, error)
   GetUnlockedAchievements() ([]Achievement, error)
   GetAchievementProgress(id string) (int, int, error)  // current, target
   ```

2. **Achievement triggers**
   - Hook into sprint completion
   - Hook into streak updates
   - Hook into XP/level milestones
   - Hook into shop purchases

3. **Achievement definitions**

   ```
   LEARNING
   - first_sprint: Complete first sprint (25c)
   - sprint_10: Complete 10 sprints (50c)
   - sprint_50: Complete 50 sprints (150c)
   - sprint_100: Complete 100 sprints (300c)
   - perfect_1: First 100% score (50c)
   - perfect_10: 10 perfect scores (200c)
   - speed_demon: Sprint under 2 minutes (75c)

   STREAKS
   - streak_7: 7-day streak (50c)
   - streak_30: 30-day streak (200c)
   - streak_100: 100-day streak (500c)
   - streak_365: 365-day streak (2000c)

   COLLECTION
   - first_purchase: Buy first item (25c)
   - own_5_items: Own 5 items (75c)
   - own_20_items: Own 20 items (200c)
   - full_outfit: Equip all 3 slots (100c)
   - legendary_owner: Own a legendary item (150c)

   SECRET
   - night_owl: Sprint between 2-5 AM (100c)
   - early_bird: Sprint before 6 AM (100c)
   - comeback: Return after 30+ day break (75c)
   - dedicated: Sprint on a holiday (50c)
   ```

4. **CLI commands**
   ```bash
   kgatectl achievements        # List all with progress
   kgatectl achievements -u     # Unlocked only
   ```

5. **Notifications**
   - Show achievement popup on unlock
   - TTS announcement if voice enabled
   - Tray notification

---

## Phase 5: Tray Integration (Week 5)

### Tasks

1. **Update tray icon based on mood**
   ```go
   // internal/tray/tray.go
   UpdateTrayIcon(mood string)
   UpdateTrayTooltip(avatar *Avatar, wallet *Wallet)
   ```

2. **Tray menu additions**
   ```
   ─────────────────
   🐱 Whiskers (Happy)
   💰 1,250 coins
   🔥 14-day streak
   ─────────────────
   📊 Quick Stats
   🎯 Daily Challenges (2/3)
   🛒 Open Shop
   ─────────────────
   ⚙️ Settings
   ❌ Quit
   ```

3. **Mood icons for tray**
   ```
   Happy:   🐱😸 (bright)
   Content: 🐱😺 (normal)
   Neutral: 🐱😐 (muted)
   Sad:     🐱😿 (grey)
   Lonely:  🐱😾 (dark)
   ```

4. **Desktop notifications**
   - Achievement unlocked
   - Daily reward ready
   - Streak about to break (evening reminder)
   - Weekly goal completed

---

## Phase 6: XP Integration (Week 6)

### Tasks

1. **Mood-based XP multiplier**
   ```go
   func GetXPMultiplier(mood string) float64 {
       switch mood {
       case "happy":   return 1.15
       case "content": return 1.05
       default:        return 1.0
       }
   }
   ```

2. **Update grading to apply multiplier**
   ```go
   // In cmdGrade
   multiplier := gamification.GetXPMultiplier(avatar.Mood)
   adjustedXP := int(float64(result.XPEarned) * multiplier)
   ```

3. **Coin earning from sprints**
   ```go
   func CoinsFromSprint(passed bool, perfect bool, firstTime bool) int {
       coins := 0
       if passed {
           if firstTime {
               coins = 50
           } else {
               coins = 10
           }
           if perfect {
               coins += 25  // Bonus for 100%
           }
       }
       return coins
   }
   ```

4. **Streak coin bonus**
   ```go
   func StreakBonus(streakDays int) int {
       // 5 coins per streak day, cap at 50
       bonus := streakDays * 5
       if bonus > 50 {
           bonus = 50
       }
       return bonus
   }
   ```

---

## ASCII Art Assets

### Creatures

```
CAT
     /\_/\
    ( o.o )
     > ^ <

CAT WITH HAT (🎩)
      🎩
     /\_/\
    ( o.o )
     > ^ <

CAT WITH AURA (✨)
   ✨     ✨
     /\_/\
  ✨( o.o )✨
     > ^ <
   ✨     ✨
```

```
SLIME (Dragon Quest)
      ___
     /   \
    | o o |
     \ ▽ /
      \_/

SLIME WITH HAT
      🎓
      ___
     /   \
    | o o |
     \ ▽ /
```

```
OCTOPUS
     ___
    (o o)
    /| |\
   / | | \
  ~  ~ ~  ~

OCTOPUS WITH HAT
      👑
     ___
    (o o)
    /| |\
```

```
SNAIL
      __@
    _/  |
   (___ /

SNAIL WITH HAT
      🎩
      __@
    _/  |
   (___ /
```

### Moods (Cat example)

```
HAPPY 😸
     /\_/\
    ( ^.^ )
     > ^ <

CONTENT 😺
     /\_/\
    ( o.o )
     > ^ <

NEUTRAL 😐
     /\_/\
    ( -.- )
     > ^ <

SAD 😿
     /\_/\
    ( ;.; )
     > ^ <

LONELY 😾
     /\_/\
    ( >.< )
     > ^ <
```

---

## File Structure

```
internal/
├── gamification/
│   ├── avatar.go       # Avatar logic, mood calculation
│   ├── wallet.go       # Coin operations
│   ├── shop.go         # Shop catalog, purchasing
│   ├── inventory.go    # Owned items, equipping
│   ├── daily.go        # Login rewards, challenges
│   ├── weekly.go       # Weekly goals
│   ├── achievements.go # Achievement checking
│   ├── render.go       # ASCII art rendering
│   └── data/
│       ├── items.go    # Shop item definitions
│       ├── achievements.go  # Achievement definitions
│       └── ascii.go    # ASCII art templates
└── db/
    └── gamification.go # Database operations
```

---

## CLI Commands Summary

```bash
# Avatar
kgatectl avatar              # Show avatar + stats + mood
kgatectl avatar pick <type>  # Choose creature (cat/slime/octopus/snail)
kgatectl avatar name <name>  # Rename your companion

# Shop & Inventory
kgatectl shop                # Browse shop
kgatectl shop <category>     # Filter (hats/held/auras/backgrounds)
kgatectl shop buy <id>       # Purchase item
kgatectl inventory           # View owned items
kgatectl equip <id>          # Equip accessory
kgatectl unequip <slot>      # Remove from slot

# Daily Systems
kgatectl daily               # Claim daily reward
kgatectl challenges          # View daily challenges
kgatectl goals               # View weekly goals

# Achievements
kgatectl achievements        # List all + progress
kgatectl achievements -u     # Unlocked only

# Wallet
kgatectl wallet              # Show coin balance
```

---

## Migration Order

1. **007_gamification_core.sql**
   - avatar, wallet, equipped tables

2. **008_gamification_shop.sql**
   - shop_items, inventory tables
   - Seed items

3. **009_gamification_daily.sql**
   - daily_login, daily_challenges, weekly_goals

4. **010_gamification_achievements.sql**
   - achievements, unlocked_achievements
   - Seed achievements

---

## Testing Checklist

### Phase 1
- [ ] Avatar creation with creature type
- [ ] Mood calculation from last_active_date
- [ ] ASCII rendering for all 4 creatures
- [ ] Mood affects display

### Phase 2
- [ ] Shop lists items correctly
- [ ] Purchase deducts coins
- [ ] Can't buy if not enough coins
- [ ] Inventory tracks purchases
- [ ] Equip/unequip works
- [ ] Rendering shows equipped items

### Phase 3
- [ ] Daily login tracks consecutive days
- [ ] Day 7 resets to day 1
- [ ] Challenges generate correctly
- [ ] Challenge progress updates from sprints
- [ ] Weekly goals span Mon-Sun
- [ ] All 4 goals bonus works

### Phase 4
- [ ] Achievements unlock at thresholds
- [ ] Coins awarded on unlock
- [ ] Secret achievements hidden until unlocked
- [ ] Progress shows correctly

### Phase 5
- [ ] Tray icon reflects mood
- [ ] Tray tooltip shows stats
- [ ] Notifications fire on events

### Phase 6
- [ ] XP multiplier applies correctly
- [ ] Coins earned from sprints
- [ ] Streak bonus calculated right
