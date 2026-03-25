<script>
  import { onMount, onDestroy } from 'svelte';
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';
  import ProgressBar from '../lib/components/ProgressBar.svelte';

  // View states: 'list', 'taking', 'results'
  let view = 'list';

  // Project & sprint selection
  let projects = [];
  let selectedProject = null;
  let sprints = [];

  // Exam state
  let currentSprint = null;
  let questions = [];
  let currentQuestionIndex = 0;
  let answers = [];
  let startTime = null;
  let elapsedSeconds = 0;
  let timerInterval = null;

  // Results
  let results = null;
  let hints = [];
  let explanations = [];

  // Hint tokens
  let hintTokens = 0;
  let usedHints = new Set();
  let revealedHints = {}; // questionNum -> hintText

  // Voice mode
  let voiceMode = false;
  let voiceStatus = { piper_available: false, moonshine_available: false, voice_enabled: false };
  let isSpeaking = false;
  let isListening = false;

  // Typewriter effect state
  let typewriterText = '';
  let typewriterTarget = '';
  let typewriterInterval = null;
  let showFullQuestion = false; // When true, skip typewriter and show everything

  async function loadProjects() {
    if (window.go?.main?.App?.GetProjects) {
      projects = await window.go.main.App.GetProjects() || [];
    }
    // Check voice status
    if (window.go?.main?.App?.GetVoiceStatus) {
      voiceStatus = await window.go.main.App.GetVoiceStatus() || voiceStatus;
    }
    // Load hint token balance
    await loadHintTokens();
  }

  async function loadHintTokens() {
    if (window.go?.main?.App?.GetHintTokenBalance) {
      const balance = await window.go.main.App.GetHintTokenBalance();
      if (balance) hintTokens = balance.tokens;
    }
  }

  async function selectProject(project) {
    console.log('selectProject called:', project);
    selectedProject = project;
    if (window.go?.main?.App?.SetActiveProject) {
      console.log('Setting active project:', project.id);
      await window.go.main.App.SetActiveProject(project.id);
    }
    if (window.go?.main?.App?.GetSprints) {
      sprints = await window.go.main.App.GetSprints() || [];
      console.log('Loaded sprints:', sprints);
    }
  }

  async function startSprint(sprint) {
    currentSprint = sprint;
    currentQuestionIndex = 0;
    hints = [];
    explanations = [];
    usedHints = new Set();
    revealedHints = {};
    typewriterText = '';
    typewriterTarget = '';
    showFullQuestion = !voiceMode; // Show immediately if not in voice mode

    // Load hint token balance and any previously used hints
    await loadHintTokens();
    if (window.go?.main?.App?.GetUsedHintsForSprint) {
      const used = await window.go.main.App.GetUsedHintsForSprint(sprint.sprint_number) || [];
      usedHints = new Set(used);
    }

    if (window.go?.main?.App?.GetSprintQuestions) {
      console.log('Getting questions for sprint:', sprint.sprint_number);
      questions = await window.go.main.App.GetSprintQuestions(sprint.sprint_number) || [];
      console.log('Loaded questions:', questions);
      // Initialize answers based on question type
      answers = questions.map(q => q.type === 'multi' ? [] : '');
    }

    startTime = Date.now();
    elapsedSeconds = 0;
    timerInterval = setInterval(() => {
      elapsedSeconds = Math.floor((Date.now() - startTime) / 1000);
    }, 1000);

    view = 'taking';

    // If voice mode is on, speak the first question (triggers typewriter)
    if (voiceMode && voiceStatus.piper_available) {
      await speakCurrentQuestion();
    }
  }

  function selectAnswerOption(optionIndex) {
    const letter = ['A', 'B', 'C', 'D'][optionIndex];
    const q = questions[currentQuestionIndex];

    if (q?.type === 'multi') {
      // Multi-choice: toggle selection
      let current = Array.isArray(answers[currentQuestionIndex]) ? [...answers[currentQuestionIndex]] : [];
      const idx = current.indexOf(letter);
      if (idx >= 0) {
        current.splice(idx, 1);
      } else {
        current.push(letter);
        current.sort();
      }
      answers[currentQuestionIndex] = current;
    } else {
      // Single choice
      answers[currentQuestionIndex] = letter;
    }
    answers = answers; // Trigger reactivity
  }

  async function nextQuestion() {
    if (currentQuestionIndex < questions.length - 1) {
      currentQuestionIndex++;
      resetTypewriter();
      if (voiceMode && voiceStatus.piper_available) {
        await speakCurrentQuestion();
      } else {
        showFullQuestion = true;
      }
    }
  }

  async function prevQuestion() {
    if (currentQuestionIndex > 0) {
      currentQuestionIndex--;
      resetTypewriter();
      if (voiceMode && voiceStatus.piper_available) {
        await speakCurrentQuestion();
      } else {
        showFullQuestion = true;
      }
    }
  }

  async function goToQuestion(index) {
    currentQuestionIndex = index;
    resetTypewriter();
    if (voiceMode && voiceStatus.piper_available) {
      await speakCurrentQuestion();
    } else {
      showFullQuestion = true;
    }
  }

  async function useHint(questionNum) {
    if (!window.go?.main?.App?.UseHintToken) return;
    try {
      const hintText = await window.go.main.App.UseHintToken(currentSprint.sprint_number, questionNum);
      revealedHints[questionNum] = hintText;
      revealedHints = revealedHints; // Trigger reactivity
      usedHints.add(questionNum);
      usedHints = new Set(usedHints); // Trigger reactivity
      hintTokens = Math.max(0, hintTokens - 1);

      // Speak hint in voice mode
      if (voiceMode && voiceStatus.piper_available) {
        await speakText('Hint: ' + hintText);
      }
    } catch (err) {
      console.error('Use hint error:', err);
    }
  }

  function resetTypewriter() {
    if (typewriterInterval) {
      clearInterval(typewriterInterval);
      typewriterInterval = null;
    }
    typewriterText = '';
    typewriterTarget = '';
    showFullQuestion = !voiceMode;
  }

  async function submitExam() {
    clearInterval(timerInterval);

    if (window.go?.main?.App?.SubmitSprintAnswers) {
      // Format answers: multi-choice as comma-separated
      const formattedAnswers = answers.map(a => {
        if (Array.isArray(a)) return a.join(',');
        return a || '';
      });
      results = await window.go.main.App.SubmitSprintAnswers(currentSprint.sprint_number, formattedAnswers);

      // Load hints after first failed attempt
      if (!results.passed && results.attempt_number >= 1) {
        if (window.go?.main?.App?.GetSprintHints) {
          hints = await window.go.main.App.GetSprintHints(currentSprint.sprint_number) || [];
        }
      }

      // Load explanations after second failed attempt
      if (!results.passed && results.attempt_number >= 2) {
        if (window.go?.main?.App?.GetSprintExplanations) {
          explanations = await window.go.main.App.GetSprintExplanations(currentSprint.sprint_number) || [];
        }
      }

      view = 'results';

      // Voice feedback for results
      if (voiceMode && voiceStatus.piper_available) {
        if (window.go?.main?.App?.SpeakSprintResult) {
          await window.go.main.App.SpeakSprintResult(results.passed, results.score_percent, results.xp_earned);
        }
      }
    }
  }

  function retryExam() {
    startSprint(currentSprint);
  }

  function backToList() {
    view = 'list';
    clearInterval(timerInterval);
    currentSprint = null;
    results = null;
    selectedProject = null;
    sprints = [];
  }

  function backToSprints() {
    view = 'list';
    clearInterval(timerInterval);
    currentSprint = null;
    results = null;
  }

  function formatTime(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  // Voice mode functions
  async function toggleVoiceMode() {
    voiceMode = !voiceMode;
    if (voiceMode && view === 'taking' && voiceStatus.piper_available) {
      await speakCurrentQuestion();
    }
  }

  async function speakCurrentQuestion() {
    if (!voiceMode || !voiceStatus.piper_available || !currentSprint) {
      showFullQuestion = true;
      return;
    }

    isSpeaking = true;
    showFullQuestion = false;
    typewriterText = '';

    try {
      // Get speech data for timing
      let speechData = null;
      if (window.go?.main?.App?.GetQuestionSpeechData) {
        speechData = await window.go.main.App.GetQuestionSpeechData(currentSprint.sprint_number, currentQuestionIndex);
      }

      if (speechData) {
        typewriterTarget = speechData.text;
        const totalChars = speechData.text.length;
        const msPerChar = speechData.estimated_ms / totalChars;

        // Start typewriter effect
        let charIndex = 0;
        if (typewriterInterval) clearInterval(typewriterInterval);
        typewriterInterval = setInterval(() => {
          if (charIndex < totalChars) {
            typewriterText = speechData.text.substring(0, charIndex + 1);
            charIndex++;
          } else {
            clearInterval(typewriterInterval);
            typewriterInterval = null;
            showFullQuestion = true;
          }
        }, msPerChar);
      }

      // Start speaking (runs in parallel with typewriter)
      if (window.go?.main?.App?.SpeakQuestion) {
        await window.go.main.App.SpeakQuestion(currentSprint.sprint_number, currentQuestionIndex);
      }
    } catch (err) {
      console.error('Speech error:', err);
      showFullQuestion = true;
    } finally {
      isSpeaking = false;
      // Ensure full question is shown after speech ends
      if (typewriterInterval) {
        clearInterval(typewriterInterval);
        typewriterInterval = null;
      }
      showFullQuestion = true;
    }
  }

  async function stopSpeech() {
    if (window.go?.main?.App?.StopSpeech) {
      await window.go.main.App.StopSpeech();
    }
    isSpeaking = false;
    // Stop typewriter and show full question
    if (typewriterInterval) {
      clearInterval(typewriterInterval);
      typewriterInterval = null;
    }
    showFullQuestion = true;
  }

  async function speakText(text) {
    if (!voiceMode || !voiceStatus.piper_available) return;

    isSpeaking = true;
    try {
      if (window.go?.main?.App?.SpeakBlocking) {
        await window.go.main.App.SpeakBlocking(text);
      }
    } catch (err) {
      console.error('Speech error:', err);
    } finally {
      isSpeaking = false;
    }
  }

  async function listenForAnswer() {
    if (!voiceStatus.moonshine_available) return;

    isListening = true;
    try {
      if (window.go?.main?.App?.Listen) {
        const transcription = await window.go.main.App.Listen();
        // Parse transcription for answer (A, B, C, D)
        const answer = parseVoiceAnswer(transcription);
        if (answer) {
          selectAnswerOption(['A', 'B', 'C', 'D'].indexOf(answer));
          await speakText(`You selected ${answer}`);
        }
      }
    } catch (err) {
      console.error('Listen error:', err);
    } finally {
      isListening = false;
    }
  }

  function parseVoiceAnswer(text) {
    const upper = text.toUpperCase().trim();
    // Direct letter
    if (['A', 'B', 'C', 'D'].includes(upper)) return upper;
    // "Option A", "Answer A", etc.
    const match = upper.match(/\b([ABCD])\b/);
    if (match) return match[1];
    // Phonetic alphabet
    if (upper.includes('ALPHA') || upper.includes('ALFA')) return 'A';
    if (upper.includes('BRAVO') || upper.includes('BEE')) return 'B';
    if (upper.includes('CHARLIE') || upper.includes('SEE')) return 'C';
    if (upper.includes('DELTA') || upper.includes('DEE')) return 'D';
    // Numbers
    if (upper.includes('ONE') || upper.includes('FIRST') || upper === '1') return 'A';
    if (upper.includes('TWO') || upper.includes('SECOND') || upper === '2') return 'B';
    if (upper.includes('THREE') || upper.includes('THIRD') || upper === '3') return 'C';
    if (upper.includes('FOUR') || upper.includes('FOURTH') || upper === '4') return 'D';
    return null;
  }

  // Cleanup on destroy
  onDestroy(() => {
    if (timerInterval) clearInterval(timerInterval);
    if (typewriterInterval) clearInterval(typewriterInterval);
    stopSpeech();
  });

  $: currentQuestion = questions[currentQuestionIndex];
  $: progress = questions.length > 0 ? ((currentQuestionIndex + 1) / questions.length) * 100 : 0;
  // Count answered: for multi-choice check array length, for single check non-empty
  $: answeredCount = answers.filter(a => {
    if (Array.isArray(a)) return a.length > 0;
    return a !== '';
  }).length;

  onMount(loadProjects);
</script>

<div class="take-exam-page">
  {#if view === 'list'}
    <h1 class="page-title">Take Exam</h1>
    <p class="page-subtitle">Select a project and sprint to test your knowledge</p>

    {#if !selectedProject}
      <Card title="Select Project">
        <div class="project-grid">
          {#each projects as project}
            <button class="project-card" on:click={() => selectProject(project)}>
              <span class="project-icon">📁</span>
              <span class="project-name">{project.name}</span>
              <span class="project-meta">{project.sprint_count || 0} sprints</span>
            </button>
          {:else}
            <p class="empty">No projects with exams. Go to Gen Exams to add projects.</p>
          {/each}
        </div>
      </Card>
    {:else}
      <div class="breadcrumb">
        <button on:click={backToList}>Projects</button>
        <span>/</span>
        <span>{selectedProject.name}</span>
      </div>

      {#if voiceStatus.piper_available}
        <div class="voice-mode-banner">
          <label class="voice-mode-toggle">
            <input type="checkbox" bind:checked={voiceMode} />
            <span class="toggle-slider"></span>
            <span class="toggle-label">🔊 Voice Mode</span>
          </label>
          <span class="voice-mode-hint">Questions will be read aloud</span>
        </div>
      {/if}

      <Card title="Sprints">
        <div class="sprints-list">
          {#each sprints as sprint}
            <div class="sprint-item" class:passed={sprint.status === 'passed'}>
              <div class="sprint-status">
                {#if sprint.status === 'passed'}
                  <span class="status-icon passed">✓</span>
                {:else}
                  <span class="status-icon pending">○</span>
                {/if}
              </div>
              <div class="sprint-info">
                <span class="sprint-title">Sprint {sprint.sprint_number}: {sprint.topic}</span>
                <span class="sprint-meta">
                  {#if sprint.best_score != null}
                    Best: {sprint.best_score}% · {sprint.attempts} attempts
                  {:else}
                    Not attempted
                  {/if}
                </span>
              </div>
              <div class="sprint-xp">
                <span class="xp-available">{sprint.xp_available} XP</span>
              </div>
              <Button
                size="small"
                variant={sprint.status === 'passed' ? 'secondary' : 'primary'}
                on:click={() => startSprint(sprint)}
              >
                {sprint.status === 'passed' ? 'Retake' : 'Start'}
              </Button>
            </div>
          {:else}
            <p class="empty">No sprints found for this project.</p>
          {/each}
        </div>
      </Card>
    {/if}

  {:else if view === 'taking'}
    <div class="exam-header">
      <div class="exam-info">
        <h2>Sprint {currentSprint.sprint_number}: {currentSprint.topic}</h2>
        <div class="exam-meta">
          <span class="timer">⏱️ {formatTime(elapsedSeconds)}</span>
          {#if hintTokens > 0}
            <span class="hint-token-badge">💡 {hintTokens}</span>
          {/if}
        </div>
      </div>
      <div class="exam-controls">
        {#if voiceStatus.piper_available}
          <button
            class="voice-toggle"
            class:active={voiceMode}
            on:click={toggleVoiceMode}
            title={voiceMode ? 'Disable voice mode' : 'Enable voice mode'}
          >
            {voiceMode ? '🔊' : '🔇'}
          </button>
          {#if voiceMode && isSpeaking}
            <button class="voice-stop" on:click={stopSpeech} title="Stop speaking">⏹️</button>
          {/if}
          {#if voiceMode && !isSpeaking}
            <button class="voice-replay" on:click={speakCurrentQuestion} title="Replay question">🔁</button>
          {/if}
          {#if voiceStatus.moonshine_available && voiceMode}
            <button
              class="voice-listen"
              class:listening={isListening}
              on:click={listenForAnswer}
              disabled={isListening}
              title="Speak your answer"
            >
              🎤
            </button>
          {/if}
        {/if}
      </div>
      <div class="exam-progress">
        <span>Question {currentQuestionIndex + 1} of {questions.length}</span>
        <ProgressBar value={progress} max={100} size="small" />
      </div>
    </div>

    <div class="question-dots">
      {#each questions as _, i}
        <button
          class="dot"
          class:current={i === currentQuestionIndex}
          class:answered={answers[i] !== ''}
          on:click={() => goToQuestion(i)}
        >
          {i + 1}
        </button>
      {/each}
    </div>

    {#if currentQuestion}
      <Card>
        <div class="question">
          <div class="question-header">
            <span class="question-tier">{currentQuestion.tier}</span>
            <span class="question-xp">{currentQuestion.xp} XP</span>
          </div>

          {#if voiceMode && !showFullQuestion && typewriterText}
            <!-- Typewriter mode: show synced text -->
            <div class="typewriter-container">
              <p class="typewriter-text">{typewriterText}<span class="cursor">|</span></p>
            </div>
            <!-- Show code immediately when speaking mentions it -->
            {#if currentQuestion.code && typewriterText.includes('code snippet')}
              <pre class="question-code">{currentQuestion.code}</pre>
            {/if}
          {:else}
            <!-- Normal mode: show full question -->
            <p class="question-text">{currentQuestion.text}</p>

            {#if currentQuestion.code}
              <pre class="question-code">{currentQuestion.code}</pre>
            {/if}

            <!-- Hint token: reveal hint on demand -->
            {#if revealedHints[currentQuestionIndex + 1]}
              <div class="active-hint">
                <span class="active-hint-icon">💡</span>
                <span>{revealedHints[currentQuestionIndex + 1]}</span>
              </div>
            {:else if hintTokens > 0 && !usedHints.has(currentQuestionIndex + 1)}
              <button class="use-hint-btn" on:click={() => useHint(currentQuestionIndex + 1)}>
                💡 Use Hint <span class="hint-cost">(1 token)</span>
              </button>
            {/if}

            {#if currentQuestion.type === 'multi'}
              <div class="multi-hint">
                <span class="multi-icon">☑️</span>
                <span>Select ALL that apply</span>
              </div>
            {/if}

            <div class="options">
              {#each currentQuestion.options as option, i}
                {@const letter = ['A', 'B', 'C', 'D'][i]}
                {@const isSelected = currentQuestion.type === 'multi'
                  ? Array.isArray(answers[currentQuestionIndex]) && answers[currentQuestionIndex].includes(letter)
                  : answers[currentQuestionIndex] === letter}
                <button
                  class="option"
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
          {/if}
        </div>
      </Card>

      <div class="exam-nav">
        <Button variant="secondary" on:click={prevQuestion} disabled={currentQuestionIndex === 0}>
          ← Previous
        </Button>

        {#if currentQuestionIndex === questions.length - 1}
          <Button on:click={submitExam} disabled={answeredCount < questions.length}>
            Submit ({answeredCount}/{questions.length})
          </Button>
        {:else}
          <Button on:click={nextQuestion}>
            Next →
          </Button>
        {/if}
      </div>
    {:else}
      <Card title="Loading Error">
        <p class="empty">No questions loaded for this sprint.</p>
        <p class="empty">Questions: {questions.length}, Index: {currentQuestionIndex}</p>
        <div style="margin-top: 1rem; text-align: center;">
          <Button variant="secondary" on:click={backToSprints}>Back to Sprints</Button>
        </div>
      </Card>
    {/if}

  {:else if view === 'results'}
    <div class="results-page">
      <div class="results-header" class:passed={results.passed}>
        <span class="results-emoji">{results.passed ? '🎉' : '📚'}</span>
        <h2>{results.passed ? 'Sprint Passed!' : 'Keep Learning!'}</h2>
        <p class="results-topic">Sprint {results.sprint_num}: {results.topic}</p>
      </div>

      <div class="results-stats">
        <div class="stat">
          <span class="stat-value">{results.score_percent}%</span>
          <span class="stat-label">Score</span>
        </div>
        <div class="stat">
          <span class="stat-value">{results.correct_count}/{results.total_questions}</span>
          <span class="stat-label">Correct</span>
        </div>
        <div class="stat">
          <span class="stat-value">+{results.xp_earned}</span>
          <span class="stat-label">XP</span>
        </div>
        <div class="stat">
          <span class="stat-value">+{results.coins_earned}</span>
          <span class="stat-label">Coins</span>
        </div>
      </div>

      <Card title="Question Breakdown">
        <div class="question-results">
          {#each results.question_results as qr, i}
            <div class="question-result" class:correct={qr.correct}>
              <div class="qr-header">
                <span class="qr-number">Q{qr.question_num}</span>
                <span class="qr-status">{qr.correct ? '✓' : '✗'}</span>
              </div>
              <div class="qr-answers">
                <span>Your answer: <strong>{qr.user_answer || '—'}</strong></span>
                {#if !qr.correct}
                  <span>Correct: <strong class="correct-answer">{qr.right_answer}</strong></span>
                {/if}
              </div>
              {#if hints[i] && !qr.correct}
                <div class="qr-hint">
                  <strong>Hint:</strong> {hints[i]}
                </div>
              {/if}
              {#if explanations[i] && !qr.correct}
                <div class="qr-explanation">
                  <strong>Explanation:</strong> {explanations[i]}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </Card>

      <div class="results-actions">
        {#if !results.passed}
          <Button on:click={retryExam}>Try Again</Button>
        {/if}
        <Button variant="secondary" on:click={backToSprints}>Back to Sprints</Button>
      </div>
    </div>
  {/if}
</div>

<style>
  .take-exam-page {
    max-width: 900px;
    margin: 0 auto;
  }

  .page-title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: var(--spacing-xs);
  }

  .page-subtitle {
    color: var(--text-muted);
    margin-bottom: var(--spacing-lg);
  }

  .breadcrumb {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-lg);
    color: var(--text-muted);
  }

  .breadcrumb button {
    background: none;
    border: none;
    color: var(--primary-400);
    cursor: pointer;
    padding: 0;
  }

  .breadcrumb button:hover {
    text-decoration: underline;
  }

  .project-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: var(--spacing-md);
  }

  .project-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-lg);
    background: var(--bg-tertiary);
    border: 2px solid transparent;
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: all 0.15s;
  }

  .project-card:hover {
    border-color: var(--primary-500);
    background: var(--bg-hover);
  }

  .project-icon {
    font-size: 32px;
  }

  .project-name {
    font-weight: 600;
    color: var(--text-primary);
  }

  .project-meta {
    font-size: 12px;
    color: var(--text-muted);
  }

  .sprints-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .sprint-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }

  .sprint-item.passed {
    opacity: 0.8;
  }

  .sprint-status {
    width: 32px;
    text-align: center;
  }

  .status-icon {
    font-size: 20px;
  }

  .status-icon.passed {
    color: var(--accent-green);
  }

  .status-icon.pending {
    color: var(--text-muted);
  }

  .sprint-info {
    flex: 1;
  }

  .sprint-title {
    display: block;
    font-weight: 600;
  }

  .sprint-meta {
    font-size: 12px;
    color: var(--text-muted);
  }

  .sprint-xp {
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--bg-card);
    border-radius: var(--radius-sm);
  }

  .xp-available {
    color: var(--primary-400);
    font-weight: 600;
    font-size: 12px;
  }

  /* Exam taking */
  .exam-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
  }

  .exam-info h2 {
    margin: 0;
    font-size: 18px;
  }

  .timer {
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }

  .exam-progress {
    text-align: right;
  }

  .exam-progress span {
    font-size: 12px;
    color: var(--text-muted);
  }

  .question-dots {
    display: flex;
    justify-content: center;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-lg);
  }

  .dot {
    width: 32px;
    height: 32px;
    border: 2px solid var(--bg-tertiary);
    background: var(--bg-card);
    border-radius: 50%;
    cursor: pointer;
    font-size: 12px;
    color: var(--text-muted);
    transition: all 0.15s;
  }

  .dot.current {
    border-color: var(--primary-500);
    color: var(--text-primary);
  }

  .dot.answered {
    background: var(--primary-600);
    color: white;
    border-color: var(--primary-600);
  }

  .question {
    padding: var(--spacing-md);
  }

  .question-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: var(--spacing-md);
  }

  .question-tier {
    background: var(--bg-tertiary);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    font-size: 12px;
    color: var(--text-secondary);
  }

  .question-xp {
    color: var(--primary-400);
    font-weight: 600;
  }

  .question-text {
    font-size: 16px;
    line-height: 1.6;
    margin-bottom: var(--spacing-lg);
  }

  /* Typewriter effect */
  .typewriter-container {
    min-height: 200px;
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    margin-bottom: var(--spacing-lg);
  }

  .typewriter-text {
    font-size: 18px;
    line-height: 1.8;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .cursor {
    animation: blink 0.7s infinite;
    color: var(--primary-400);
    font-weight: bold;
  }

  @keyframes blink {
    0%, 50% { opacity: 1; }
    51%, 100% { opacity: 0; }
  }

  .question-code {
    margin-bottom: var(--spacing-lg);
  }

  .options {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .option {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border: 2px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    color: var(--text-primary);
    transition: all 0.15s;
  }

  .option:hover {
    background: var(--bg-hover);
  }

  .option.selected {
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

  .option.selected .option-letter {
    background: var(--primary-500);
    color: white;
  }

  /* Multi-choice styling */
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

  .option.multi .option-letter {
    border-radius: var(--radius-sm);
    background: transparent;
    border: 2px solid var(--text-muted);
    font-size: 16px;
  }

  .option.multi.selected .option-letter {
    background: var(--primary-500);
    border-color: var(--primary-500);
    color: white;
  }

  .option-text {
    flex: 1;
  }

  .exam-nav {
    display: flex;
    justify-content: space-between;
    margin-top: var(--spacing-lg);
  }

  /* Results */
  .results-page {
    max-width: 700px;
    margin: 0 auto;
  }

  .results-header {
    text-align: center;
    padding: var(--spacing-xl);
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    margin-bottom: var(--spacing-lg);
  }

  .results-header.passed {
    background: linear-gradient(135deg, var(--accent-green), var(--primary-600));
  }

  .results-emoji {
    font-size: 48px;
    display: block;
    margin-bottom: var(--spacing-sm);
  }

  .results-header h2 {
    margin: 0;
    font-size: 24px;
  }

  .results-topic {
    margin-top: var(--spacing-sm);
    opacity: 0.8;
  }

  .results-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
  }

  .stat {
    text-align: center;
    padding: var(--spacing-md);
    background: var(--bg-card);
    border-radius: var(--radius-md);
  }

  .stat-value {
    display: block;
    font-size: 24px;
    font-weight: 700;
    color: var(--primary-400);
  }

  .stat-label {
    font-size: 12px;
    color: var(--text-muted);
  }

  .question-results {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .question-result {
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border-left: 4px solid var(--accent-red);
  }

  .question-result.correct {
    border-left-color: var(--accent-green);
  }

  .qr-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: var(--spacing-sm);
  }

  .qr-number {
    font-weight: 600;
  }

  .qr-status {
    font-size: 18px;
  }

  .question-result.correct .qr-status {
    color: var(--accent-green);
  }

  .question-result:not(.correct) .qr-status {
    color: var(--accent-red);
  }

  .qr-answers {
    display: flex;
    gap: var(--spacing-lg);
    font-size: 14px;
    color: var(--text-secondary);
  }

  .correct-answer {
    color: var(--accent-green);
  }

  .qr-hint, .qr-explanation {
    margin-top: var(--spacing-sm);
    padding: var(--spacing-sm);
    background: var(--bg-card);
    border-radius: var(--radius-sm);
    font-size: 13px;
  }

  .qr-hint {
    border-left: 3px solid var(--accent-gold);
  }

  .qr-explanation {
    border-left: 3px solid var(--accent-blue);
  }

  .results-actions {
    display: flex;
    justify-content: center;
    gap: var(--spacing-md);
    margin-top: var(--spacing-lg);
  }

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-xl);
  }

  /* Voice mode banner */
  .voice-mode-banner {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-card);
    border-radius: var(--radius-md);
    margin-bottom: var(--spacing-md);
  }

  .voice-mode-toggle {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    cursor: pointer;
  }

  .voice-mode-toggle input {
    display: none;
  }

  .toggle-slider {
    width: 44px;
    height: 24px;
    background: var(--bg-tertiary);
    border-radius: 12px;
    position: relative;
    transition: background 0.2s;
  }

  .toggle-slider::after {
    content: '';
    position: absolute;
    width: 20px;
    height: 20px;
    background: white;
    border-radius: 50%;
    top: 2px;
    left: 2px;
    transition: transform 0.2s;
  }

  .voice-mode-toggle input:checked + .toggle-slider {
    background: var(--primary-500);
  }

  .voice-mode-toggle input:checked + .toggle-slider::after {
    transform: translateX(20px);
  }

  .toggle-label {
    font-weight: 600;
  }

  .voice-mode-hint {
    font-size: 12px;
    color: var(--text-muted);
  }

  /* Voice controls */
  .exam-controls {
    display: flex;
    gap: var(--spacing-xs);
    align-items: center;
  }

  .voice-toggle,
  .voice-stop,
  .voice-replay,
  .voice-listen {
    width: 36px;
    height: 36px;
    border: none;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .voice-toggle:hover,
  .voice-stop:hover,
  .voice-replay:hover,
  .voice-listen:hover {
    background: var(--bg-hover);
  }

  .voice-toggle.active {
    background: var(--primary-600);
  }

  .voice-stop {
    background: var(--accent-red);
  }

  .voice-listen.listening {
    background: var(--accent-red);
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .voice-status {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: 12px;
    color: var(--text-muted);
    margin-left: auto;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }

  .voice-status.speaking {
    color: var(--primary-400);
  }

  .voice-status.listening {
    color: var(--accent-red);
  }

  /* Hint tokens in exam */
  .exam-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .hint-token-badge {
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--primary-900);
    border: 1px solid var(--primary-600);
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 600;
    color: var(--primary-400);
  }

  .use-hint-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-tertiary);
    border: 1px dashed var(--primary-500);
    border-radius: var(--radius-md);
    color: var(--primary-400);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s;
    margin-bottom: var(--spacing-md);
  }

  .use-hint-btn:hover {
    background: var(--primary-900);
    border-style: solid;
  }

  .hint-cost {
    color: var(--text-muted);
    font-size: 11px;
  }

  .active-hint {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--primary-900);
    border-left: 3px solid var(--primary-500);
    border-radius: var(--radius-md);
    margin-bottom: var(--spacing-md);
    font-size: 14px;
    color: var(--primary-300);
    animation: hintReveal 0.3s ease;
  }

  .active-hint-icon {
    flex-shrink: 0;
    font-size: 16px;
  }

  @keyframes hintReveal {
    from { opacity: 0; transform: translateY(-8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
