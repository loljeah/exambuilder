export namespace main {
	
	export class AchievementData {
	    id: string;
	    name: string;
	    description: string;
	    category: string;
	    icon: string;
	    reward_coins: number;
	    secret: boolean;
	    unlocked: boolean;
	    unlocked_at: string;
	
	    static createFrom(source: any = {}) {
	        return new AchievementData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.description = source["description"];
	        this.category = source["category"];
	        this.icon = source["icon"];
	        this.reward_coins = source["reward_coins"];
	        this.secret = source["secret"];
	        this.unlocked = source["unlocked"];
	        this.unlocked_at = source["unlocked_at"];
	    }
	}
	export class AvatarData {
	    creature_type: string;
	    name: string;
	    mood: string;
	    xp_multiplier: number;
	
	    static createFrom(source: any = {}) {
	        return new AvatarData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.creature_type = source["creature_type"];
	        this.name = source["name"];
	        this.mood = source["mood"];
	        this.xp_multiplier = source["xp_multiplier"];
	    }
	}
	export class ChallengeData {
	    id: number;
	    description: string;
	    target: number;
	    progress: number;
	    reward_coins: number;
	    completed: boolean;
	    claimed: boolean;
	
	    static createFrom(source: any = {}) {
	        return new ChallengeData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.description = source["description"];
	        this.target = source["target"];
	        this.progress = source["progress"];
	        this.reward_coins = source["reward_coins"];
	        this.completed = source["completed"];
	        this.claimed = source["claimed"];
	    }
	}
	export class DailyLoginData {
	    current_day: number;
	    total_claims: number;
	    can_claim: boolean;
	
	    static createFrom(source: any = {}) {
	        return new DailyLoginData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.current_day = source["current_day"];
	        this.total_claims = source["total_claims"];
	        this.can_claim = source["can_claim"];
	    }
	}
	export class DailyStatsData {
	    date: string;
	    sessions_count: number;
	    sprints_attempted: number;
	    sprints_passed: number;
	    questions_answered: number;
	    questions_correct: number;
	    xp_earned: number;
	
	    static createFrom(source: any = {}) {
	        return new DailyStatsData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.date = source["date"];
	        this.sessions_count = source["sessions_count"];
	        this.sprints_attempted = source["sprints_attempted"];
	        this.sprints_passed = source["sprints_passed"];
	        this.questions_answered = source["questions_answered"];
	        this.questions_correct = source["questions_correct"];
	        this.xp_earned = source["xp_earned"];
	    }
	}
	export class ProjectData {
	    id: string;
	    name: string;
	    path: string;
	
	    static createFrom(source: any = {}) {
	        return new ProjectData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.path = source["path"];
	    }
	}
	export class WeeklyGoalData {
	    id: number;
	    description: string;
	    target: number;
	    progress: number;
	    reward_coins: number;
	    completed: boolean;
	    claimed: boolean;
	
	    static createFrom(source: any = {}) {
	        return new WeeklyGoalData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.description = source["description"];
	        this.target = source["target"];
	        this.progress = source["progress"];
	        this.reward_coins = source["reward_coins"];
	        this.completed = source["completed"];
	        this.claimed = source["claimed"];
	    }
	}
	export class WalletData {
	    coins: number;
	    lifetime_coins: number;
	
	    static createFrom(source: any = {}) {
	        return new WalletData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.coins = source["coins"];
	        this.lifetime_coins = source["lifetime_coins"];
	    }
	}
	export class ProfileData {
	    level: number;
	    total_xp: number;
	    current_streak: number;
	    best_streak: number;
	    sprints_passed: number;
	
	    static createFrom(source: any = {}) {
	        return new ProfileData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.level = source["level"];
	        this.total_xp = source["total_xp"];
	        this.current_streak = source["current_streak"];
	        this.best_streak = source["best_streak"];
	        this.sprints_passed = source["sprints_passed"];
	    }
	}
	export class DashboardData {
	    profile: ProfileData;
	    avatar: AvatarData;
	    wallet: WalletData;
	    daily_login: DailyLoginData;
	    challenges: ChallengeData[];
	    weekly_goals: WeeklyGoalData[];
	    review_due: number;
	    active_project?: ProjectData;
	    pending_sprints: number;
	
	    static createFrom(source: any = {}) {
	        return new DashboardData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.profile = this.convertValues(source["profile"], ProfileData);
	        this.avatar = this.convertValues(source["avatar"], AvatarData);
	        this.wallet = this.convertValues(source["wallet"], WalletData);
	        this.daily_login = this.convertValues(source["daily_login"], DailyLoginData);
	        this.challenges = this.convertValues(source["challenges"], ChallengeData);
	        this.weekly_goals = this.convertValues(source["weekly_goals"], WeeklyGoalData);
	        this.review_due = source["review_due"];
	        this.active_project = this.convertValues(source["active_project"], ProjectData);
	        this.pending_sprints = source["pending_sprints"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class DomainAchievementData {
	    id: string;
	    name: string;
	    description: string;
	    icon: string;
	    xp_reward: number;
	    unlocked: boolean;
	    unlocked_at?: string;
	
	    static createFrom(source: any = {}) {
	        return new DomainAchievementData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.description = source["description"];
	        this.icon = source["icon"];
	        this.xp_reward = source["xp_reward"];
	        this.unlocked = source["unlocked"];
	        this.unlocked_at = source["unlocked_at"];
	    }
	}
	export class DomainData {
	    id: string;
	    domain_id: string;
	    name: string;
	    description: string;
	    color: string;
	    icon: string;
	    total_xp: number;
	    earned_xp: number;
	    level: number;
	    level_title: string;
	    next_level_xp: number;
	    sprints_total: number;
	    sprints_passed: number;
	    sprints_perfect: number;
	    progress_pct: number;
	
	    static createFrom(source: any = {}) {
	        return new DomainData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.domain_id = source["domain_id"];
	        this.name = source["name"];
	        this.description = source["description"];
	        this.color = source["color"];
	        this.icon = source["icon"];
	        this.total_xp = source["total_xp"];
	        this.earned_xp = source["earned_xp"];
	        this.level = source["level"];
	        this.level_title = source["level_title"];
	        this.next_level_xp = source["next_level_xp"];
	        this.sprints_total = source["sprints_total"];
	        this.sprints_passed = source["sprints_passed"];
	        this.sprints_perfect = source["sprints_perfect"];
	        this.progress_pct = source["progress_pct"];
	    }
	}
	export class GenerationGateData {
	    domain_id: string;
	    domain_level: number;
	    can_sprint: boolean;
	    can_custom: boolean;
	    can_exam: boolean;
	    can_challenge: boolean;
	    first_free_used: boolean;
	    sprint_cost: number;
	    custom_cost: number;
	    exam_cost: number;
	    challenge_cost: number;
	
	    static createFrom(source: any = {}) {
	        return new GenerationGateData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.domain_id = source["domain_id"];
	        this.domain_level = source["domain_level"];
	        this.can_sprint = source["can_sprint"];
	        this.can_custom = source["can_custom"];
	        this.can_exam = source["can_exam"];
	        this.can_challenge = source["can_challenge"];
	        this.first_free_used = source["first_free_used"];
	        this.sprint_cost = source["sprint_cost"];
	        this.custom_cost = source["custom_cost"];
	        this.exam_cost = source["exam_cost"];
	        this.challenge_cost = source["challenge_cost"];
	    }
	}
	export class GenerationResultData {
	    generation_id: number;
	    sprint_ids: number[];
	    coins_spent: number;
	    status: string;
	
	    static createFrom(source: any = {}) {
	        return new GenerationResultData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.generation_id = source["generation_id"];
	        this.sprint_ids = source["sprint_ids"];
	        this.coins_spent = source["coins_spent"];
	        this.status = source["status"];
	    }
	}
	export class HintPackData {
	    tier: string;
	    tokens: number;
	    cost: number;
	
	    static createFrom(source: any = {}) {
	        return new HintPackData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.tier = source["tier"];
	        this.tokens = source["tokens"];
	        this.cost = source["cost"];
	    }
	}
	export class HintTokenData {
	    tokens: number;
	    lifetime_tokens: number;
	
	    static createFrom(source: any = {}) {
	        return new HintTokenData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.tokens = source["tokens"];
	        this.lifetime_tokens = source["lifetime_tokens"];
	    }
	}
	export class KnowledgeQuestionData {
	    sprint_number: number;
	    sprint_topic: string;
	    question_num: number;
	    tier: string;
	    difficulty: number;
	    xp: number;
	    text: string;
	    code: string;
	    options: string[];
	    correct_idx: number;
	    domain_id: string;
	    domain_name: string;
	    hint: string;
	    explanation: string;
	    times_answered: number;
	    times_correct: number;
	    last_answered?: string;
	    mastered: boolean;
	
	    static createFrom(source: any = {}) {
	        return new KnowledgeQuestionData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.sprint_number = source["sprint_number"];
	        this.sprint_topic = source["sprint_topic"];
	        this.question_num = source["question_num"];
	        this.tier = source["tier"];
	        this.difficulty = source["difficulty"];
	        this.xp = source["xp"];
	        this.text = source["text"];
	        this.code = source["code"];
	        this.options = source["options"];
	        this.correct_idx = source["correct_idx"];
	        this.domain_id = source["domain_id"];
	        this.domain_name = source["domain_name"];
	        this.hint = source["hint"];
	        this.explanation = source["explanation"];
	        this.times_answered = source["times_answered"];
	        this.times_correct = source["times_correct"];
	        this.last_answered = source["last_answered"];
	        this.mastered = source["mastered"];
	    }
	}
	export class OllamaConfigData {
	    base_url: string;
	    model: string;
	    timeout_seconds: number;
	    max_retries: number;
	
	    static createFrom(source: any = {}) {
	        return new OllamaConfigData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.base_url = source["base_url"];
	        this.model = source["model"];
	        this.timeout_seconds = source["timeout_seconds"];
	        this.max_retries = source["max_retries"];
	    }
	}
	
	
	export class PullProgressData {
	    active: boolean;
	    model: string;
	    status: string;
	    percent: number;
	    error?: string;
	
	    static createFrom(source: any = {}) {
	        return new PullProgressData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.active = source["active"];
	        this.model = source["model"];
	        this.status = source["status"];
	        this.percent = source["percent"];
	        this.error = source["error"];
	    }
	}
	export class QuestionData {
	    number: number;
	    tier: string;
	    stars: number;
	    xp: number;
	    text: string;
	    code: string;
	    options: string[];
	    correct_idx: number;
	
	    static createFrom(source: any = {}) {
	        return new QuestionData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.number = source["number"];
	        this.tier = source["tier"];
	        this.stars = source["stars"];
	        this.xp = source["xp"];
	        this.text = source["text"];
	        this.code = source["code"];
	        this.options = source["options"];
	        this.correct_idx = source["correct_idx"];
	    }
	}
	export class QuestionResultData {
	    question_num: number;
	    correct: boolean;
	    user_answer: string;
	    right_answer: string;
	    xp_earned: number;
	
	    static createFrom(source: any = {}) {
	        return new QuestionResultData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.question_num = source["question_num"];
	        this.correct = source["correct"];
	        this.user_answer = source["user_answer"];
	        this.right_answer = source["right_answer"];
	        this.xp_earned = source["xp_earned"];
	    }
	}
	export class SprintData {
	    id: number;
	    sprint_number: number;
	    topic: string;
	    status: string;
	    best_score: number;
	    attempts: number;
	    xp_available: number;
	    xp_earned: number;
	    domain_id: string;
	
	    static createFrom(source: any = {}) {
	        return new SprintData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.sprint_number = source["sprint_number"];
	        this.topic = source["topic"];
	        this.status = source["status"];
	        this.best_score = source["best_score"];
	        this.attempts = source["attempts"];
	        this.xp_available = source["xp_available"];
	        this.xp_earned = source["xp_earned"];
	        this.domain_id = source["domain_id"];
	    }
	}
	export class UnlockedAchievementData {
	    id: string;
	    name: string;
	    icon: string;
	    xp_reward: number;
	
	    static createFrom(source: any = {}) {
	        return new UnlockedAchievementData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.icon = source["icon"];
	        this.xp_reward = source["xp_reward"];
	    }
	}
	export class SprintResultData {
	    sprint_num: number;
	    topic: string;
	    passed: boolean;
	    score_percent: number;
	    correct_count: number;
	    total_questions: number;
	    xp_earned: number;
	    xp_available: number;
	    attempt_number: number;
	    coins_earned: number;
	    question_results: QuestionResultData[];
	    domain_level_up: boolean;
	    domain_new_level: number;
	    domain_new_title: string;
	    domain_name: string;
	    unlocked_achievements: UnlockedAchievementData[];
	
	    static createFrom(source: any = {}) {
	        return new SprintResultData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.sprint_num = source["sprint_num"];
	        this.topic = source["topic"];
	        this.passed = source["passed"];
	        this.score_percent = source["score_percent"];
	        this.correct_count = source["correct_count"];
	        this.total_questions = source["total_questions"];
	        this.xp_earned = source["xp_earned"];
	        this.xp_available = source["xp_available"];
	        this.attempt_number = source["attempt_number"];
	        this.coins_earned = source["coins_earned"];
	        this.question_results = this.convertValues(source["question_results"], QuestionResultData);
	        this.domain_level_up = source["domain_level_up"];
	        this.domain_new_level = source["domain_new_level"];
	        this.domain_new_title = source["domain_new_title"];
	        this.domain_name = source["domain_name"];
	        this.unlocked_achievements = this.convertValues(source["unlocked_achievements"], UnlockedAchievementData);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class SystemInfoData {
	    gpu_type: string;
	    gpu_name: string;
	    vram_gb: number;
	    ram_gb: number;
	    rec_model: string;
	    rec_size: string;
	    rec_reason: string;
	
	    static createFrom(source: any = {}) {
	        return new SystemInfoData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.gpu_type = source["gpu_type"];
	        this.gpu_name = source["gpu_name"];
	        this.vram_gb = source["vram_gb"];
	        this.ram_gb = source["ram_gb"];
	        this.rec_model = source["rec_model"];
	        this.rec_size = source["rec_size"];
	        this.rec_reason = source["rec_reason"];
	    }
	}
	export class TestResultData {
	    reachable: boolean;
	    model_loaded: boolean;
	    generate_ok: boolean;
	    response_time_ms: number;
	    error?: string;
	
	    static createFrom(source: any = {}) {
	        return new TestResultData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.reachable = source["reachable"];
	        this.model_loaded = source["model_loaded"];
	        this.generate_ok = source["generate_ok"];
	        this.response_time_ms = source["response_time_ms"];
	        this.error = source["error"];
	    }
	}
	
	

}

