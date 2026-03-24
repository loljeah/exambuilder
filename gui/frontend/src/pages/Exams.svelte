<script>
  import { onMount, onDestroy } from 'svelte';
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';
  import ProgressBar from '../lib/components/ProgressBar.svelte';
  import { playClick, playCorrect, playWrong, playSprintPassed, playSprintFailed, playLevelUp, playXP, playAchievement } from '../lib/audio.js';
  import * as api from '../lib/api';

  // Helper to add timeout to promises
  function withTimeout(promise, ms = 5000) {
    return Promise.race([
      promise,
      new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Request timeout')), ms)
      )
    ]);
  }

  // View states: 'select' | 'taking' | 'results'
  let view = 'select';

  // Selection state
  let projects = [];
  let sprints = [];
  let selectedProject = null;
  let selectedSprint = null;

  // Exam taking state
  let questions = [];
  let currentQuestionIndex = 0;
  let answers = [];
  let selectedAnswer = null;
  let timeElapsed = 0;
  let timerInterval = null;

  // Voice/typewriter state
  let piperAvailable = false;
  let isSpeaking = false;
  let typewriterText = '';
  let typewriterInterval = null;
  let displayQuestionText = ''; // Just the question text for display

  // Results state
  let result = null;
  let hints = [];
  let explanations = [];

  async function loadProjects() {
    try {
      projects = await withTimeout(api.GetProjects(), 5000) || [];
      // Check if piper is available
      piperAvailable = await withTimeout(api.IsPiperAvailable(), 2000);
      console.log('Piper available:', piperAvailable);
    } catch (err) {
      console.error('loadProjects error:', err);
      projects = [];
    }
  }

  async function selectProject(project) {
    selectedProject = project;
    selectedSprint = null;
    try {
      await withTimeout(api.SetActiveProject(project.id), 5000);
      sprints = await withTimeout(api.GetSprints(), 5000) || [];
    } catch (err) {
      console.error('selectProject error:', err);
      sprints = [];
    }
  }

  async function startExam(sprint) {
    selectedSprint = sprint;
    try {
      questions = await withTimeout(api.GetSprintQuestions(sprint.sprint_number), 5000) || [];
    } catch (err) {
      console.error('GetSprintQuestions error:', err);
      questions = [];
    }

    if (questions.length === 0) {
      console.warn('No questions found for sprint');
      return;
    }

    answers = questions.map(q => q.type === 'multi' ? [] : null);
    currentQuestionIndex = 0;
    selectedAnswer = answers[0] ?? null;
    timeElapsed = 0;
    result = null;
    hints = [];
    explanations = [];
    typewriterText = '';
    displayQuestionText = '';

    if (timerInterval) clearInterval(timerInterval);
    timerInterval = setInterval(() => timeElapsed++, 1000);

    view = 'taking';

    // Start voice/typewriter for first question
    await speakCurrentQuestion();
  }

  function stopSpeechAndTypewriter() {
    stopTypewriter();
    if (isSpeaking) {
      api.StopSpeech().catch(() => {});
    }
    isSpeaking = false;
  }

  async function speakCurrentQuestion() {
    // Stop any current speech
    stopSpeechAndTypewriter();
    typewriterText = '';
    displayQuestionText = '';

    if (!selectedSprint || currentQuestionIndex < 0 || !currentQuestion) return;

    // Set the display text to just the question
    displayQuestionText = currentQuestion.text;

    // Start typewriter effect for just the question text
    startTypewriter(displayQuestionText);

    // Start TTS if piper is available
    if (piperAvailable) {
      console.log('Speaking question:', selectedSprint.sprint_number, currentQuestionIndex);
      isSpeaking = true;
      api.SpeakQuestion(selectedSprint.sprint_number, currentQuestionIndex)
        .then(() => console.log('SpeakQuestion completed'))
        .catch(err => console.error('SpeakQuestion error:', err))
        .finally(() => { isSpeaking = false; });
    }
  }

  function startTypewriter(text) {
    stopTypewriter();
    typewriterText = '';
    let index = 0;
    const speed = 25; // ms per character

    typewriterInterval = setInterval(() => {
      if (index < text.length) {
        typewriterText += text[index];
        index++;
      } else {
        stopTypewriter();
      }
    }, speed);
  }

  function stopTypewriter() {
    if (typewriterInterval) {
      clearInterval(typewriterInterval);
      typewriterInterval = null;
    }
    // Show full question text when stopped
    if (displayQuestionText) {
      typewriterText = displayQuestionText;
    }
  }

  function selectAnswerOption(optionIndex) {
    // Stop speech when selecting answer
    stopSpeechAndTypewriter();

    const letter = String.fromCharCode(65 + optionIndex);
    const q = questions[currentQuestionIndex];

    if (q?.type === 'multi') {
      let current = Array.isArray(selectedAnswer) ? [...selectedAnswer] : [];
      const idx = current.indexOf(letter);
      if (idx >= 0) {
        current.splice(idx, 1);
      } else {
        current.push(letter);
        current.sort();
      }
      selectedAnswer = current;
      answers[currentQuestionIndex] = current;
    } else {
      selectedAnswer = letter;
      answers[currentQuestionIndex] = letter;
    }
    playClick();
  }

  async function nextQuestion() {
    if (currentQuestionIndex < questions.length - 1) {
      currentQuestionIndex++;
      selectedAnswer = answers[currentQuestionIndex];
      await speakCurrentQuestion();
    }
  }

  async function prevQuestion() {
    if (currentQuestionIndex > 0) {
      currentQuestionIndex--;
      selectedAnswer = answers[currentQuestionIndex];
      await speakCurrentQuestion();
    }
  }

  async function goToQuestion(index) {
    currentQuestionIndex = index;
    selectedAnswer = answers[currentQuestionIndex];
    await speakCurrentQuestion();
  }

  async function submitExam() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }

    try {
      const formattedAnswers = answers.map(a => {
        if (Array.isArray(a)) return a.join(',');
        return a || '';
      });
      result = await withTimeout(api.SubmitSprintAnswers(
        selectedSprint.sprint_number,
        formattedAnswers
      ), 10000);

      if (result.passed) {
        playSprintPassed();
        if (result.xp_earned > 0) setTimeout(() => playXP(), 600);
        if (result.domain_level_up) setTimeout(() => playLevelUp(), 1000);
        if (result.unlocked_achievements?.length > 0) {
          setTimeout(() => playAchievement(), result.domain_level_up ? 1800 : 1200);
        }
      } else {
        playSprintFailed();
      }

      explanations = await withTimeout(api.GetSprintExplanations(selectedSprint.sprint_number), 5000) || [];

      if (!result.passed && result.attempt_number >= 1) {
        hints = await withTimeout(api.GetSprintHints(selectedSprint.sprint_number), 5000) || [];
      }

      // Refresh sprints list to show updated status
      sprints = await withTimeout(api.GetSprints(), 5000) || [];

      // Speak result if piper available
      if (piperAvailable) {
        api.SpeakSprintResult(result.passed, result.score_percent, result.xp_earned)
          .catch(err => console.warn('SpeakSprintResult error:', err));
      }

      view = 'results';
    } catch (err) {
      console.error('submitExam error:', err);
    }
  }

  function backToSelect() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    stopSpeechAndTypewriter();
    view = 'select';
    result = null;
  }

  function retakeExam() {
    startExam(selectedSprint);
  }

  function handleExamKeydown(e) {
    if (view !== 'taking' || !currentQuestion) return;

    // 1-4 for answer selection
    if (['1', '2', '3', '4'].includes(e.key)) {
      const optionIndex = parseInt(e.key) - 1;
      if (optionIndex < currentQuestion.options.length) {
        selectAnswerOption(optionIndex);
        e.preventDefault();
      }
      return;
    }

    // A-D for answer selection
    const key = e.key.toUpperCase();
    if (['A', 'B', 'C', 'D'].includes(key)) {
      const optionIndex = key.charCodeAt(0) - 65;
      if (optionIndex < currentQuestion.options.length) {
        selectAnswerOption(optionIndex);
        e.preventDefault();
      }
      return;
    }

    // Enter for next/submit
    if (e.key === 'Enter') {
      if (currentQuestionIndex === questions.length - 1) {
        if (answeredCount >= questions.length) submitExam();
      } else {
        nextQuestion();
      }
      e.preventDefault();
      return;
    }

    // Escape to exit
    if (e.key === 'Escape') {
      backToSelect();
      e.preventDefault();
    }
  }

  function formatTime(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  $: currentQuestion = questions[currentQuestionIndex];
  $: answeredCount = answers.filter(a => {
    if (Array.isArray(a)) return a.length > 0;
    return a !== null;
  }).length;
  $: progressPercent = questions.length > 0 ? (answeredCount / questions.length) * 100 : 0;

  onMount(() => {
    loadProjects();
  });

  onDestroy(() => {
    if (timerInterval) clearInterval(timerInterval);
    stopSpeechAndTypewriter();
  });
</script>

<svelte:window on:keydown={handleExamKeydown} />

<div class="exams-page">
  {#if view === 'select'}
    <!-- EXAM SELECTION VIEW -->
    <h1 class="page-title">Take Exam</h1>

    <div class="selection-grid">
      <Card title="Select Project">
        <div class="project-list">
          {#each projects as project}
            <button
              class="project-item"
              class:selected={selectedProject?.id === project.id}
              on:click={() => selectProject(project)}
            >
              <span class="project-icon">📁</span>
              <span class="project-name">{project.name}</span>
            </button>
          {:else}
            <p class="empty">No projects found. Add a project first.</p>
          {/each}
        </div>
      </Card>

      {#if selectedProject}
        <Card title="Select Sprint" subtitle={selectedProject.name}>
          <div class="sprint-list">
            {#each sprints as sprint}
              {@const isCompleted = sprint.status === 'passed' || sprint.status === 'completed'}
              <button
                class="sprint-item"
                class:completed={isCompleted}
                on:click={() => startExam(sprint)}
              >
                <div class="sprint-status">
                  {#if isCompleted}✓{:else}○{/if}
                </div>
                <div class="sprint-info">
                  <h4>Sprint {sprint.sprint_number}: {sprint.topic}</h4>
                  <p>
                    {#if isCompleted}
                      Completed • Best: {sprint.best_score}% • {sprint.attempts} attempts
                    {:else if sprint.attempts > 0}
                      Best: {sprint.best_score}% • {sprint.attempts} attempts
                    {:else}
                      Not attempted
                    {/if}
                  </p>
                </div>
                <div class="sprint-xp">
                  {#if isCompleted}
                    <span class="earned">✓ {sprint.xp_earned} XP</span>
                  {:else}
                    <span>⭐ {sprint.xp_available} XP</span>
                  {/if}
                </div>
                <span class="sprint-action">
                  {isCompleted ? 'Retake' : 'Start'} →
                </span>
              </button>
            {:else}
              <p class="empty">No sprints found. Scan the project first.</p>
            {/each}
          </div>
        </Card>
      {/if}
    </div>

  {:else if view === 'taking'}
    <!-- EXAM TAKING VIEW -->
    <div class="exam-header">
      <div class="exam-title">
        <button class="exit-btn" on:click={backToSelect}>✕</button>
        <h2>Sprint {selectedSprint.sprint_number}: {selectedSprint.topic}</h2>
        <span class="timer">⏱️ {formatTime(timeElapsed)}</span>
      </div>
      <div class="exam-progress">
        <ProgressBar value={progressPercent} />
        <span class="progress-text">{answeredCount}/{questions.length} answered</span>
      </div>
    </div>

    <div class="exam-content">
      {#if currentQuestion}
        <!-- Question Box -->
        <Card>
          <div class="question-header">
            <span class="question-number">Q{currentQuestion.number}</span>
            <span class="question-tier tier-{currentQuestion.tier?.toLowerCase()}">{currentQuestion.tier}</span>
            <span class="question-type-badge" class:multi={currentQuestion.type === 'multi'}>
              {currentQuestion.type === 'multi' ? '☑ Multi' : '○ Single'}
            </span>
            <span class="question-xp">⭐ {currentQuestion.xp} XP</span>
            {#if piperAvailable}
              <button class="voice-btn" class:speaking={isSpeaking} on:click={speakCurrentQuestion} title="Read aloud">
                {isSpeaking ? '🔊' : '🔈'}
              </button>
            {/if}
          </div>

          <div class="question-box">
            <div class="question-label">Question</div>
            <div class="question-text">
              {#if typewriterText && typewriterText !== displayQuestionText}
                <p class="typewriter">{typewriterText}</p>
              {:else}
                <p>{currentQuestion.text}</p>
              {/if}
            </div>
            {#if currentQuestion.code}
              <pre class="code-block"><code>{currentQuestion.code}</code></pre>
            {/if}
          </div>
        </Card>

        <!-- Answers Box -->
        <Card>
          <div class="answers-box">
            <div class="answers-label">
              {#if currentQuestion.type === 'multi'}
                <span class="multi-indicator">☑</span>
                <span>Select ALL that apply</span>
              {:else}
                <span class="single-indicator">○</span>
                <span>Choose one answer</span>
              {/if}
            </div>

            <div class="options-list">
              {#each currentQuestion.options as option, i}
                {@const letter = String.fromCharCode(65 + i)}
                {@const isSelected = currentQuestion.type === 'multi'
                  ? Array.isArray(selectedAnswer) && selectedAnswer.includes(letter)
                  : selectedAnswer === letter}
                <button
                  class="option-btn"
                  class:selected={isSelected}
                  class:multi={currentQuestion.type === 'multi'}
                  on:click={() => selectAnswerOption(i)}
                >
                  <span class="option-letter">
                    {#if currentQuestion.type === 'multi'}
                      {isSelected ? '☑' : '☐'}
                    {:else}
                      {isSelected ? '◉' : '○'}
                    {/if}
                  </span>
                  <span class="option-letter-label">{letter}</span>
                  <span class="option-text">{option}</span>
                </button>
              {/each}
            </div>
          </div>
        </Card>

        <div class="question-nav">
          <Button variant="secondary" disabled={currentQuestionIndex === 0} on:click={prevQuestion}>
            ← Previous
          </Button>

          <div class="question-dots">
            {#each questions as _, i}
              <button
                class="question-dot"
                class:current={i === currentQuestionIndex}
                class:answered={answers[i] !== null && (Array.isArray(answers[i]) ? answers[i].length > 0 : true)}
                on:click={() => goToQuestion(i)}
              >
                {i + 1}
              </button>
            {/each}
          </div>

          {#if currentQuestionIndex === questions.length - 1}
            <Button variant="primary" disabled={answeredCount < questions.length} on:click={submitExam}>
              Submit ✓
            </Button>
          {:else}
            <Button variant="secondary" on:click={nextQuestion}>
              Next →
            </Button>
          {/if}
        </div>
      {/if}
    </div>

  {:else if view === 'results'}
    <!-- RESULTS VIEW -->
    <div class="results-page">
      <div class="results-header" class:passed={result.passed}>
        <div class="results-icon">{result.passed ? '🎉' : '💪'}</div>
        <h1>{result.passed ? 'Sprint Passed!' : 'Keep Going!'}</h1>
        <p class="results-subtitle">Sprint {result.sprint_num}: {result.topic}</p>
      </div>

      <div class="results-stats">
        <div class="stat-box">
          <span class="stat-value">{result.score_percent}%</span>
          <span class="stat-label">Score</span>
        </div>
        <div class="stat-box">
          <span class="stat-value">{result.correct_count}/{result.total_questions}</span>
          <span class="stat-label">Correct</span>
        </div>
        <div class="stat-box xp">
          <span class="stat-value">+{result.xp_earned}</span>
          <span class="stat-label">XP Earned</span>
        </div>
        <div class="stat-box coins">
          <span class="stat-value">+{result.coins_earned}</span>
          <span class="stat-label">Coins</span>
        </div>
      </div>

      <Card title="Question Breakdown">
        <div class="question-results">
          {#each result.question_results as qr, i}
            <div class="question-result" class:correct={qr.correct} class:incorrect={!qr.correct}>
              <div class="qr-indicator">{qr.correct ? '✓' : '✗'}</div>
              <div class="qr-info">
                <span class="qr-number">Q{qr.question_num}</span>
                <span class="qr-answer">
                  Your answer: {qr.user_answer || '—'}
                  {#if !qr.correct}
                    <span class="correct-answer">(Correct: {qr.right_answer})</span>
                  {/if}
                </span>
              </div>
              <div class="qr-xp">{#if qr.correct}+{qr.xp_earned} XP{/if}</div>
            </div>

            {#if !qr.correct && hints[i]}
              <div class="hint-box">
                <span class="hint-label">💡 Hint:</span>
                <span class="hint-text">{hints[i]}</span>
              </div>
            {/if}

            {#if explanations[i]}
              <div class="explanation-box" class:correct-explanation={qr.correct}>
                <span class="explanation-label">{qr.correct ? '✓ Why correct:' : '📖 Explanation:'}</span>
                <span class="explanation-text">{explanations[i]}</span>
              </div>
            {/if}
          {/each}
        </div>
      </Card>

      {#if result.domain_level_up}
        <div class="domain-level-up">
          <span class="level-up-icon">⬆️</span>
          <div class="level-up-content">
            <h3>{result.domain_name} Level Up!</h3>
            <p>Level {result.domain_new_level}: <span class="level-title">{result.domain_new_title}</span></p>
          </div>
        </div>
      {/if}

      {#if result.unlocked_achievements?.length > 0}
        <div class="achievements-unlocked">
          <h3>🏆 Achievements Unlocked</h3>
          <div class="achievement-list">
            {#each result.unlocked_achievements as ach}
              <div class="achievement-card">
                <span class="ach-icon">{ach.icon || '🎖️'}</span>
                <div class="ach-info">
                  <span class="ach-name">{ach.name}</span>
                  <span class="ach-xp">+{ach.xp_reward} XP</span>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <div class="results-actions">
        {#if !result.passed}
          <Button variant="primary" on:click={retakeExam}>Retry Sprint</Button>
        {/if}
        <Button variant="secondary" on:click={backToSelect}>Back to Exams</Button>
      </div>
    </div>
  {/if}
</div>

<style>
  .exams-page {
    max-width: 100%;
    width: 100%;
    margin: 0 auto;
    box-sizing: border-box;
  }

  .exam-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    width: 100%;
    max-width: 100%;
    overflow: hidden;
  }

  .page-title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: var(--spacing-lg);
  }

  .selection-grid {
    display: grid;
    grid-template-columns: 300px 1fr;
    gap: var(--spacing-lg);
  }

  @media (max-width: 768px) {
    .selection-grid {
      grid-template-columns: 1fr;
    }
  }

  .project-list, .sprint-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .project-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-tertiary);
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    color: var(--text-primary);
    transition: all 0.15s;
  }

  .project-item:hover {
    background: var(--bg-hover);
  }

  .project-item.selected {
    border-color: var(--primary-500);
    background: var(--primary-900);
  }

  .project-icon {
    font-size: 18px;
  }

  .project-name {
    flex: 1;
  }

  .sprint-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    color: var(--text-primary);
    transition: all 0.15s;
  }

  .sprint-item:hover {
    background: var(--bg-hover);
    transform: translateX(4px);
  }

  .sprint-status {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    background: var(--bg-card);
    border-radius: 50%;
  }

  .sprint-item.completed .sprint-status {
    background: var(--accent-green);
    color: white;
  }

  .sprint-info {
    flex: 1;
  }

  .sprint-info h4 {
    margin: 0 0 var(--spacing-xs);
    font-size: 14px;
  }

  .sprint-info p {
    margin: 0;
    font-size: 12px;
    color: var(--text-muted);
  }

  .sprint-xp {
    color: var(--accent-gold);
    font-size: 13px;
  }

  .sprint-xp .earned {
    color: var(--accent-green);
  }

  .sprint-action {
    color: var(--primary-400);
    font-size: 13px;
  }

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-lg);
  }

  /* Exam Taking View */
  .exam-header {
    margin-bottom: var(--spacing-lg);
  }

  .exam-title {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-md);
  }

  .exit-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    border: none;
    border-radius: var(--radius-md);
    color: var(--text-muted);
    font-size: 16px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .exit-btn:hover {
    background: var(--accent-red);
    color: white;
  }

  .exam-title h2 {
    font-size: 20px;
    margin: 0;
    flex: 1;
  }

  .timer {
    font-size: 16px;
    color: var(--text-secondary);
    font-family: monospace;
  }

  .exam-progress {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .progress-text {
    font-size: 13px;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .question-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-md);
  }

  .question-number {
    font-weight: 700;
    font-size: 18px;
  }

  .question-tier {
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .tier-easy { background: var(--accent-green); color: white; }
  .tier-medium { background: var(--primary-500); color: white; }
  .tier-boss, .tier-hard { background: var(--accent-red); color: white; }

  .question-type-badge {
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 600;
    background: var(--bg-tertiary);
    color: var(--text-muted);
  }

  .question-type-badge.multi {
    background: var(--primary-700);
    color: var(--primary-200);
  }

  .question-xp {
    margin-left: auto;
    color: var(--accent-gold);
  }

  .voice-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: 16px;
    transition: all 0.15s;
  }

  .voice-btn:hover {
    background: var(--primary-700);
  }

  .voice-btn.speaking {
    background: var(--primary-500);
    animation: pulse-voice 1s ease-in-out infinite;
  }

  @keyframes pulse-voice {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.1); }
  }

  .question-box {
    border: 1px solid var(--bg-tertiary);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    background: var(--bg-primary);
    overflow: hidden;
    word-wrap: break-word;
    overflow-wrap: break-word;
  }

  .question-label, .answers-label {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    margin-bottom: var(--spacing-sm);
  }

  .multi-indicator {
    color: var(--primary-400);
    font-size: 16px;
  }

  .single-indicator {
    color: var(--text-muted);
    font-size: 16px;
  }

  .question-text {
    margin-bottom: 0;
  }

  .question-text p {
    font-size: 16px;
    line-height: 1.6;
    margin: 0;
    word-wrap: break-word;
    overflow-wrap: break-word;
    white-space: pre-wrap;
  }

  .question-text .typewriter {
    display: inline;
    border-right: 2px solid var(--primary-400);
    animation: blink-cursor 0.7s step-end infinite;
    word-wrap: break-word;
    overflow-wrap: break-word;
    white-space: pre-wrap;
  }

  @keyframes blink-cursor {
    0%, 100% { border-color: var(--primary-400); }
    50% { border-color: transparent; }
  }

  .code-block {
    margin-top: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-card);
    border-radius: var(--radius-md);
    overflow-x: auto;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 13px;
  }

  .answers-box {
    padding: var(--spacing-sm);
  }

  .options-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .option-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: all 0.15s;
    color: var(--text-primary);
  }

  .option-btn:hover {
    background: var(--bg-hover);
  }

  .option-btn.selected {
    border-color: var(--primary-500);
    background: var(--primary-900);
  }

  .option-letter {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    color: var(--text-muted);
  }

  .option-btn.selected .option-letter {
    color: var(--primary-400);
  }

  .option-btn.multi .option-letter {
    font-size: 18px;
  }

  .option-btn.multi.selected .option-letter {
    color: var(--primary-400);
  }

  .option-letter-label {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    border-radius: 50%;
    font-weight: 600;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .option-btn.selected .option-letter-label {
    background: var(--primary-500);
    color: white;
  }

  .option-text {
    flex: 1;
    font-size: 14px;
  }

  .question-nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: var(--spacing-lg);
    padding-top: var(--spacing-lg);
    border-top: 1px solid var(--bg-tertiary);
  }

  .question-dots {
    display: flex;
    gap: var(--spacing-xs);
  }

  .question-dot {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    border: none;
    border-radius: 50%;
    font-size: 11px;
    cursor: pointer;
    color: var(--text-muted);
    transition: all 0.15s;
  }

  .question-dot:hover {
    background: var(--bg-hover);
  }

  .question-dot.current {
    background: var(--primary-500);
    color: white;
  }

  .question-dot.answered {
    background: var(--accent-green);
    color: white;
  }

  .question-dot.current.answered {
    background: var(--primary-500);
  }

  /* Results View */
  .results-page {
    text-align: center;
  }

  .results-header {
    padding: var(--spacing-xl);
    margin-bottom: var(--spacing-lg);
    border-radius: var(--radius-lg);
    background: linear-gradient(135deg, var(--bg-card) 0%, var(--bg-tertiary) 100%);
  }

  .results-header.passed {
    background: linear-gradient(135deg, var(--accent-green) 0%, var(--primary-600) 100%);
  }

  .results-icon {
    font-size: 48px;
    margin-bottom: var(--spacing-md);
  }

  .results-header h1 {
    margin: 0 0 var(--spacing-xs);
    font-size: 28px;
  }

  .results-subtitle {
    margin: 0;
    opacity: 0.8;
  }

  .results-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
  }

  .stat-box {
    background: var(--bg-card);
    padding: var(--spacing-md);
    border-radius: var(--radius-md);
  }

  .stat-value {
    display: block;
    font-size: 24px;
    font-weight: 700;
  }

  .stat-label {
    font-size: 12px;
    color: var(--text-muted);
  }

  .stat-box.xp .stat-value,
  .stat-box.coins .stat-value { color: var(--accent-gold); }

  .question-results {
    text-align: left;
  }

  .question-result {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-sm);
    margin-bottom: var(--spacing-xs);
  }

  .question-result.correct {
    background: rgba(74, 222, 128, 0.1);
  }

  .question-result.incorrect {
    background: rgba(248, 113, 113, 0.1);
  }

  .qr-indicator {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-weight: 700;
    font-size: 12px;
  }

  .question-result.correct .qr-indicator {
    background: var(--accent-green);
    color: white;
  }

  .question-result.incorrect .qr-indicator {
    background: var(--accent-red);
    color: white;
  }

  .qr-info {
    flex: 1;
  }

  .qr-number {
    font-weight: 600;
    margin-right: var(--spacing-sm);
  }

  .qr-answer {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .correct-answer {
    color: var(--accent-green);
    margin-left: var(--spacing-xs);
  }

  .qr-xp {
    color: var(--accent-gold);
    font-size: 13px;
  }

  .hint-box, .explanation-box {
    margin: var(--spacing-xs) 0 var(--spacing-sm) 40px;
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-sm);
    font-size: 13px;
  }

  .hint-box {
    background: rgba(252, 211, 77, 0.1);
    border-left: 3px solid var(--accent-gold);
  }

  .explanation-box {
    background: rgba(147, 112, 219, 0.1);
    border-left: 3px solid var(--primary-400);
  }

  .explanation-box.correct-explanation {
    background: rgba(34, 197, 94, 0.1);
    border-left: 3px solid var(--accent-green);
  }

  .hint-label, .explanation-label {
    font-weight: 600;
    margin-right: var(--spacing-xs);
  }

  .results-actions {
    display: flex;
    justify-content: center;
    gap: var(--spacing-md);
    margin-top: var(--spacing-lg);
  }

  .domain-level-up {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-lg);
    background: linear-gradient(135deg, var(--primary-700) 0%, var(--primary-500) 100%);
    border-radius: var(--radius-lg);
    margin-bottom: var(--spacing-lg);
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.02); }
  }

  .level-up-icon {
    font-size: 48px;
  }

  .level-up-content h3 {
    margin: 0;
    font-size: 20px;
    color: white;
  }

  .level-up-content p {
    margin: var(--spacing-xs) 0 0;
    font-size: 16px;
    color: rgba(255, 255, 255, 0.9);
  }

  .level-title {
    font-weight: 700;
    color: var(--accent-gold);
  }

  .achievements-unlocked {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    margin-bottom: var(--spacing-lg);
    border: 2px solid var(--accent-gold);
  }

  .achievements-unlocked h3 {
    margin: 0 0 var(--spacing-md);
    font-size: 16px;
    color: var(--accent-gold);
  }

  .achievement-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .achievement-card {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }

  .ach-icon {
    font-size: 32px;
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    border-radius: 50%;
  }

  .ach-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .ach-name {
    font-weight: 600;
    font-size: 14px;
  }

  .ach-xp {
    color: var(--accent-gold);
    font-size: 13px;
  }
</style>
