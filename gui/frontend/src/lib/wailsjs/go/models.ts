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
	export class EquippedData {
	    hat_id: string;
	    held_id: string;
	    aura_id: string;
	    background_id: string;
	
	    static createFrom(source: any = {}) {
	        return new EquippedData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.hat_id = source["hat_id"];
	        this.held_id = source["held_id"];
	        this.aura_id = source["aura_id"];
	        this.background_id = source["background_id"];
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
	export class ShopItemData {
	    id: string;
	    name: string;
	    description: string;
	    slot: string;
	    price: number;
	    rarity: string;
	    unlock_level: number;
	    owned: boolean;
	
	    static createFrom(source: any = {}) {
	        return new ShopItemData(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.description = source["description"];
	        this.slot = source["slot"];
	        this.price = source["price"];
	        this.rarity = source["rarity"];
	        this.unlock_level = source["unlock_level"];
	        this.owned = source["owned"];
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
	

}

