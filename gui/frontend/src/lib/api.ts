// API adapter for Wails vs Browser mode
// Automatically routes calls to Wails bindings or HTTP bridge

declare global {
    interface Window {
        go?: {
            main?: {
                App?: {
                    GetProjects: () => Promise<ProjectData[]>;
                    SetActiveProject: (id: string) => Promise<void>;
                    GetActiveProject: () => Promise<ProjectData | null>;
                    GetSprints: () => Promise<SprintData[]>;
                    GetSprintQuestions: (n: number) => Promise<QuestionData[]>;
                    SubmitSprintAnswers: (n: number, answers: string[]) => Promise<SprintResultData>;
                    GetSprintHints: (n: number) => Promise<string[]>;
                    GetSprintExplanations: (n: number) => Promise<string[]>;
                    GetDashboardData: () => Promise<DashboardData>;
                    IsPiperAvailable: () => Promise<boolean>;
                    SpeakQuestion: (sprintNumber: number, questionIndex: number) => Promise<void>;
                    SpeakSprintResult: (passed: boolean, scorePercent: number, xpEarned: number) => Promise<void>;
                    StopSpeech: () => Promise<void>;
                    AddProject: (path: string) => Promise<void>;
                    RemoveProject: (id: string) => Promise<void>;
                    ScanAndImportExams: (id: string) => Promise<string>;
                    GetDomains: () => Promise<DomainData[]>;
                    GetDomainAchievements: (domainId: string) => Promise<DomainAchievementData[]>;
                    GetProfile: () => Promise<ProfileData>;
                    GetAvatar: () => Promise<AvatarData>;
                    SetCreatureType: (type: string) => Promise<void>;
                    SetAvatarName: (name: string) => Promise<void>;
                    GetWallet: () => Promise<WalletData>;
                    GetDailyLogin: () => Promise<DailyLoginData>;
                    ClaimDailyReward: () => Promise<number>;
                    GetDailyChallenges: () => Promise<ChallengeData[]>;
                    ClaimChallengeReward: (id: number) => Promise<number>;
                    GetWeeklyGoals: () => Promise<WeeklyGoalData[]>;
                    ClaimWeeklyGoalReward: (id: number) => Promise<number>;
                    GetAchievements: () => Promise<AchievementData[]>;
                    GetAchievementCounts: () => Promise<[number, number]>;
                    GetKnowledgeBase: () => Promise<KnowledgeQuestionData[]>;
                    GetKnowledgeByDomain: (domainId: string) => Promise<KnowledgeQuestionData[]>;
                    GetStats: (period: string) => Promise<DailyStatsData[]>;
                };
            };
        };
    }
}

// Types matching Go structs
export interface ProjectData {
    id: string;
    name: string;
    path: string;
}

export interface SprintData {
    id: number;
    sprint_number: number;
    topic: string;
    status: string;
    best_score: number;
    attempts: number;
    xp_available: number;
    xp_earned: number;
    domain_id: string;
}

export interface QuestionData {
    number: number;
    tier: string;
    stars: number;
    xp: number;
    text: string;
    code: string;
    options: string[];
    correct_idx: number;
    type?: string;
}

export interface SprintResultData {
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
}

export interface QuestionResultData {
    question_num: number;
    correct: boolean;
    user_answer: string;
    right_answer: string;
    xp_earned: number;
}

export interface UnlockedAchievementData {
    id: string;
    name: string;
    icon: string;
    xp_reward: number;
}

export interface DashboardData {
    profile: ProfileData;
    avatar: AvatarData;
    wallet: WalletData;
    daily_login: DailyLoginData;
    challenges: ChallengeData[];
    weekly_goals: WeeklyGoalData[];
    review_due: number;
    active_project: ProjectData | null;
    pending_sprints: number;
}

export interface ProfileData {
    level: number;
    total_xp: number;
    current_streak: number;
    best_streak: number;
    sprints_passed: number;
}

export interface AvatarData {
    creature_type: string;
    name: string;
    mood: string;
    xp_multiplier: number;
}

export interface WalletData {
    coins: number;
    lifetime_coins: number;
}

export interface DailyLoginData {
    current_day: number;
    total_claims: number;
    can_claim: boolean;
}

export interface ChallengeData {
    id: number;
    description: string;
    target: number;
    progress: number;
    reward_coins: number;
    completed: boolean;
    claimed: boolean;
}

export interface WeeklyGoalData {
    id: number;
    description: string;
    target: number;
    progress: number;
    reward_coins: number;
    completed: boolean;
    claimed: boolean;
}

export interface DomainData {
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
}

export interface DomainAchievementData {
    id: string;
    name: string;
    description: string;
    icon: string;
    xp_reward: number;
    unlocked: boolean;
    unlocked_at: string | null;
}

export interface AchievementData {
    id: string;
    name: string;
    description: string;
    category: string;
    icon: string;
    reward_coins: number;
    secret: boolean;
    unlocked: boolean;
    unlocked_at: string;
}

export interface KnowledgeQuestionData {
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
    last_answered: string | null;
    mastered: boolean;
}

export interface DailyStatsData {
    date: string;
    sessions_count: number;
    sprints_attempted: number;
    sprints_passed: number;
    questions_answered: number;
    questions_correct: number;
    xp_earned: number;
}

// Detect Wails vs browser mode
const isWails = (): boolean => typeof window !== 'undefined' && window.go?.main?.App !== undefined;

const HTTP_BASE = 'http://localhost:3001/api';

// Generic HTTP call helper
async function httpCall<T>(method: string, ...args: unknown[]): Promise<T> {
    const response = await fetch(`${HTTP_BASE}/${method}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(args),
    });
    const data = await response.json();
    if (data.error) throw new Error(data.error);
    return data.result as T;
}

// Mode detection helper
export const isBrowserMode = () => !isWails();

// ============================================================================
// Projects & Sprints
// ============================================================================

export async function GetProjects(): Promise<ProjectData[]> {
    if (isWails()) return window.go!.main!.App!.GetProjects();
    return httpCall<ProjectData[]>('GetProjects');
}

export async function SetActiveProject(id: string): Promise<void> {
    if (isWails()) return window.go!.main!.App!.SetActiveProject(id);
    return httpCall<void>('SetActiveProject', id);
}

export async function GetActiveProject(): Promise<ProjectData | null> {
    if (isWails()) return window.go!.main!.App!.GetActiveProject();
    return httpCall<ProjectData | null>('GetActiveProject');
}

export async function GetSprints(): Promise<SprintData[]> {
    if (isWails()) return window.go!.main!.App!.GetSprints();
    return httpCall<SprintData[]>('GetSprints');
}

export async function GetSprintQuestions(n: number): Promise<QuestionData[]> {
    if (isWails()) return window.go!.main!.App!.GetSprintQuestions(n);
    return httpCall<QuestionData[]>('GetSprintQuestions', n);
}

export async function SubmitSprintAnswers(n: number, answers: string[]): Promise<SprintResultData> {
    if (isWails()) return window.go!.main!.App!.SubmitSprintAnswers(n, answers);
    return httpCall<SprintResultData>('SubmitSprintAnswers', n, answers);
}

export async function GetSprintHints(n: number): Promise<string[]> {
    if (isWails()) return window.go!.main!.App!.GetSprintHints(n);
    return httpCall<string[]>('GetSprintHints', n);
}

export async function GetSprintExplanations(n: number): Promise<string[]> {
    if (isWails()) return window.go!.main!.App!.GetSprintExplanations(n);
    return httpCall<string[]>('GetSprintExplanations', n);
}

export async function AddProject(path: string): Promise<void> {
    if (isWails()) return window.go!.main!.App!.AddProject(path);
    return httpCall<void>('AddProject', path);
}

export async function RemoveProject(id: string): Promise<void> {
    if (isWails()) return window.go!.main!.App!.RemoveProject(id);
    return httpCall<void>('RemoveProject', id);
}

export async function ScanAndImportExams(id: string): Promise<string> {
    if (isWails()) return window.go!.main!.App!.ScanAndImportExams(id);
    return httpCall<string>('ScanAndImportExams', id);
}

// ============================================================================
// Dashboard
// ============================================================================

export async function GetDashboardData(): Promise<DashboardData> {
    if (isWails()) return window.go!.main!.App!.GetDashboardData();
    return httpCall<DashboardData>('GetDashboardData');
}

// ============================================================================
// Domains
// ============================================================================

export async function GetDomains(): Promise<DomainData[]> {
    if (isWails()) return window.go!.main!.App!.GetDomains();
    return httpCall<DomainData[]>('GetDomains');
}

export async function GetDomainAchievements(domainId: string): Promise<DomainAchievementData[]> {
    if (isWails()) return window.go!.main!.App!.GetDomainAchievements(domainId);
    return httpCall<DomainAchievementData[]>('GetDomainAchievements', domainId);
}

// ============================================================================
// Profile & Avatar
// ============================================================================

export async function GetProfile(): Promise<ProfileData> {
    if (isWails()) return window.go!.main!.App!.GetProfile();
    return httpCall<ProfileData>('GetProfile');
}

export async function GetAvatar(): Promise<AvatarData> {
    if (isWails()) return window.go!.main!.App!.GetAvatar();
    return httpCall<AvatarData>('GetAvatar');
}

export async function SetCreatureType(type: string): Promise<void> {
    if (isWails()) return window.go!.main!.App!.SetCreatureType(type);
    return httpCall<void>('SetCreatureType', type);
}

export async function SetAvatarName(name: string): Promise<void> {
    if (isWails()) return window.go!.main!.App!.SetAvatarName(name);
    return httpCall<void>('SetAvatarName', name);
}

// ============================================================================
// Wallet & Economy
// ============================================================================

export async function GetWallet(): Promise<WalletData> {
    if (isWails()) return window.go!.main!.App!.GetWallet();
    return httpCall<WalletData>('GetWallet');
}

export async function GetDailyLogin(): Promise<DailyLoginData> {
    if (isWails()) return window.go!.main!.App!.GetDailyLogin();
    return httpCall<DailyLoginData>('GetDailyLogin');
}

export async function ClaimDailyReward(): Promise<number> {
    if (isWails()) return window.go!.main!.App!.ClaimDailyReward();
    return httpCall<number>('ClaimDailyReward');
}

export async function GetDailyChallenges(): Promise<ChallengeData[]> {
    if (isWails()) return window.go!.main!.App!.GetDailyChallenges();
    return httpCall<ChallengeData[]>('GetDailyChallenges');
}

export async function ClaimChallengeReward(id: number): Promise<number> {
    if (isWails()) return window.go!.main!.App!.ClaimChallengeReward(id);
    return httpCall<number>('ClaimChallengeReward', id);
}

export async function GetWeeklyGoals(): Promise<WeeklyGoalData[]> {
    if (isWails()) return window.go!.main!.App!.GetWeeklyGoals();
    return httpCall<WeeklyGoalData[]>('GetWeeklyGoals');
}

export async function ClaimWeeklyGoalReward(id: number): Promise<number> {
    if (isWails()) return window.go!.main!.App!.ClaimWeeklyGoalReward(id);
    return httpCall<number>('ClaimWeeklyGoalReward', id);
}

// ============================================================================
// Achievements
// ============================================================================

export async function GetAchievements(): Promise<AchievementData[]> {
    if (isWails()) return window.go!.main!.App!.GetAchievements();
    return httpCall<AchievementData[]>('GetAchievements');
}

export async function GetAchievementCounts(): Promise<[number, number]> {
    if (isWails()) return window.go!.main!.App!.GetAchievementCounts();
    return httpCall<[number, number]>('GetAchievementCounts');
}

// ============================================================================
// Knowledge Base
// ============================================================================

export async function GetKnowledgeBase(): Promise<KnowledgeQuestionData[]> {
    if (isWails()) return window.go!.main!.App!.GetKnowledgeBase();
    return httpCall<KnowledgeQuestionData[]>('GetKnowledgeBase');
}

export async function GetKnowledgeByDomain(domainId: string): Promise<KnowledgeQuestionData[]> {
    if (isWails()) return window.go!.main!.App!.GetKnowledgeByDomain(domainId);
    return httpCall<KnowledgeQuestionData[]>('GetKnowledgeByDomain', domainId);
}

// ============================================================================
// Stats
// ============================================================================

export async function GetStats(period: string): Promise<DailyStatsData[]> {
    if (isWails()) return window.go!.main!.App!.GetStats(period);
    return httpCall<DailyStatsData[]>('GetStats', period);
}

// ============================================================================
// Voice/TTS (limited in browser mode)
// ============================================================================

export async function IsPiperAvailable(): Promise<boolean> {
    if (isWails()) return window.go!.main!.App!.IsPiperAvailable();
    return false; // TTS not available in browser mode
}

export async function SpeakQuestion(sprintNumber: number, questionIndex: number): Promise<void> {
    if (isWails()) return window.go!.main!.App!.SpeakQuestion(sprintNumber, questionIndex);
    // No-op in browser mode
}

export async function SpeakSprintResult(passed: boolean, scorePercent: number, xpEarned: number): Promise<void> {
    if (isWails()) return window.go!.main!.App!.SpeakSprintResult(passed, scorePercent, xpEarned);
    // No-op in browser mode
}

export async function StopSpeech(): Promise<void> {
    if (isWails()) return window.go!.main!.App!.StopSpeech();
    // No-op in browser mode
}
