<script>
  import { onMount, onDestroy } from 'svelte';
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';
  import ProgressBar from '../lib/components/ProgressBar.svelte';
  import { playClick, playCorrect, playWrong, playSprintPassed, playSprintFailed, playLevelUp, playXP, playAchievement } from '../lib/audio.js';

  // View states: 'list' | 'taking' | 'results'
  let view = 'list';

  // Domain data
  let domains = [];

  // Project/Sprint data
  let projects = [];
  let sprints = [];
  let selectedProject = null;

  // Exam taking state
  let currentSprint = null;
  let questions = [];
  let currentQuestionIndex = 0;
  let answers = [];           // For single: 'A', for multi: ['A', 'C']
  let selectedAnswer = null;  // For single: 'A', for multi: ['A', 'C']
  let timeElapsed = 0;
  let timerInterval = null;

  // Voice/TTS state
  let isSpeaking = false;

  // Results state
  let result = null;
  let hints = [];
  let explanations = [];

  // Typewriter animation state
  let typewriterText = '';
  let typewriterInterval = null;
  let showFullQuestion = true;
  let optionsVisible = [false, false, false, false];

  async function loadProjects() {
    if (window.go?.main?.App?.GetProjects) {
      projects = await window.go.main.App.GetProjects();
    }
  }

  // Stop speech and cleanup exam state
  async function stopExam() {
    // Stop timer
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    // Stop typewriter
    if (typewriterInterval) {
      clearInterval(typewriterInterval);
      typewriterInterval = null;
    }
    // Stop voice
    if (isSpeaking && window.go?.main?.App?.StopSpeech) {
      try {
        await window.go.main.App.StopSpeech();
      } catch (e) {
        console.warn('StopSpeech error:', e);
      }
    }
    isSpeaking = false;
  }

  async function selectProject(project) {
    selectedProject = project;
    if (window.go?.main?.App?.SetActiveProject) {
      await window.go.main.App.SetActiveProject(project.id);
    }
    if (window.go?.main?.App?.GetSprints) {
      sprints = await window.go.main.App.GetSprints();
    }
    if (window.go?.main?.App?.GetDomains) {
      domains = await window.go.main.App.GetDomains() || [];
    }
  }

  async function startSprint(sprint) {
    currentSprint = sprint;
    if (window.go?.main?.App?.GetSprintQuestions) {
      questions = await window.go.main.App.GetSprintQuestions(sprint.sprint_number);
    }

    // Initialize answers array based on question type
    answers = questions.map(q => q.type === 'multi' ? [] : null);
    currentQuestionIndex = 0;
    selectedAnswer = answers[0];
    timeElapsed = 0;
    result = null;
    hints = [];
    explanations = [];
    typewriterText = '';
    showFullQuestion = true;
    optionsVisible = [true, true, true, true]; // Show immediately in non-voice mode

    // Start timer
    timerInterval = setInterval(() => {
      timeElapsed++;
    }, 1000);

    view = 'taking';
  }

  function selectAnswerOption(optionIndex) {
    const letter = String.fromCharCode(65 + optionIndex); // 0->A, 1->B, etc.
    const q = questions[currentQuestionIndex];

    if (q?.type === 'multi') {
      // Multi-choice: toggle selection
      let current = Array.isArray(selectedAnswer) ? [...selectedAnswer] : [];
      const idx = current.indexOf(letter);
      if (idx >= 0) {
        current.splice(idx, 1);
      } else {
        current.push(letter);
        current.sort(); // Keep sorted for consistent comparison
      }
      selectedAnswer = current;
      answers[currentQuestionIndex] = current;
    } else {
      // Single choice: replace selection
      selectedAnswer = letter;
      answers[currentQuestionIndex] = letter;
    }
    playClick(); // Light feedback on selection
  }

  function nextQuestion() {
    if (currentQuestionIndex < questions.length - 1) {
      currentQuestionIndex++;
      selectedAnswer = answers[currentQuestionIndex];
    }
  }

  function prevQuestion() {
    if (currentQuestionIndex > 0) {
      currentQuestionIndex--;
      selectedAnswer = answers[currentQuestionIndex];
    }
  }

  function goToQuestion(index) {
    currentQuestionIndex = index;
    selectedAnswer = answers[currentQuestionIndex];
  }

  async function submitExam() {
    // Stop exam (timer, voice, etc.)
    await stopExam();

    // Submit answers - format multi-choice as comma-separated
    if (window.go?.main?.App?.SubmitSprintAnswers) {
      try {
        const formattedAnswers = answers.map(a => {
          if (Array.isArray(a)) {
            return a.join(','); // Multi-choice: ['A', 'C'] -> 'A,C'
          }
          return a || '';
        });
        result = await window.go.main.App.SubmitSprintAnswers(
          currentSprint.sprint_number,
          formattedAnswers
        );

        // Play audio feedback
        if (result.passed) {
          playSprintPassed();
          if (result.xp_earned > 0) {
            setTimeout(() => playXP(), 600);
          }
          // Play level up sound if domain leveled up
          if (result.domain_level_up) {
            setTimeout(() => playLevelUp(), 1000);
          }
          // Play achievement sound for each unlocked achievement
          if (result.unlocked_achievements?.length > 0) {
            setTimeout(() => playAchievement(), result.domain_level_up ? 1800 : 1200);
          }
        } else {
          playSprintFailed();
        }

        // Always load explanations for all questions (show after answering)
        if (window.go?.main?.App?.GetSprintExplanations) {
          explanations = await window.go.main.App.GetSprintExplanations(currentSprint.sprint_number) || [];
        }

        // Load hints for incorrect answers
        if (!result.passed && result.attempt_number >= 1) {
          if (window.go?.main?.App?.GetSprintHints) {
            hints = await window.go.main.App.GetSprintHints(currentSprint.sprint_number) || [];
          }
        }

        // Reload domains to show updated XP/levels
        if (window.go?.main?.App?.GetDomains) {
          domains = await window.go.main.App.GetDomains() || [];
        }

        view = 'results';
      } catch (err) {
        console.error('Submit failed:', err);
      }
    }
  }

  async function backToList() {
    // Stop exam first (voice, timer, etc.)
    await stopExam();

    view = 'list';
    currentSprint = null;
    questions = [];
    answers = [];
    result = null;

    // Reload sprints to get updated status
    if (selectedProject) {
      selectProject(selectedProject);
    }
  }

  function retakeSprint() {
    startSprint(currentSprint);
  }

  function formatTime(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  $: currentQuestion = questions[currentQuestionIndex];
  // Count answered: for single choice check !== null, for multi check array.length > 0
  $: answeredCount = answers.filter(a => {
    if (Array.isArray(a)) return a.length > 0;
    return a !== null;
  }).length;
  $: progressPercent = questions.length > 0 ? (answeredCount / questions.length) * 100 : 0;

  onMount(() => {
    loadProjects();
  });

  onDestroy(async () => {
    await stopExam();
  });
</script>

<div class="exams-page">
  {#if view === 'list'}
    <!-- PROJECT/SPRINT LIST VIEW -->
    <div class="list-layout">
      <div class="main-content">
        <h1 class="page-title">Exams</h1>

        <div class="project-selector">
      <label>Project:</label>
      <select on:change={(e) => {
        const project = projects.find(p => p.id === e.target.value);
        if (project) selectProject(project);
      }}>
        <option value="">Select a project...</option>
        {#each projects as project}
          <option value={project.id}>{project.name}</option>
        {/each}
      </select>
    </div>

    {#if selectedProject}
      <Card title="Sprints" subtitle={selectedProject.name}>
        <div class="sprints-list">
          {#each sprints as sprint}
            <div class="sprint-item" class:passed={sprint.status === 'passed'}>
              <div class="sprint-status">
                {#if sprint.status === 'passed'}
                  ✓
                {:else}
                  ○
                {/if}
              </div>
              <div class="sprint-info">
                <h4>Sprint {sprint.sprint_number}: {sprint.topic}</h4>
                <p>
                  {#if sprint.status === 'passed'}
                    Best: {sprint.best_score}% • {sprint.attempts} attempts
                  {:else if sprint.attempts > 0}
                    Best: {sprint.best_score}% • {sprint.attempts} attempts
                  {:else}
                    Not attempted
                  {/if}
                </p>
              </div>
              <div class="sprint-xp">
                {#if sprint.status === 'passed'}
                  <span class="xp earned">✓ {sprint.xp_earned}/{sprint.xp_available} XP</span>
                {:else}
                  <span class="xp">⭐ {sprint.xp_available} XP</span>
                {/if}
              </div>
              <Button
                variant={sprint.status === 'passed' ? 'secondary' : 'primary'}
                size="small"
                on:click={() => startSprint(sprint)}
                title={sprint.status === 'passed' ? 'Practice mode - no XP awarded' : ''}
              >
                {sprint.status === 'passed' ? 'Practice' : 'Start'}
              </Button>
            </div>
          {:else}
            <p class="empty">No sprints available for this project</p>
          {/each}
        </div>
      </Card>
    {:else}
      <Card>
        <p class="empty">Select a project to view sprints</p>
      </Card>
    {/if}
      </div>

      <!-- DOMAIN SIDEBAR -->
      {#if domains.length > 0}
        <aside class="domain-sidebar">
          <h3>Knowledge Domains</h3>
          {#each domains as domain}
            <div class="domain-card" style="--domain-color: {domain.color}">
              <div class="domain-header">
                <span class="domain-icon">{domain.icon}</span>
                <span class="domain-name">{domain.name}</span>
              </div>
              <div class="domain-level">
                <span class="level-badge">Lv.{domain.level}</span>
                <span class="level-title">{domain.level_title}</span>
              </div>
              <div class="domain-progress">
                <div class="xp-bar">
                  <div class="xp-fill" style="width: {domain.progress_pct}%"></div>
                </div>
                <span class="xp-text">{domain.earned_xp}/{domain.total_xp} XP</span>
              </div>
              <div class="domain-stats">
                <span>✓ {domain.sprints_passed}/{domain.sprints_total} sprints</span>
                {#if domain.sprints_perfect > 0}
                  <span>★ {domain.sprints_perfect} perfect</span>
                {/if}
              </div>
            </div>
          {/each}
        </aside>
      {/if}
    </div>

  {:else if view === 'taking'}
    <!-- EXAM TAKING VIEW -->
    <div class="exam-header">
      <div class="exam-title">
        <button class="exit-btn" on:click={backToList} title="Exit exam">✕</button>
        <h2>Sprint {currentSprint.sprint_number}: {currentSprint.topic}</h2>
        <span class="timer">⏱️ {formatTime(timeElapsed)}</span>
      </div>
      <div class="exam-progress">
        <ProgressBar value={progressPercent} />
        <span class="progress-text">{answeredCount}/{questions.length} answered</span>
      </div>
    </div>

    <div class="exam-content">
      {#if currentQuestion}
        <Card>
          <div class="question-header">
            <span class="question-number">Q{currentQuestion.number}</span>
            <span class="question-tier tier-{currentQuestion.tier.toLowerCase()}">{currentQuestion.tier}</span>
            <span class="question-xp">⭐ {currentQuestion.xp} XP</span>
          </div>

          <div class="question-text">
            <p>{currentQuestion.text}</p>

            {#if currentQuestion.code}
              <pre class="code-block"><code>{currentQuestion.code}</code></pre>
            {/if}
          </div>

          {#if currentQuestion.type === 'multi'}
            <div class="multi-hint">
              <span class="multi-icon">☑️</span>
              <span>Select ALL that apply</span>
            </div>
          {/if}

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
                    {letter}
                  {/if}
                </span>
                <span class="option-text">{option}</span>
              </button>
            {/each}
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
                class:answered={answers[i] !== null}
                on:click={() => goToQuestion(i)}
              >
                {i + 1}
              </button>
            {/each}
          </div>

          {#if currentQuestionIndex === questions.length - 1}
            <Button
              variant="primary"
              disabled={answeredCount < questions.length}
              on:click={submitExam}
            >
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
        <div class="results-icon">
          {#if result.passed}
            🎉
          {:else}
            💪
          {/if}
        </div>
        <h1>
          {#if result.passed}
            Sprint Passed!
          {:else}
            Keep Going!
          {/if}
        </h1>
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
              <div class="qr-indicator">
                {#if qr.correct}
                  ✓
                {:else}
                  ✗
                {/if}
              </div>
              <div class="qr-info">
                <span class="qr-number">Q{qr.question_num}</span>
                <span class="qr-answer">
                  Your answer: {qr.user_answer || '—'}
                  {#if !qr.correct}
                    <span class="correct-answer">(Correct: {qr.right_answer})</span>
                  {/if}
                </span>
              </div>
              <div class="qr-xp">
                {#if qr.correct}
                  +{qr.xp_earned} XP
                {/if}
              </div>
            </div>

            {#if !qr.correct && hints[i]}
              <div class="hint-box">
                <span class="hint-label">💡 Hint:</span>
                <span class="hint-text">{hints[i]}</span>
              </div>
            {/if}

            {#if explanations[i]}
              <div class="explanation-box" class:correct-explanation={qr.correct}>
                <span class="explanation-label">{qr.correct ? '✓' : '📖'} {qr.correct ? 'Why correct:' : 'Explanation:'}</span>
                <span class="explanation-text">{explanations[i]}</span>
              </div>
            {/if}
          {/each}
        </div>
      </Card>

      <!-- Domain Level Up Celebration -->
      {#if result.domain_level_up}
        <div class="domain-level-up">
          <span class="level-up-icon">⬆️</span>
          <div class="level-up-content">
            <h3>{result.domain_name} Level Up!</h3>
            <p class="new-level">Level {result.domain_new_level}: <span class="level-title">{result.domain_new_title}</span></p>
          </div>
        </div>
      {/if}

      <!-- Unlocked Achievements -->
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
          <Button variant="primary" on:click={retakeSprint}>
            Retry Sprint
          </Button>
        {/if}
        <Button variant="secondary" on:click={backToList}>
          Back to Sprints
        </Button>
      </div>

      {#if result.passed}
        <div class="celebration">
          <span class="confetti">🎊</span>
          <span class="confetti">🎉</span>
          <span class="confetti">⭐</span>
          <span class="confetti">🏆</span>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .exams-page {
    max-width: 1200px;
    margin: 0 auto;
  }

  .list-layout {
    display: grid;
    grid-template-columns: 1fr 280px;
    gap: var(--spacing-lg);
  }

  .main-content {
    min-width: 0;
  }

  /* Domain Sidebar */
  .domain-sidebar {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    padding: var(--spacing-md);
    height: fit-content;
    position: sticky;
    top: var(--spacing-lg);
  }

  .domain-sidebar h3 {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: var(--spacing-md);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .domain-card {
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    padding: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
    border-left: 3px solid var(--domain-color, var(--accent-primary));
  }

  .domain-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-xs);
  }

  .domain-icon {
    font-size: 16px;
  }

  .domain-name {
    font-weight: 600;
    font-size: 13px;
    color: var(--text-primary);
  }

  .domain-level {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-xs);
  }

  .level-badge {
    background: var(--domain-color, var(--accent-primary));
    color: white;
    font-size: 11px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .level-title {
    font-size: 12px;
    color: var(--text-secondary);
    font-style: italic;
  }

  .domain-progress {
    margin-bottom: var(--spacing-xs);
  }

  .xp-bar {
    height: 6px;
    background: var(--bg-card);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: 2px;
  }

  .xp-fill {
    height: 100%;
    background: var(--domain-color, var(--accent-primary));
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .xp-text {
    font-size: 10px;
    color: var(--text-tertiary);
  }

  .domain-stats {
    display: flex;
    gap: var(--spacing-sm);
    font-size: 10px;
    color: var(--text-tertiary);
  }

  @media (max-width: 900px) {
    .list-layout {
      grid-template-columns: 1fr;
    }

    .domain-sidebar {
      order: -1;
      display: flex;
      flex-wrap: wrap;
      gap: var(--spacing-sm);
    }

    .domain-sidebar h3 {
      width: 100%;
    }

    .domain-card {
      flex: 1;
      min-width: 200px;
    }
  }

  .page-title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: var(--spacing-lg);
  }

  /* Project Selector */
  .project-selector {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
  }

  .project-selector select {
    flex: 1;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-card);
    border: 1px solid var(--bg-tertiary);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: 14px;
  }

  /* Sprints List */
  .sprints-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .sprint-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    transition: transform 0.15s, box-shadow 0.15s;
  }

  .sprint-item:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
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

  .sprint-item.passed .sprint-status {
    background: var(--accent-green);
    color: white;
  }

  .sprint-info {
    flex: 1;
  }

  .sprint-info h4 {
    margin: 0 0 var(--spacing-xs);
    font-size: 15px;
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

  .sprint-xp .xp.earned {
    color: var(--success-color, #22c55e);
  }

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-xl);
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
    flex-shrink: 0;
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
    flex-shrink: 0;
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

  .question-xp {
    margin-left: auto;
    color: var(--accent-gold);
  }

  .question-text {
    margin-bottom: var(--spacing-lg);
  }

  .question-text p {
    font-size: 16px;
    line-height: 1.6;
    margin: 0;
  }

  .code-block {
    margin-top: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-primary);
    border-radius: var(--radius-md);
    overflow-x: auto;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 13px;
  }

  /* Multi-choice hint */
  .multi-hint {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--primary-900);
    border: 1px solid var(--primary-600);
    border-radius: var(--radius-md);
    margin-bottom: var(--spacing-md);
    font-size: 13px;
    color: var(--primary-300);
  }

  .multi-icon {
    font-size: 16px;
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
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    border-radius: 50%;
    font-weight: 600;
    font-size: 14px;
  }

  .option-btn.selected .option-letter {
    background: var(--primary-500);
    color: white;
  }

  /* Multi-choice checkbox styling */
  .option-btn.multi .option-letter {
    border-radius: var(--radius-sm);
    background: transparent;
    border: 2px solid var(--text-muted);
    font-size: 16px;
  }

  .option-btn.multi.selected .option-letter {
    background: var(--primary-500);
    border-color: var(--primary-500);
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

  .stat-box.xp .stat-value { color: var(--accent-gold); }
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
    border-left: 3px solid var(--success-color, #22c55e);
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

  .celebration {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    pointer-events: none;
    display: flex;
    justify-content: space-around;
    padding-top: 100px;
  }

  .confetti {
    font-size: 32px;
    animation: fall 2s ease-out forwards;
  }

  .confetti:nth-child(1) { animation-delay: 0s; }
  .confetti:nth-child(2) { animation-delay: 0.2s; }
  .confetti:nth-child(3) { animation-delay: 0.4s; }
  .confetti:nth-child(4) { animation-delay: 0.6s; }

  @keyframes fall {
    0% {
      transform: translateY(-100px) rotate(0deg);
      opacity: 1;
    }
    100% {
      transform: translateY(100vh) rotate(720deg);
      opacity: 0;
    }
  }

  /* Domain Level Up */
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
    animation: bounce 0.5s ease infinite;
  }

  @keyframes bounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
  }

  .level-up-content {
    flex: 1;
  }

  .level-up-content h3 {
    margin: 0;
    font-size: 20px;
    color: white;
  }

  .new-level {
    margin: var(--spacing-xs) 0 0;
    font-size: 16px;
    color: rgba(255, 255, 255, 0.9);
  }

  .level-title {
    font-weight: 700;
    color: var(--accent-gold);
  }

  /* Achievements Unlocked */
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
    animation: slideIn 0.3s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateX(-20px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
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
    font-weight: 500;
  }
</style>
