<script>
  import { onMount } from 'svelte';
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';

  let projects = [];
  let newProjectPath = '';
  let loading = false;
  let message = '';

  async function loadProjects() {
    if (window.go?.main?.App?.GetProjects) {
      projects = await window.go.main.App.GetProjects() || [];
    }
  }

  async function addProject() {
    if (!newProjectPath.trim()) return;
    loading = true;
    message = '';

    try {
      if (window.go?.main?.App?.AddProject) {
        await window.go.main.App.AddProject(newProjectPath.trim());
        newProjectPath = '';
        await loadProjects();
        message = 'Project added successfully!';
      }
    } catch (err) {
      message = 'Error: ' + err.message;
    }
    loading = false;
  }

  async function scanForExams(projectId) {
    loading = true;
    message = '';

    try {
      if (window.go?.main?.App?.ScanAndImportExams) {
        const result = await window.go.main.App.ScanAndImportExams(projectId);
        message = result || 'Scan complete!';
        await loadProjects();
      }
    } catch (err) {
      message = 'Error: ' + err.message;
    }
    loading = false;
  }

  async function generateExam(projectId) {
    loading = true;
    message = '';

    try {
      if (window.go?.main?.App?.GenerateExam) {
        const result = await window.go.main.App.GenerateExam(projectId);
        message = result || 'Exam generated!';
        await loadProjects();
      }
    } catch (err) {
      message = 'Error: ' + err.message;
    }
    loading = false;
  }

  async function removeProject(projectId, projectName) {
    if (!confirm(`Remove project "${projectName}"?\n\nThis will also delete all sprints and exam progress for this project.`)) {
      return;
    }

    loading = true;
    message = '';

    try {
      if (window.go?.main?.App?.RemoveProject) {
        await window.go.main.App.RemoveProject(projectId);
        await loadProjects();
        message = 'Project removed successfully!';
      }
    } catch (err) {
      message = 'Error: ' + (err.message || err);
    }
    loading = false;
  }

  onMount(loadProjects);
</script>

<div class="gen-exams-page">
  <h1 class="page-title">Generate Exams</h1>
  <p class="page-subtitle">Add project folders and generate exam questions from exam_*.md files</p>

  {#if message}
    <div class="message" class:error={message.startsWith('Error')}>
      {message}
    </div>
  {/if}

  <Card title="Add Project">
    <div class="add-project">
      <input
        type="text"
        bind:value={newProjectPath}
        placeholder="Enter project folder path (e.g., /home/user/myproject)"
        class="path-input"
        on:keydown={(e) => e.key === 'Enter' && addProject()}
      />
      <Button on:click={addProject} disabled={loading || !newProjectPath.trim()}>
        Add Project
      </Button>
    </div>
  </Card>

  <Card title="Projects ({projects.length})">
    <div class="projects-list">
      {#each projects as project}
        <div class="project-item">
          <div class="project-info">
            <span class="project-name">{project.name}</span>
            <span class="project-path">{project.path}</span>
            <span class="project-stats">
              {project.sprint_count || 0} sprints
              {#if project.has_exam_file}
                <span class="badge">exam_*.md found</span>
              {/if}
            </span>
          </div>
          <div class="project-actions">
            <Button size="small" variant="secondary" on:click={() => scanForExams(project.id)} disabled={loading}>
              Scan & Import
            </Button>
            <Button size="small" on:click={() => generateExam(project.id)} disabled={loading}>
              Generate Exam
            </Button>
            <button class="remove-btn" on:click={() => removeProject(project.id, project.name)} disabled={loading} title="Remove project">
              ✕
            </button>
          </div>
        </div>
      {:else}
        <p class="empty">No projects added yet. Add a project folder above.</p>
      {/each}
    </div>
  </Card>

  <Card title="How It Works">
    <div class="help-content">
      <ol>
        <li><strong>Add Project</strong> - Point to a folder containing your codebase</li>
        <li><strong>Scan & Import</strong> - Finds existing exam_*.md files and imports questions</li>
        <li><strong>Generate Exam</strong> - Analyzes code and creates new exam questions</li>
        <li>Go to <strong>Take Exam</strong> to practice and test your knowledge!</li>
      </ol>
    </div>
  </Card>
</div>

<style>
  .gen-exams-page {
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

  .message {
    padding: var(--spacing-md);
    background: var(--accent-green);
    color: white;
    border-radius: var(--radius-md);
    margin-bottom: var(--spacing-lg);
  }

  .message.error {
    background: var(--accent-red);
  }

  .add-project {
    display: flex;
    gap: var(--spacing-md);
  }

  .path-input {
    flex: 1;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--bg-hover);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: 14px;
  }

  .path-input:focus {
    outline: none;
    border-color: var(--primary-500);
  }

  .projects-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .project-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }

  .project-info {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .project-name {
    font-weight: 600;
    font-size: 16px;
  }

  .project-path {
    font-size: 12px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .project-stats {
    font-size: 12px;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .badge {
    background: var(--accent-green);
    color: white;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 10px;
  }

  .project-actions {
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
  }

  .remove-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    font-size: 14px;
  }

  .remove-btn:hover {
    background: var(--accent-red);
    color: white;
  }

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-xl);
  }

  .help-content ol {
    margin: 0;
    padding-left: var(--spacing-lg);
  }

  .help-content li {
    margin-bottom: var(--spacing-sm);
    color: var(--text-secondary);
  }

  .help-content strong {
    color: var(--text-primary);
  }
</style>
