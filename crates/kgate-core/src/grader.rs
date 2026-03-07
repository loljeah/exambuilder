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
        let correct = user_answer.to_ascii_uppercase() == question.answer.to_ascii_uppercase();

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
