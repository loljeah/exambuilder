# Knowledge Gate — Gamification System Design

## Vision

A tamagotchi-style companion that grows with your learning. Your avatar (a cat/creature) evolves, gains accessories, and thrives when you learn consistently. Neglect your studies, and your companion gets sad. A shop system lets you spend earned currency on cosmetics, power-ups, and customization.

---

## Core Systems

### 1. Avatar System — "Study Buddy"

Your personal companion that reflects your learning journey.

#### Creature Types (Unlockable)
```
Default:     🐱 Whiskers (cat)
Level 10:    🦊 Rusty (fox)
Level 25:    🐲 Sparky (dragon)
Level 50:    🦉 Oracle (owl)
Level 100:   🐙 Cthulhu Jr (cosmic horror)
Secret:      🤖 Claude (AI companion) — unlock by passing 100 sprints
```

#### Evolution Stages
Each creature evolves through 5 stages based on total XP:

| Stage | XP Required | Appearance | Abilities |
|-------|-------------|------------|-----------|
| Egg | 0 | 🥚 | None — just hatched |
| Baby | 100 | Small, simple | Basic encouragement |
| Teen | 500 | Growing, playful | Hints available |
| Adult | 2000 | Full-sized, detailed | Power-ups unlocked |
| Elder | 10000 | Majestic, glowing | All abilities, mentor mode |

#### Mood System
Avatar mood changes based on recent activity:

| Mood | Trigger | Visual | Effect |
|------|---------|--------|--------|
| 😸 Ecstatic | 7+ day streak | Sparkles, dancing | +20% XP bonus |
| 😺 Happy | Active today | Smiling, alert | +10% XP bonus |
| 😐 Neutral | 1 day inactive | Resting | Normal XP |
| 😿 Sad | 2-3 days inactive | Droopy, grey | -10% XP |
| 😾 Angry | 4+ days inactive | Dark clouds | -25% XP, needs petting |
| 💀 Ghost | 14+ days inactive | Translucent | Must revive with streak |

#### Needs (Tamagotchi Style)
```
Hunger:     Feed by completing sprints
Happiness:  Pet by reviewing knowledge items
Energy:     Rest by taking breaks (pomodoro)
Hygiene:    Clean by fixing wrong answers (retakes)
```

#### Avatar Display (Terminal)
```
╭─────────────────────────────╮
│     ∧＿∧                    │
│    ( ･ω･)  Whiskers         │
│    |⊃ ⊂|   Level 12 Cat     │
│    |  |    😺 Happy          │
│    ∪ ∪     ████████░░ 80%   │
│                             │
│  🍖 Full  💕 Happy  ⚡ Alert │
╰─────────────────────────────╯
```

---

### 2. Currency System

#### Coins (Primary Currency)
- Earned from: sprints, streaks, achievements
- Used for: shop items, cosmetics, consumables
- Cannot be purchased with real money

| Action | Coins Earned |
|--------|-------------|
| Pass sprint (first time) | 50 |
| Pass sprint (retry) | 10 |
| Perfect sprint (100%) | 100 bonus |
| Daily streak bonus | 5 × streak_days |
| Weekly goal complete | 200 |
| Achievement unlock | varies |

#### Gems (Premium Currency)
- Earned from: milestones, special events, long streaks
- Used for: rare items, avatar unlocks, special effects
- Harder to earn, more valuable

| Action | Gems Earned |
|--------|-------------|
| 30-day streak | 50 |
| 100-day streak | 200 |
| Level milestone (every 10) | 25 |
| Complete all project sprints | 100 |
| Secret achievement | 10-50 |

---

### 3. Shop System

#### Categories

**Accessories** (Cosmetic)
```
Hats:
  🎩 Top Hat — 100 coins
  🎓 Grad Cap — 200 coins (unlock: pass 10 sprints)
  👒 Sun Hat — 150 coins
  🤠 Cowboy Hat — 300 coins
  👑 Crown — 500 gems (legendary)

Glasses:
  🤓 Nerd Glasses — 75 coins
  😎 Cool Shades — 150 coins
  🥽 Goggles — 200 coins
  🔮 Magic Monocle — 300 gems

Collars/Necklaces:
  📿 Beads — 50 coins
  🔔 Bell Collar — 100 coins
  💎 Diamond Collar — 400 gems
  ⚡ Lightning Amulet — 250 gems

Wings/Back:
  🦋 Butterfly Wings — 200 coins
  👼 Angel Wings — 500 gems
  🦇 Bat Wings — 300 coins
  🔥 Fire Wings — 750 gems (legendary)
```

**Environments** (Backgrounds)
```
🏠 Cozy Room — Free (default)
🌲 Forest Glade — 200 coins
🏖️ Beach Paradise — 300 coins
🌙 Night Sky — 250 coins
🚀 Space Station — 500 coins
🏰 Castle — 400 gems
🌈 Rainbow Realm — 600 gems
🕳️ The Void — 1000 gems (legendary)
```

**Consumables** (Power-ups)
```
🍖 Mega Treat — 50 coins
   Effect: Instant mood boost to Happy

☕ Focus Potion — 75 coins
   Effect: +25% XP for next 3 sprints

🛡️ Streak Shield — 200 coins
   Effect: Protect streak for 1 missed day

❄️ Freeze Token — 150 coins
   Effect: Pause streak decay for 3 days

💡 Hint Pack — 100 coins
   Effect: 5 free hints on hard questions

🎰 Lucky Charm — 300 coins
   Effect: Double coins for 1 hour

🎁 Mystery Box — 500 coins
   Effect: Random item (common to rare)
```

**Special Items**
```
🥚 Creature Egg — 1000 gems
   Effect: Unlock new creature type

🧬 Evolution Stone — 500 gems
   Effect: Instant evolution to next stage

📜 Name Change Scroll — 100 gems
   Effect: Rename your avatar

🎨 Color Palette — 200 gems
   Effect: Custom avatar colors

✨ Prestige Token — 2000 gems
   Effect: Reset level for permanent bonus
```

---

### 4. Achievement System

#### Categories

**Learning Achievements**
```
📚 First Steps — Complete your first sprint
🎯 Sharpshooter — Get 100% on a sprint
🔥 On Fire — 7-day streak
⚡ Lightning Fast — Complete sprint in under 2 minutes
🧠 Big Brain — Master 50 knowledge items
📖 Bookworm — Complete 100 sprints
🎓 Graduate — Complete all sprints in a project
🏆 Perfectionist — 100% on 10 sprints in a row
```

**Streak Achievements**
```
Week Warrior — 7-day streak
Month Master — 30-day streak
Quarter Queen — 90-day streak
Year Legend — 365-day streak
```

**Collection Achievements**
```
Fashionista — Own 10 accessories
Decorator — Own 5 environments
Hoarder — Earn 10,000 coins total
Gem Collector — Earn 1,000 gems total
Full Wardrobe — Own all hats
```

**Secret Achievements**
```
🌙 Night Owl — Complete sprint between 2-5 AM
🌅 Early Bird — Complete sprint before 6 AM
🎃 Spooky — Complete sprint on Halloween
🦃 Dedicated — Complete sprint on Thanksgiving
🎄 No Rest — Complete sprint on Christmas
🐛 Bug Hunter — Find and report a bug
🤖 AI Whisperer — Use voice mode 100 times
```

#### Achievement Display
```
╭─ Achievements ─────────────────────────╮
│ 🏆 Unlocked: 24/50                     │
│                                        │
│ Recent:                                │
│   ✨ On Fire — 7-day streak (today)    │
│   ✨ Sharpshooter — 100% sprint (2d)   │
│                                        │
│ Next unlock:                           │
│   🔒 Month Master — 23/30 days         │
│   ████████████████████░░░░ 77%         │
╰────────────────────────────────────────╯
```

---

### 5. Progression System

#### XP & Levels

```
Level = floor(sqrt(total_xp / 10))

Level 1:   0-10 XP
Level 2:   10-40 XP
Level 3:   40-90 XP
Level 5:   160-250 XP
Level 10:  810-1000 XP
Level 25:  6010-6250 XP
Level 50:  24010-25000 XP
Level 100: 99010-100000 XP
```

#### Level Rewards
```
Every level:     +10 coins
Every 5 levels:  +1 gem, unlock shop item
Every 10 levels: +25 gems, new title
Every 25 levels: New creature unlock
Level 100:       Prestige available
```

#### Prestige System
Reset to level 1 with permanent bonuses:

| Prestige | Bonus | Badge |
|----------|-------|-------|
| ⭐ | +5% XP permanently | Bronze star |
| ⭐⭐ | +10% XP, +5% coins | Silver star |
| ⭐⭐⭐ | +15% XP, +10% coins | Gold star |
| ⭐⭐⭐⭐ | +20% XP, +15% coins | Platinum star |
| ⭐⭐⭐⭐⭐ | +25% XP, +20% coins, exclusive creature | Diamond star |

---

### 6. Daily & Weekly Systems

#### Daily Login Rewards
```
Day 1: 10 coins
Day 2: 20 coins
Day 3: 30 coins
Day 4: 50 coins
Day 5: 75 coins
Day 6: 100 coins
Day 7: 150 coins + 5 gems + Mystery Box
```
*Cycle repeats, missing a day resets to Day 1*

#### Daily Challenges
```
Examples:
- Complete 2 sprints → 25 coins
- Get 80%+ on a sprint → 30 coins
- Review 5 knowledge items → 20 coins
- Use voice mode once → 15 coins
```

#### Weekly Goals (Customizable)
```
Default:
- Complete 10 sprints → 100 coins
- Maintain 7-day streak → 50 coins + 5 gems
- Master 5 knowledge items → 75 coins
- Get 3 perfect scores → 150 coins

Reward for all 4: 200 bonus coins + Mystery Box
```

#### Monthly Events
```
January:   "New Year, New Knowledge" — Double XP week
February:  "Love Learning" — Heart accessories in shop
March:     "Spring Cleaning" — Bonus for retaking failed sprints
April:     "April Fools" — Silly cosmetics, random effects
October:   "Spooktober" — Halloween items, ghost mode
December:  "Winter Wisdom" — Holiday items, gift exchanges
```

---

### 7. Social Features (Optional)

#### Profiles
```
╭─ ljsm's Profile ────────────────────────╮
│                                         │
│  🐱 Whiskers (Elder Cat)               │
│  ⭐⭐ Prestige 2                        │
│  Level 47 • 23,450 XP                  │
│                                         │
│  🔥 142-day streak                      │
│  🏆 38 achievements                     │
│  📚 312 sprints passed                  │
│                                         │
│  Title: Knowledge Keeper                │
│  Motto: "Learning never stops"          │
│                                         │
│  Badges: 🎓🔥⚡🧠🏆                      │
╰─────────────────────────────────────────╯
```

#### Leaderboards
```
Weekly Sprint Leaderboard:
1. 🥇 alice      — 47 sprints
2. 🥈 bob        — 42 sprints
3. 🥉 ljsm       — 38 sprints
4.    charlie    — 35 sprints
5.    dave       — 31 sprints
```

#### Challenges
- Challenge a friend to beat your sprint score
- Wager coins on the outcome
- Winner takes the pot

#### Gifting
- Send coins or items to friends
- Daily gift limit to prevent abuse
- Gift wrapping cosmetics

---

### 8. Terminal UI

#### Main Dashboard
```
╭─────────────────────────────────────────────────────────────────╮
│  KNOWLEDGE GATE                           💰 1,250  💎 45       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│    ∧＿∧       📊 Stats                   🎯 Daily Challenges    │
│   ( ･ω･)      Level 12 • 1,250 XP        ☑ Complete 2 sprints  │
│   |⊃🎩⊂|      🔥 14-day streak           ☐ Get 80%+ score      │
│   |  |        📚 47 sprints passed       ☐ Review 5 items      │
│   ∪ ∪                                                          │
│  Whiskers     Next: ████████░░ 80%       Reward: 75 coins      │
│  😺 Happy      250 XP to level 13                              │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  [S]prints  [R]eview  [P]rofile  [A]chievements  [SH]op        │
╰─────────────────────────────────────────────────────────────────╯
```

#### Shop UI
```
╭─ SHOP ──────────────────────────────────────────────────────────╮
│  💰 1,250 coins    💎 45 gems                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  🎩 ACCESSORIES          🏠 ENVIRONMENTS       🍖 CONSUMABLES   │
│                                                                 │
│  [1] 🎓 Grad Cap  200c   [5] 🌲 Forest  200c   [9] ☕ Focus 75c │
│  [2] 😎 Shades    150c   [6] 🏖️ Beach   300c   [10] 🛡️ Shield 200c│
│  [3] 🥽 Goggles   200c   [7] 🌙 Night   250c   [11] ❄️ Freeze 150c│
│  [4] 👑 Crown    500g    [8] 🏰 Castle  400g   [12] 🎁 Mystery 500c│
│                                                                 │
│  Featured: 🔥 Fire Wings — 750 gems (20% off this week!)       │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  Enter number to buy, [B]ack to exit                           │
╰─────────────────────────────────────────────────────────────────╯
```

#### Avatar Customization
```
╭─ AVATAR CUSTOMIZATION ──────────────────────────────────────────╮
│                                                                 │
│         Current Look:              Inventory:                   │
│                                                                 │
│            ∧＿∧                    🎩 Top Hat                   │
│           ( ･ω･)                   🎓 Grad Cap ✓               │
│           |⊃🎓⊂|                   😎 Cool Shades              │
│           |  |                     🥽 Goggles                   │
│           ∪ ∪                      📿 Beads                     │
│                                                                 │
│         Background:                                             │
│         🌲 Forest Glade                                        │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  [1-9] Equip item  [E]nvironment  [N]ame  [B]ack               │
╰─────────────────────────────────────────────────────────────────╯
```

---

## Database Schema Additions

```sql
-- Avatar and customization
CREATE TABLE avatars (
    id TEXT PRIMARY KEY,
    creature_type TEXT DEFAULT 'cat',
    name TEXT DEFAULT 'Whiskers',
    stage TEXT DEFAULT 'egg',
    mood TEXT DEFAULT 'neutral',
    hunger INTEGER DEFAULT 100,
    happiness INTEGER DEFAULT 100,
    energy INTEGER DEFAULT 100,
    total_xp INTEGER DEFAULT 0,
    equipped_hat TEXT,
    equipped_glasses TEXT,
    equipped_collar TEXT,
    equipped_wings TEXT,
    environment TEXT DEFAULT 'cozy_room',
    custom_colors TEXT, -- JSON
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Currency
CREATE TABLE wallet (
    id TEXT PRIMARY KEY,
    coins INTEGER DEFAULT 0,
    gems INTEGER DEFAULT 0,
    lifetime_coins INTEGER DEFAULT 0,
    lifetime_gems INTEGER DEFAULT 0
);

-- Inventory
CREATE TABLE inventory (
    id TEXT PRIMARY KEY,
    item_type TEXT NOT NULL, -- accessory, environment, consumable
    item_id TEXT NOT NULL,
    quantity INTEGER DEFAULT 1,
    acquired_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(item_type, item_id)
);

-- Shop items catalog
CREATE TABLE shop_items (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    item_type TEXT NOT NULL,
    price_coins INTEGER DEFAULT 0,
    price_gems INTEGER DEFAULT 0,
    unlock_requirement TEXT, -- JSON: {type: "level", value: 10}
    rarity TEXT DEFAULT 'common', -- common, uncommon, rare, legendary
    limited_time BOOLEAN DEFAULT FALSE,
    available_until DATETIME
);

-- Purchase history
CREATE TABLE purchases (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL,
    price_coins INTEGER,
    price_gems INTEGER,
    purchased_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Daily/weekly tracking
CREATE TABLE daily_rewards (
    id TEXT PRIMARY KEY,
    current_day INTEGER DEFAULT 1,
    last_claimed DATE,
    streak_days INTEGER DEFAULT 0
);

CREATE TABLE daily_challenges (
    id TEXT PRIMARY KEY,
    date DATE NOT NULL,
    challenge_type TEXT NOT NULL,
    target INTEGER NOT NULL,
    progress INTEGER DEFAULT 0,
    completed BOOLEAN DEFAULT FALSE,
    reward_coins INTEGER,
    reward_gems INTEGER,
    UNIQUE(date, challenge_type)
);

CREATE TABLE weekly_goals (
    id TEXT PRIMARY KEY,
    week_start DATE NOT NULL,
    goal_type TEXT NOT NULL,
    target INTEGER NOT NULL,
    progress INTEGER DEFAULT 0,
    completed BOOLEAN DEFAULT FALSE,
    UNIQUE(week_start, goal_type)
);

-- Prestige
CREATE TABLE prestige (
    id TEXT PRIMARY KEY,
    level INTEGER DEFAULT 0,
    total_prestiges INTEGER DEFAULT 0,
    xp_bonus_percent INTEGER DEFAULT 0,
    coin_bonus_percent INTEGER DEFAULT 0,
    last_prestige_at DATETIME
);
```

---

## CLI Commands

```bash
# Avatar
kgatectl avatar                    # Show avatar status
kgatectl avatar name "Fluffy"      # Rename avatar
kgatectl avatar pet                # Pet your avatar (happiness boost)
kgatectl avatar feed               # Use a treat (requires item)

# Shop
kgatectl shop                      # Browse shop
kgatectl shop buy <item_id>        # Purchase item
kgatectl shop featured             # Show featured/sale items

# Inventory
kgatectl inventory                 # List owned items
kgatectl equip <item_id>           # Equip accessory
kgatectl unequip <slot>            # Remove accessory

# Currency
kgatectl wallet                    # Show coins/gems balance
kgatectl daily                     # Claim daily reward

# Challenges
kgatectl challenges                # Show daily challenges
kgatectl goals                     # Show weekly goals

# Achievements
kgatectl achievements              # List all achievements
kgatectl achievements unlocked     # Show unlocked only
kgatectl achievements progress     # Show in-progress

# Social (optional)
kgatectl profile                   # Show your profile
kgatectl leaderboard               # Weekly leaderboard
kgatectl challenge <user> <sprint> # Challenge someone
```

---

## Implementation Phases

### Phase 1: Core Avatar System
- Avatar table and basic CRUD
- Mood system based on activity
- Simple ASCII display
- XP-based evolution stages

### Phase 2: Currency & Shop
- Wallet system
- Shop catalog with items
- Purchase flow
- Inventory management

### Phase 3: Daily Systems
- Daily login rewards
- Daily challenges generation
- Weekly goals
- Streak bonuses integration

### Phase 4: Achievements
- Achievement definitions
- Unlock triggers
- Progress tracking
- Notification system

### Phase 5: Customization
- Accessory equipping
- Environment selection
- Avatar rendering with items
- Custom colors

### Phase 6: Advanced
- Prestige system
- Mystery boxes
- Seasonal events
- Social features (if desired)

---

## Config Options

```toml
[gamification]
enabled = true
avatar_enabled = true
shop_enabled = true
daily_rewards_enabled = true
achievements_enabled = true

[avatar]
default_creature = "cat"
mood_decay_hours = 24          # Hours until mood drops
hunger_decay_hours = 48        # Hours until hungry
evolution_xp_multiplier = 1.0  # Adjust evolution speed

[currency]
coin_multiplier = 1.0          # Adjust coin earning rate
gem_multiplier = 1.0           # Adjust gem earning rate
daily_reward_reset_on_miss = true

[achievements]
notification_enabled = true
secret_hints_enabled = false   # Show hints for secret achievements
```

---

## ASCII Art Assets

```
# Cat stages
Egg:   🥚
Baby:  /ᐠ - ˕ -マ
Teen:  /ᐠ｡ꞈ｡ᐟ\
Adult: ∧＿∧
       ( ･ω･)
Elder: ∧＿∧✨
       ( ･ω･)
       /|  |ヽ

# Moods
Happy:    (･ω･)
Ecstatic: (＾ω＾)
Neutral:  (･_･)
Sad:      (･ω･`)
Angry:    (`ω´)
Sleeping: (￣ω￣) zzZ

# With accessories
Hat:      🎩
         ∧＿∧
        ( ･ω･)

Glasses: ∧＿∧
        ( ･ω･)🤓

Crown:    👑
         ∧＿∧
        ( ･ω･)
```
