use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedExam {
    pub project_name: String,
    pub sprints: Vec<ParsedSprint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSprint {
    pub number: i32,
    pub topic: String,
    pub target_minutes: i32,
    pub pass_percent: i32,
    pub total_xp: i32,
    pub questions: Vec<ParsedQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedQuestion {
    pub number: i32,
    pub tier: String,
    pub difficulty: String,
    pub xp: i32,
    pub text: String,
    pub code_snippet: Option<String>,
    pub options: Vec<String>,
    pub answer: char,
    pub hint: Option<String>,
    pub explanation: Option<String>,
    pub extra: Option<String>,
}

pub fn parse_exam_file(content: &str) -> Result<ParsedExam> {
    let mut project_name = String::new();
    let mut sprints: Vec<ParsedSprint> = Vec::new();
    let mut current_sprint: Option<ParsedSprint> = None;
    let mut current_question: Option<ParsedQuestion> = None;
    let mut in_answer_key = false;
    let mut answers: std::collections::HashMap<(i32, i32), AnswerInfo> = std::collections::HashMap::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Project name from header
        if line.starts_with("# Exam:") {
            project_name = line.trim_start_matches("# Exam:").trim().to_string();
        }

        // Answer key section — stop parsing questions
        if line.contains("Answer Key") || line.contains("🔑") {
            in_answer_key = true;

            // Save the last question/sprint before entering answer key
            if let Some(ref mut sprint) = current_sprint {
                if let Some(q) = current_question.take() {
                    sprint.questions.push(q);
                }
            }
        }

        // Sprint header: ## Sprint N: Topic
        if line.starts_with("## Sprint") && !in_answer_key {
            // Save previous sprint
            if let Some(mut sprint) = current_sprint.take() {
                if let Some(q) = current_question.take() {
                    sprint.questions.push(q);
                }
                sprints.push(sprint);
            }

            let (num, topic) = parse_sprint_header(line)?;

            // Look ahead for target/pass info
            let mut target = 3;
            let mut pass = 70;
            let mut xp = 30;

            for j in (i + 1)..std::cmp::min(i + 5, lines.len()) {
                let look = lines[j];
                if look.contains("Target:") {
                    if let Some(mins) = extract_number(look, "Target:") {
                        target = mins;
                    }
                }
                if look.contains("Pass:") {
                    if let Some(p) = extract_number(look, "Pass:") {
                        pass = p;
                    }
                }
                if look.contains("XP") && !look.starts_with("###") {
                    if let Some(x) = extract_number(look, "") {
                        xp = x;
                    }
                }
            }

            current_sprint = Some(ParsedSprint {
                number: num,
                topic,
                target_minutes: target,
                pass_percent: pass,
                total_xp: xp,
                questions: Vec::new(),
            });
        }

        // Question header: ### Q1. [TIER] Difficulty — XP
        if line.starts_with("### Q") && !in_answer_key {
            if let Some(ref mut sprint) = current_sprint {
                if let Some(q) = current_question.take() {
                    sprint.questions.push(q);
                }
            }

            if let Some(q) = parse_question_header(line) {
                current_question = Some(q);
            }
        }

        // Code block handling
        if line.starts_with("```") && current_question.is_some() && !in_answer_key {
            let mut code_lines: Vec<String> = Vec::new();
            let lang = line.trim_start_matches("```").trim();
            i += 1;

            // Collect until closing ```
            while i < lines.len() && !lines[i].trim().starts_with("```") {
                code_lines.push(lines[i].to_string());
                i += 1;
            }

            if let Some(ref mut q) = current_question {
                let code = code_lines.join("\n");
                if !code.trim().is_empty() {
                    q.code_snippet = Some(if lang.is_empty() {
                        code
                    } else {
                        format!("```{}\n{}\n```", lang, code)
                    });
                }
            }
        }

        // Question text (lines after ### Q header, before options or code)
        if let Some(ref mut q) = current_question {
            if !in_answer_key && !line.starts_with("###") && !line.starts_with("- ") && !line.starts_with("```")
                && !line.is_empty() && !line.starts_with("##") {
                // Append to text (multi-line support)
                if q.text.is_empty() {
                    q.text = line.to_string();
                } else if q.options.is_empty() {
                    // Still collecting question text
                    q.text = format!("{}\n{}", q.text, line);
                }
            }
        }

        // Options: - A) through - D) — strip letter prefix, store text only
        if !in_answer_key && line.starts_with("- ") && line.len() > 4 {
            if let Some(ref mut q) = current_question {
                let raw = &line[2..];
                // Strip "A) ", "B) ", etc. prefix if present
                let opt = if raw.len() >= 3
                    && raw.as_bytes()[0].is_ascii_uppercase()
                    && raw.as_bytes()[1] == b')'
                    && raw.as_bytes()[2] == b' '
                {
                    raw[3..].to_string()
                } else {
                    raw.to_string()
                };
                q.options.push(opt);
            }
        }

        // Parse answer key entries
        if in_answer_key && line.starts_with("**Q") {
            if let Some((sprint_num, q_num, answer, hint, explanation, extra)) = parse_answer_line(line, &lines, i) {
                answers.insert((sprint_num, q_num), AnswerInfo { answer, hint, explanation, extra });
            }
        }

        i += 1;
    }

    // Save last sprint
    if let Some(mut sprint) = current_sprint {
        if let Some(q) = current_question {
            sprint.questions.push(q);
        }
        sprints.push(sprint);
    }

    // Apply answers to questions
    for sprint in &mut sprints {
        for q in &mut sprint.questions {
            if let Some(info) = answers.get(&(sprint.number, q.number)) {
                q.answer = info.answer;
                q.hint = info.hint.clone();
                q.explanation = info.explanation.clone();
                q.extra = info.extra.clone();
            }
        }
    }

    // Recalculate XP per sprint
    for sprint in &mut sprints {
        sprint.total_xp = sprint.questions.iter().map(|q| q.xp).sum();
    }

    Ok(ParsedExam {
        project_name,
        sprints,
    })
}

struct AnswerInfo {
    answer: char,
    hint: Option<String>,
    explanation: Option<String>,
    extra: Option<String>,
}

fn parse_sprint_header(line: &str) -> Result<(i32, String)> {
    // ## Sprint 1: Topic Name
    let line = line.trim_start_matches('#').trim();
    let line = line.trim_start_matches("Sprint").trim();

    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() < 2 {
        return Err(anyhow!("Invalid sprint header: {}", line));
    }

    let num: i32 = parts[0].trim().parse()?;
    let topic = parts[1].trim().to_string();

    Ok((num, topic))
}

fn parse_question_header(line: &str) -> Option<ParsedQuestion> {
    // ### Q1. [RECALL] Easy — 10 XP
    let line = line.trim_start_matches('#').trim();

    // Extract question number
    let num_end = line.find('.')?;
    let num_str = line[1..num_end].trim();
    let number: i32 = num_str.parse().ok()?;

    // Extract tier
    let tier = if line.contains("[RECALL]") {
        "RECALL"
    } else if line.contains("[COMPREHENSION]") {
        "COMPREHENSION"
    } else if line.contains("[APPLICATION]") {
        "APPLICATION"
    } else if line.contains("[ANALYSIS]") {
        "ANALYSIS"
    } else {
        "RECALL"
    }.to_string();

    // Extract difficulty
    let difficulty = if line.to_lowercase().contains("easy") {
        "Easy"
    } else if line.to_lowercase().contains("medium") {
        "Medium"
    } else if line.to_lowercase().contains("challenge") || line.to_lowercase().contains("hard") {
        "Challenge"
    } else {
        "Medium"
    }.to_string();

    // Extract XP
    let xp = extract_number(line, "XP").unwrap_or(10);

    Some(ParsedQuestion {
        number,
        tier,
        difficulty,
        xp,
        text: String::new(),
        code_snippet: None,
        options: Vec::new(),
        answer: 'A',
        hint: None,
        explanation: None,
        extra: None,
    })
}

fn parse_answer_line(line: &str, lines: &[&str], idx: usize) -> Option<(i32, i32, char, Option<String>, Option<String>, Option<String>)> {
    // **Q1. Answer: B** — 10 XP
    // Look for sprint context from preceding ### Sprint header
    let mut sprint_num = 1;
    for j in (0..idx).rev() {
        if lines[j].contains("### Sprint") {
            if let Some(n) = extract_number(lines[j], "Sprint") {
                sprint_num = n;
                break;
            }
        }
    }

    let q_start = line.find('Q')? + 1;
    let q_end = line.find('.')?;
    let q_num: i32 = line[q_start..q_end].trim().parse().ok()?;

    let answer = if line.contains("Answer:") {
        let ans_start = line.find("Answer:")? + 7;
        line[ans_start..].trim().chars().next()?
    } else {
        'A'
    };

    // Look for hint and explanation in following lines
    let mut hint = None;
    let mut explanation = None;
    let mut extra = None;

    for j in (idx + 1)..std::cmp::min(idx + 10, lines.len()) {
        let l = lines[j];
        if l.starts_with("Hint:") {
            hint = Some(l.trim_start_matches("Hint:").trim().to_string());
        }
        if l.starts_with("Full:") {
            explanation = Some(l.trim_start_matches("Full:").trim().to_string());
        }
        if l.starts_with("Extra:") {
            extra = Some(l.trim_start_matches("Extra:").trim().to_string());
        }
        if l.starts_with("**Q") || l.starts_with("### Sprint") || l.starts_with("## ") {
            break;
        }
    }

    Some((sprint_num, q_num, answer, hint, explanation, extra))
}

fn extract_number(s: &str, after: &str) -> Option<i32> {
    let start = if after.is_empty() { 0 } else { s.find(after)? + after.len() };
    let substr = &s[start..];

    let mut num_str = String::new();
    for c in substr.chars() {
        if c.is_ascii_digit() {
            num_str.push(c);
        } else if !num_str.is_empty() {
            break;
        }
    }

    num_str.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sprint_header() {
        let (num, topic) = parse_sprint_header("## Sprint 1: Bash Scripting Basics").unwrap();
        assert_eq!(num, 1);
        assert_eq!(topic, "Bash Scripting Basics");
    }

    #[test]
    fn test_parse_question_header() {
        let q = parse_question_header("### Q1. [RECALL] Easy — 10 XP").unwrap();
        assert_eq!(q.number, 1);
        assert_eq!(q.tier, "RECALL");
        assert_eq!(q.difficulty, "Easy");
        assert_eq!(q.xp, 10);
    }

    #[test]
    fn test_answer_key_does_not_leak_into_options() {
        let content = r#"# Exam: TestProject

## Sprint 1: Basics
**Topic**: Testing
**Target Time**: 3 minutes
**Questions**: 2

### Q1. [RECALL] Easy — 10 XP

What is 1+1?

- A) 1
- B) 2
- C) 3
- D) 4

### Q2. [RECALL] Medium — 10 XP

What is 2+2?

- A) 2
- B) 3
- C) 4
- D) 5

---

## Answer Key

### Sprint 1
- Q1: **B** - 2
- Q2: **C** - 4
"#;
        let exam = parse_exam_file(content).unwrap();
        assert_eq!(exam.sprints.len(), 1);
        assert_eq!(exam.sprints[0].questions.len(), 2);

        // Q1 should have exactly 4 options, not more
        assert_eq!(exam.sprints[0].questions[0].options.len(), 4);
        // Q2 should have exactly 4 options — answer key lines must not leak
        assert_eq!(exam.sprints[0].questions[1].options.len(), 4);
    }

    #[test]
    fn test_homeb0t_format_no_bleeding() {
        // Reproduces the exact homeb0t exam format: answer key uses "- Q1: C) ..."
        // style (not "**Q1. Answer: B**"), plus study resources with "- /path/..." lines.
        // The last question of the last sprint must not absorb answer key or study resource lines.
        let content = r#"# exam_homeb0t.md

## Knowledge Gate: homeb0t

## Sprint 1: Architecture

### Q1. [EASY] -- 10 XP

What platform?

- A) Telegram
- B) WhatsApp
- C) Signal
- D) Discord

---

## Sprint 2: Cameras

### Q1. [EASY] -- 10 XP

What type is cam1?

- A) usb
- B) csi
- C) esp32cam
- D) rtsp

### Q2. [MEDIUM] -- 10 XP

What capture method?

- A) RTSP
- B) MJPEG
- C) HTTP GET
- D) V4L2

### Q3. [BOSS] -- 10 XP

What mode for websocket?

- A) normal
- B) native
- C) json-rpc
- D) websocket

---

## XP Progress

| Sprint | XP |
|--------|----|
| 1 | 10 |
| 2 | 30 |

---

## Answer Key

**Sprint 1:**
- Q1: C) Signal

**Sprint 2:**
- Q1: C) esp32cam (from config.yaml)
- Q2: C) HTTP GET to /snapshot endpoint
- Q3: C) json-rpc (from setup_signal_api.sh)

---

## Study Resources

*Unlocked after attempt*

**Sprint 1:**
- `/home/ljsm/gitZ/homeb0t/CONCEPT.md` - Architecture reference

**Sprint 2:**
- `/home/ljsm/gitZ/homeb0t/config.yaml` - Camera config
- `/home/ljsm/gitZ/homeb0t/scripts/setup_signal_api.sh` - Signal API setup
"#;
        let exam = parse_exam_file(content).unwrap();

        // Sprint 1 Q1: exactly 4 options
        assert_eq!(exam.sprints[0].questions[0].options.len(), 4,
            "Sprint 1 Q1 has {} options, expected 4: {:?}",
            exam.sprints[0].questions[0].options.len(),
            exam.sprints[0].questions[0].options);

        // Sprint 2 Q3 (last question, last sprint): exactly 4 options — no bleeding
        let last_sprint = exam.sprints.last().unwrap();
        let last_q = last_sprint.questions.last().unwrap();
        assert_eq!(last_q.options.len(), 4,
            "Sprint 2 Q3 has {} options, expected 4: {:?}",
            last_q.options.len(), last_q.options);
    }

    #[test]
    fn test_extra_field_parsed() {
        let content = r#"# Exam: TestProject

## Sprint 1: Basics
**Questions**: 1

### Q1. [RECALL] Easy — 10 XP

What is 1+1?

- A) 1
- B) 2
- C) 3
- D) 4

---

## Answer Key

### Sprint 1
**Q1. Answer: B** — 10 XP
Hint: Think simple addition
Full: 1+1 equals 2
Extra: The successor of 1 in Peano arithmetic is defined as S(1) = 2.
"#;
        let exam = parse_exam_file(content).unwrap();
        let q = &exam.sprints[0].questions[0];
        assert_eq!(q.answer, 'B');
        assert_eq!(q.hint.as_deref(), Some("Think simple addition"));
        assert_eq!(q.explanation.as_deref(), Some("1+1 equals 2"));
        assert_eq!(q.extra.as_deref(), Some("The successor of 1 in Peano arithmetic is defined as S(1) = 2."));
    }
}
