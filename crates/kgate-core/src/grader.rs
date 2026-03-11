use crate::parser::{ParsedQuestion, ParsedSprint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionResult {
    pub question_number: i32,
    pub correct: bool,
    pub user_answer: char,
    pub correct_answer: char,
    pub xp_earned: i32,
    pub xp_possible: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintResult {
    pub sprint_number: i32,
    pub topic: String,
    pub passed: bool,
    pub score_percent: i32,
    pub correct_count: i32,
    pub total_questions: i32,
    pub xp_earned: i32,
    pub xp_possible: i32,
    pub question_results: Vec<QuestionResult>,
    pub attempt_number: i32,
}

impl SprintResult {
    pub fn wrong_questions(&self) -> Vec<&QuestionResult> {
        self.question_results.iter().filter(|r| !r.correct).collect()
    }
}

pub fn grade_sprint(sprint: &ParsedSprint, answers: &[char], attempt: i32) -> SprintResult {
    let mut question_results = Vec::new();
    let mut correct_count = 0;
    let mut xp_earned = 0;

    for (i, question) in sprint.questions.iter().enumerate() {
        let user_answer = answers.get(i).copied().unwrap_or(' ');
        let correct = user_answer.eq_ignore_ascii_case(&question.answer);

        let q_xp = if correct { question.xp } else { 0 };
        if correct {
            correct_count += 1;
        }
        xp_earned += q_xp;

        question_results.push(QuestionResult {
            question_number: question.number,
            correct,
            user_answer: user_answer.to_ascii_uppercase(),
            correct_answer: question.answer,
            xp_earned: q_xp,
            xp_possible: question.xp,
        });
    }

    let total = sprint.questions.len() as i32;
    let score_percent = if total > 0 {
        (correct_count * 100) / total
    } else {
        0
    };

    // 60% pass threshold (2/3 for 3 questions)
    let passed = score_percent >= 60;

    SprintResult {
        sprint_number: sprint.number,
        topic: sprint.topic.clone(),
        passed,
        score_percent,
        correct_count,
        total_questions: total,
        xp_earned: if passed { xp_earned } else { 0 },
        xp_possible: sprint.total_xp,
        question_results,
        attempt_number: attempt,
    }
}

pub fn get_feedback(result: &SprintResult, questions: &[ParsedQuestion]) -> SprintFeedback {
    let mut feedback = SprintFeedback {
        message: String::new(),
        show_hints: false,
        show_answers: false,
        hints: Vec::new(),
        explanations: Vec::new(),
    };

    if result.passed {
        feedback.message = format!(
            "Sprint {} PASSED! {}/{} correct — {} XP earned",
            result.sprint_number,
            result.correct_count,
            result.total_questions,
            result.xp_earned
        );
    } else {
        feedback.message = format!(
            "{}/{} on Sprint {} — need 60% to pass",
            result.correct_count,
            result.total_questions,
            result.sprint_number
        );

        // Attempt 1: hints only
        if result.attempt_number == 1 {
            feedback.show_hints = true;
            for qr in result.wrong_questions() {
                if let Some(q) = questions.iter().find(|q| q.number == qr.question_number) {
                    if let Some(hint) = &q.hint {
                        feedback.hints.push((qr.question_number, hint.clone()));
                    }
                }
            }
        }
        // Attempt 2+: full answers
        else {
            feedback.show_answers = true;
            for qr in result.wrong_questions() {
                if let Some(q) = questions.iter().find(|q| q.number == qr.question_number) {
                    let exp = q.explanation.clone().unwrap_or_else(|| {
                        format!("The correct answer is {}", qr.correct_answer)
                    });
                    feedback.explanations.push((qr.question_number, qr.correct_answer, exp));
                }
            }
        }
    }

    feedback
}

#[derive(Debug, Clone)]
pub struct SprintFeedback {
    pub message: String,
    pub show_hints: bool,
    pub show_answers: bool,
    pub hints: Vec<(i32, String)>,
    pub explanations: Vec<(i32, char, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ParsedQuestion, ParsedSprint};

    /// Helper: build a ParsedSprint with 3 questions (answers B, A, C) and 10 XP each.
    fn test_sprint() -> ParsedSprint {
        ParsedSprint {
            number: 1,
            topic: "Test Topic".to_string(),
            target_minutes: 3,
            pass_percent: 60,
            total_xp: 30,
            questions: vec![
                ParsedQuestion {
                    number: 1,
                    tier: "RECALL".to_string(),
                    difficulty: "Easy".to_string(),
                    xp: 10,
                    text: "Question 1".to_string(),
                    code_snippet: None,
                    options: vec![
                        "Wrong".to_string(),
                        "Correct".to_string(),
                        "Wrong".to_string(),
                        "Wrong".to_string(),
                    ],
                    answer: 'B',
                    hint: Some("Think about option B".to_string()),
                    explanation: Some("B is correct because...".to_string()),
                    extra: None,
                },
                ParsedQuestion {
                    number: 2,
                    tier: "COMPREHENSION".to_string(),
                    difficulty: "Medium".to_string(),
                    xp: 10,
                    text: "Question 2".to_string(),
                    code_snippet: None,
                    options: vec![
                        "Correct".to_string(),
                        "Wrong".to_string(),
                        "Wrong".to_string(),
                        "Wrong".to_string(),
                    ],
                    answer: 'A',
                    hint: Some("Think about option A".to_string()),
                    explanation: Some("A is correct because...".to_string()),
                    extra: None,
                },
                ParsedQuestion {
                    number: 3,
                    tier: "APPLICATION".to_string(),
                    difficulty: "Challenge".to_string(),
                    xp: 10,
                    text: "Question 3".to_string(),
                    code_snippet: None,
                    options: vec![
                        "Wrong".to_string(),
                        "Wrong".to_string(),
                        "Correct".to_string(),
                        "Wrong".to_string(),
                    ],
                    answer: 'C',
                    hint: Some("Think about option C".to_string()),
                    explanation: Some("C is correct because...".to_string()),
                    extra: None,
                },
            ],
        }
    }

    #[test]
    fn test_all_correct_passes_with_full_xp() {
        let sprint = test_sprint();
        let result = grade_sprint(&sprint, &['B', 'A', 'C'], 1);
        assert!(result.passed);
        assert_eq!(result.score_percent, 100);
        assert_eq!(result.correct_count, 3);
        assert_eq!(result.xp_earned, 30);
        assert_eq!(result.xp_possible, 30);
    }

    #[test]
    fn test_all_wrong_fails_with_zero_xp() {
        let sprint = test_sprint();
        let result = grade_sprint(&sprint, &['A', 'B', 'A'], 1);
        assert!(!result.passed);
        assert_eq!(result.score_percent, 0);
        assert_eq!(result.correct_count, 0);
        // When failed, xp_earned is 0 regardless
        assert_eq!(result.xp_earned, 0);
    }

    #[test]
    fn test_two_of_three_correct_passes() {
        // 2/3 = 66% which is >= 60%
        let sprint = test_sprint();
        let result = grade_sprint(&sprint, &['B', 'A', 'A'], 1); // Q3 wrong
        assert!(result.passed);
        assert_eq!(result.score_percent, 66);
        assert_eq!(result.correct_count, 2);
        assert_eq!(result.xp_earned, 20); // Only XP for correct Qs
    }

    #[test]
    fn test_one_of_three_correct_fails() {
        // 1/3 = 33% which is < 60%
        let sprint = test_sprint();
        let result = grade_sprint(&sprint, &['B', 'B', 'A'], 1); // Q2, Q3 wrong
        assert!(!result.passed);
        assert_eq!(result.score_percent, 33);
        assert_eq!(result.correct_count, 1);
        assert_eq!(result.xp_earned, 0); // 0 when failed
    }

    #[test]
    fn test_case_insensitive_answers() {
        let sprint = test_sprint();
        // Lowercase answers should match uppercase correct answers
        let result = grade_sprint(&sprint, &['b', 'a', 'c'], 1);
        assert!(result.passed);
        assert_eq!(result.score_percent, 100);
        assert_eq!(result.correct_count, 3);
    }

    #[test]
    fn test_empty_answers_fail() {
        let sprint = test_sprint();
        let result = grade_sprint(&sprint, &[], 1);
        assert!(!result.passed);
        assert_eq!(result.score_percent, 0);
        assert_eq!(result.correct_count, 0);
    }

    #[test]
    fn test_feedback_attempt_1_shows_hints_not_answers() {
        let sprint = test_sprint();
        let result = grade_sprint(&sprint, &['A', 'B', 'A'], 1); // all wrong
        let feedback = get_feedback(&result, &sprint.questions);

        assert!(feedback.show_hints);
        assert!(!feedback.show_answers);
        assert!(!feedback.hints.is_empty());
        assert!(feedback.explanations.is_empty());
    }

    #[test]
    fn test_feedback_attempt_2_shows_answers() {
        let sprint = test_sprint();
        let result = grade_sprint(&sprint, &['A', 'B', 'A'], 2); // all wrong, attempt 2
        let feedback = get_feedback(&result, &sprint.questions);

        assert!(!feedback.show_hints);
        assert!(feedback.show_answers);
        assert!(feedback.hints.is_empty());
        assert!(!feedback.explanations.is_empty());
    }

    #[test]
    fn test_feedback_passed_has_congrats_message() {
        let sprint = test_sprint();
        let result = grade_sprint(&sprint, &['B', 'A', 'C'], 1);
        let feedback = get_feedback(&result, &sprint.questions);

        assert!(feedback.message.contains("PASSED"));
        assert!(feedback.message.contains("XP"));
        assert!(!feedback.show_hints);
        assert!(!feedback.show_answers);
    }

    #[test]
    fn test_perfect_score_result() {
        let sprint = test_sprint();
        let result = grade_sprint(&sprint, &['B', 'A', 'C'], 1);
        assert_eq!(result.score_percent, 100);
        assert_eq!(result.xp_earned, result.xp_possible);
        assert!(result.wrong_questions().is_empty());
    }

    #[test]
    fn test_user_answer_stored_uppercase() {
        let sprint = test_sprint();
        let result = grade_sprint(&sprint, &['b', 'a', 'c'], 1);
        // user_answer should be stored as uppercase
        assert_eq!(result.question_results[0].user_answer, 'B');
        assert_eq!(result.question_results[1].user_answer, 'A');
        assert_eq!(result.question_results[2].user_answer, 'C');
    }
}
