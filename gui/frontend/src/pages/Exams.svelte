<script>
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';
  import ProgressBar from '../lib/components/ProgressBar.svelte';

  let projects = [];
  let sprints = [];
  let selectedProject = null;

  async function loadProjects() {
    if (window.go?.main?.App?.GetProjects) {
      projects = await window.go.main.App.GetProjects();
    }
  }

  async function selectProject(project) {
    selectedProject = project;
    if (window.go?.main?.App?.SetActiveProject) {
      await window.go.main.App.SetActiveProject(project.id);
    }
    if (window.go?.main?.App?.GetSprints) {
      sprints = await window.go.main.App.GetSprints();
    }
  }

  // Load on mount
  loadProjects();
</script>

<div class="exams-page">
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
    <Button variant="secondary" icon="📂">Import Exam</Button>
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
                {:else}
                  Not attempted
                {/if}
              </p>
            </div>
            <div class="sprint-xp">
              <span class="xp">⭐ {sprint.xp_available} XP</span>
            </div>
            <Button variant={sprint.status === 'passed' ? 'secondary' : 'primary'} size="small">
              {sprint.status === 'passed' ? 'Retake' : 'Start'}
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

<style>
  .exams-page {
    max-width: 900px;
    margin: 0 auto;
  }

  .page-title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: var(--spacing-lg);
  }

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

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-xl);
  }
</style>
