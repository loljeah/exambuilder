# Exam Data Structure Specification

Version: 2.1
Last Updated: 2026-03-23

## Design Goals

1. **Parseable in single pass** - No backtracking, no multi-regex patterns
2. **Voice-first** - Every element readable aloud without modification
3. **Self-contained** - Hints/explanations inline, not in separate section
4. **Strict format** - No ambiguity, validation possible during parse
5. **ADHD-optimized** - Sprint chunking, easy-first progression, visual markers
6. **Domain-aware** - Knowledge domains and subdomains for stats tracking
7. **Auto-gamification** - Domain-specific achievements and leveling

---

## File Format: `exam_<project>.toml`

TOML chosen over Markdown for:
- Native arrays and tables (no regex needed)
- Clear key-value structure
- Comments support
- Standard parsers in all languages

### Complete Example

```toml
[meta]
project = "exambuilder"
version = "2.1"
generated = "2026-03-23"
voice_ready = true
pass_threshold = 60
total_xp = 150
content_type = "code"  # code | medical | legal | scientific | technical | study | other

# Knowledge Domain Map - auto-generated, used for stats and achievements
[[meta.domain]]
id = "architecture"
name = "Software Architecture"
description = "System design, patterns, and structure"
color = "#3B82F6"
icon = "🏗️"
sprints = [1, 2]  # Which sprints cover this domain
total_xp = 60

[[meta.domain]]
id = "security"
name = "Security & Hardening"
description = "Authentication, authorization, attack surface"
color = "#EF4444"
icon = "🔒"
sprints = [5]
total_xp = 30

[[meta.domain]]
id = "data"
name = "Data Layer"
description = "Database, migrations, queries"
color = "#10B981"
icon = "🗄️"
sprints = [3, 4]
total_xp = 60

# Auto-generated achievements for this project's domains
[[meta.achievement]]
id = "architecture_novice"
domain = "architecture"
name = "Blueprint Reader"
description = "Pass your first Architecture sprint"
condition = "domain_sprints_passed >= 1"
xp_reward = 25
icon = "📐"

[[meta.achievement]]
id = "architecture_master"
domain = "architecture"
name = "System Architect"
description = "Master all Architecture sprints with 100%"
condition = "domain_sprints_perfect == domain_sprints_total"
xp_reward = 100
icon = "🏛️"

[[meta.achievement]]
id = "security_first"
domain = "security"
name = "Security Aware"
description = "Pass the Security sprint"
condition = "domain_sprints_passed >= 1"
xp_reward = 30
icon = "🛡️"

# Domain level thresholds (XP needed per level in this domain)
[[meta.domain_levels]]
domain = "architecture"
levels = [0, 30, 80, 150, 300, 500]  # XP thresholds for levels 1-6
titles = ["Novice", "Apprentice", "Practitioner", "Expert", "Master", "Architect"]

[[sprint]]
number = 1
topic = "Rust Workspace & CLI"
domain = "architecture"           # Links to meta.domain.id
subdomain = "project_structure"   # Optional: more specific categorization
target_minutes = 3
voice_compatible = true

[[sprint.question]]
number = 1
tier = "RECALL"           # RECALL | COMPREHENSION | APPLICATION | ANALYSIS
difficulty = 1            # 1=Easy, 2=Medium, 3=Hard
xp = 10
text = "In this project's Cargo.toml, what is kgate-core responsible for?"
code = ""                 # Optional: code snippet (max 8 lines)
options = [
    "The command-line interface and terminal UI",
    "The core library: parser, grader, database, and models",
    "Voice synthesis and audio playback",
    "The nix build system and packaging"
]
answer = 1                # 0-indexed: B = index 1
hint = "Look at the crate names — one is a library, one is a binary"
explanation = "kgate-core contains parser, grader, database, models. kgate is the CLI binary that imports kgate-core."

[[sprint.question]]
number = 2
tier = "COMPREHENSION"
difficulty = 2
xp = 10
text = "The CLI uses clap with the derive pattern. Why is command: Option<Commands> rather than required?"
code = ""
options = [
    "To allow optional logging flags",
    "So running kgate with no subcommand picks a random unpassed sprint",
    "Because clap requires all fields to be optional",
    "To support piped input from other programs"
]
answer = 1
hint = "Think about the default behavior when no command is given"
explanation = "Option<Commands> means no subcommand = None, triggering the random sprint picker for quick practice."

[[sprint.question]]
number = 3
tier = "APPLICATION"
difficulty = 3
xp = 10
text = "What Rust pattern enables nested subcommands like 'kgate take exam 18 sprint 1 --voice'?"
code = """
enum TakeCommands {
    Exam {
        number: usize,
        #[command(subcommand)]
        action: TakeExamAction,
    },
}
"""
options = [
    "Enum variants with named fields containing another #[command(subcommand)] enum",
    "Trait objects stored in a HashMap",
    "Recursive generic type parameters",
    "A custom derive macro that flattens the command tree"
]
answer = 0
hint = "Notice the #[command(subcommand)] attribute"
explanation = "Clap's derive macro supports nesting by placing #[command(subcommand)] on a field whose type is another enum with subcommands."

# Sprint 2, 3, 4, 5... follow same structure
```

---

## Data Types

### Meta Section

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `project` | string | yes | Project name (matches folder) |
| `version` | string | yes | Format version ("2.1") |
| `generated` | date | yes | ISO 8601 date |
| `voice_ready` | bool | yes | All questions TTS-compatible |
| `pass_threshold` | int | yes | Percent to pass (60) |
| `total_xp` | int | yes | Sum of all question XP |
| `content_type` | string | yes | code, medical, legal, scientific, technical, study, other |
| `domain` | array | yes | Knowledge domain definitions (see below) |
| `achievement` | array | no | Auto-generated domain achievements |
| `domain_levels` | array | no | XP thresholds per domain |

### Domain Section (meta.domain)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier (snake_case) |
| `name` | string | yes | Display name |
| `description` | string | yes | Brief description |
| `color` | string | yes | Hex color for UI |
| `icon` | string | yes | Emoji icon |
| `sprints` | array[int] | yes | Sprint numbers covering this domain |
| `total_xp` | int | yes | Total XP available in this domain |

### Achievement Section (meta.achievement)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier |
| `domain` | string | yes | Domain ID this achievement belongs to |
| `name` | string | yes | Achievement name |
| `description` | string | yes | How to earn it |
| `condition` | string | yes | Unlock condition expression |
| `xp_reward` | int | yes | XP bonus when unlocked |
| `icon` | string | yes | Emoji icon |

### Domain Levels Section (meta.domain_levels)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `domain` | string | yes | Domain ID |
| `levels` | array[int] | yes | XP thresholds for each level |
| `titles` | array[string] | yes | Title names for each level |

### Sprint Section

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `number` | int | yes | Sprint number (1-based) |
| `topic` | string | yes | Sprint topic (max 40 chars) |
| `domain` | string | yes | Domain ID (references meta.domain.id) |
| `subdomain` | string | no | More specific categorization |
| `target_minutes` | int | yes | Target completion time |
| `voice_compatible` | bool | yes | Voice mode flag |

### Question Section

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `number` | int | yes | Question number within sprint (1-based) |
| `tier` | string | yes | RECALL, COMPREHENSION, APPLICATION, ANALYSIS |
| `difficulty` | int | yes | 1=Easy, 2=Medium, 3=Hard |
| `xp` | int | yes | XP value (10-20) |
| `text` | string | yes | Question stem (voice-readable) |
| `code` | string | no | Code snippet (max 8 lines, empty if none) |
| `options` | array[4] | yes | Exactly 4 options (no "all of the above") |
| `answer` | int | yes | Correct option index (0-3) |
| `hint` | string | yes | One-line hint (shown on 1st fail) |
| `explanation` | string | yes | Full explanation (shown on 2nd fail) |

---

## Validation Rules

### Sprint Level
- 2-5 questions per sprint
- Questions ordered Easy → Hard (difficulty 1, 2, 3)
- Q1 must be difficulty 1 (easy win)
- Total sprint XP = sum of question XP

### Question Level
- Text must not contain "all of the above" or "none of the above"
- Text max 200 chars for voice readability
- Code max 8 lines (counted by `\n`)
- Options array exactly length 4
- Answer index 0-3
- Hint max 100 chars
- Explanation max 300 chars
- No config lookups (ports, VLANs, IDs)

### Tier Rules
| Tier | Typical Difficulty | Tests |
|------|-------------------|-------|
| RECALL | 1 | Direct fact retrieval |
| COMPREHENSION | 1-2 | Understanding why |
| APPLICATION | 2-3 | Using knowledge in new context |
| ANALYSIS | 3 | Breaking down, comparing |

---

## Go Struct Mapping

```go
type ExamFile struct {
    Meta    ExamMeta  `toml:"meta"`
    Sprints []Sprint  `toml:"sprint"`
}

type ExamMeta struct {
    Project       string            `toml:"project"`
    Version       string            `toml:"version"`
    Generated     string            `toml:"generated"`
    VoiceReady    bool              `toml:"voice_ready"`
    PassThreshold int               `toml:"pass_threshold"`
    TotalXP       int               `toml:"total_xp"`
    ContentType   string            `toml:"content_type"`
    Domains       []Domain          `toml:"domain"`
    Achievements  []DomainAchieve   `toml:"achievement"`
    DomainLevels  []DomainLevelDef  `toml:"domain_levels"`
}

type Domain struct {
    ID          string `toml:"id"`
    Name        string `toml:"name"`
    Description string `toml:"description"`
    Color       string `toml:"color"`
    Icon        string `toml:"icon"`
    Sprints     []int  `toml:"sprints"`
    TotalXP     int    `toml:"total_xp"`
}

type DomainAchievement struct {
    ID          string `toml:"id"`
    Domain      string `toml:"domain"`
    Name        string `toml:"name"`
    Description string `toml:"description"`
    Condition   string `toml:"condition"`
    XPReward    int    `toml:"xp_reward"`
    Icon        string `toml:"icon"`
}

type DomainLevelDef struct {
    Domain string   `toml:"domain"`
    Levels []int    `toml:"levels"`
    Titles []string `toml:"titles"`
}

type Sprint struct {
    Number          int        `toml:"number"`
    Topic           string     `toml:"topic"`
    Domain          string     `toml:"domain"`
    Subdomain       string     `toml:"subdomain"`
    TargetMinutes   int        `toml:"target_minutes"`
    VoiceCompatible bool       `toml:"voice_compatible"`
    Questions       []Question `toml:"question"`
}

type Question struct {
    Number      int      `toml:"number"`
    Tier        string   `toml:"tier"`
    Difficulty  int      `toml:"difficulty"`
    XP          int      `toml:"xp"`
    Text        string   `toml:"text"`
    Code        string   `toml:"code"`
    Options     []string `toml:"options"`
    Answer      int      `toml:"answer"`
    Hint        string   `toml:"hint"`
    Explanation string   `toml:"explanation"`
}
```

---

## Parser Implementation

```go
import "github.com/BurntSushi/toml"

func ParseExamFile(path string) (*ExamFile, error) {
    var exam ExamFile
    if _, err := toml.DecodeFile(path, &exam); err != nil {
        return nil, err
    }
    if err := validate(&exam); err != nil {
        return nil, err
    }
    return &exam, nil
}

func validate(e *ExamFile) error {
    for _, s := range e.Sprints {
        if len(s.Questions) < 2 || len(s.Questions) > 5 {
            return fmt.Errorf("sprint %d: must have 2-5 questions", s.Number)
        }
        if s.Questions[0].Difficulty != 1 {
            return fmt.Errorf("sprint %d: Q1 must be difficulty 1", s.Number)
        }
        for _, q := range s.Questions {
            if len(q.Options) != 4 {
                return fmt.Errorf("sprint %d Q%d: must have exactly 4 options", s.Number, q.Number)
            }
            if q.Answer < 0 || q.Answer > 3 {
                return fmt.Errorf("sprint %d Q%d: answer must be 0-3", s.Number, q.Number)
            }
            if strings.Count(q.Code, "\n") > 8 {
                return fmt.Errorf("sprint %d Q%d: code exceeds 8 lines", s.Number, q.Number)
            }
        }
    }
    return nil
}
```

---

## Migration from Markdown (v1.0)

For existing `exam_*.md` files:

```bash
# Convert markdown to TOML
kgatectl exam convert exam_project.md --output exam_project.toml

# Validate TOML file
kgatectl exam validate exam_project.toml
```

The converter will:
1. Parse existing markdown with legacy parser
2. Extract all questions, answers, hints
3. Output strict TOML format
4. Report validation errors

---

## Voice Mode Integration

TOML fields map directly to speech:

```
"Question {number}. {text}"
[pause if code] "See code snippet below."
"Option A: {options[0]}. Option B: {options[1]}. Option C: {options[2]}. Option D: {options[3]}."
"Your answer?"
```

After answer:
- Correct: "Correct! Plus {xp} XP."
- Wrong (attempt 1): "Incorrect. Hint: {hint}"
- Wrong (attempt 2+): "Incorrect. {explanation}"

---

## File Locations

```
project-root/
├── exam_<project>.toml     # New format (preferred)
├── exam_<project>.md       # Legacy format (deprecated)
└── .claude/
    └── skills/
        └── teachANDexam/
            └── SKILL.md    # Generation rules
```

---

## Benefits Over Markdown

| Aspect | Markdown v1.0 | TOML v2.0 |
|--------|---------------|-----------|
| Parsing | 5 regex patterns | Native TOML decode |
| Validation | Post-parse, partial | Built-in, complete |
| Hints/Explanations | Separate section | Inline with question |
| Answer format | Multiple legacy formats | Single integer |
| Multi-line text | Regex breaks | Native support |
| Code blocks | Regex extraction | Simple string field |
| Error messages | Line number guessing | TOML parser precision |
| Generation | Template + regex escape | Struct → Marshal |
