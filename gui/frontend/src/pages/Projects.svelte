<script>
  import { onMount } from 'svelte';
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';

  // Helper to add timeout to promises
  function withTimeout(promise, ms = 5000) {
    return Promise.race([
      promise,
      new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Request timeout')), ms)
      )
    ]);
  }

  // Domain data
  let domains = [];

  // Project/Sprint data
  let projects = [];
  let sprints = [];
  let selectedProject = null;

  // Project management state
  let showAddProjectModal = false;
  let newProjectPath = '';
  let addProjectError = '';
  let addingProject = false;
  let scanningProject = false;

  async function loadProjects() {
    console.log('loadProjects called');
    console.log('window.go available:', !!window.go);
    if (window.go?.main?.App?.GetProjects) {
      try {
        projects = await withTimeout(window.go.main.App.GetProjects(), 5000) || [];
        console.log('Projects loaded:', projects);
      } catch (err) {
        console.error('GetProjects error:', err);
        projects = [];
      }
    } else {
      console.warn('GetProjects not available');
      projects = [];
    }
  }

  async function addProject() {
    console.log('addProject called, path:', newProjectPath);
    if (!newProjectPath.trim()) {
      addProjectError = 'Please enter a project path';
      return;
    }
    addProjectError = '';
    addingProject = true;
    try {
      if (window.go?.main?.App?.AddProject) {
        console.log('Calling AddProject with:', newProjectPath.trim());
        const result = await withTimeout(window.go.main.App.AddProject(newProjectPath.trim()), 10000);
        console.log('AddProject result:', result);
        newProjectPath = '';
        showAddProjectModal = false;
        await loadProjects();
      } else {
        addProjectError = 'AddProject function not available - Wails bindings not loaded';
        console.error('AddProject not available');
      }
    } catch (err) {
      console.error('AddProject error:', err);
      addProjectError = err.message || 'Failed to add project';
    } finally {
      addingProject = false;
    }
  }

  async function removeProject(projectID) {
    if (!confirm('Remove this project? This will delete all exam progress for this project.')) {
      return;
    }
    try {
      if (window.go?.main?.App?.RemoveProject) {
        await withTimeout(window.go.main.App.RemoveProject(projectID), 5000);
        if (selectedProject?.id === projectID) {
          selectedProject = null;
          sprints = [];
          domains = [];
        }
        await loadProjects();
      }
    } catch (err) {
      console.error('Remove project failed:', err);
    }
  }

  async function scanAndImport() {
    console.log('scanAndImport called, selectedProject:', selectedProject);
    if (!selectedProject) {
      console.log('No project selected');
      return;
    }
    scanningProject = true;
    try {
      if (window.go?.main?.App?.ScanAndImportExams) {
        console.log('Calling ScanAndImportExams with:', selectedProject.id);
        const msg = await withTimeout(window.go.main.App.ScanAndImportExams(selectedProject.id), 30000);
        console.log('Scan result:', msg);
        // Refresh sprints and domains
        if (window.go?.main?.App?.GetSprints) {
          sprints = await withTimeout(window.go.main.App.GetSprints(), 5000) || [];
          console.log('Sprints loaded:', sprints);
        }
        if (window.go?.main?.App?.GetDomains) {
          domains = await withTimeout(window.go.main.App.GetDomains(), 5000) || [];
          console.log('Domains loaded:', domains);
        }
      } else {
        console.error('ScanAndImportExams not available');
      }
    } catch (err) {
      console.error('Scan failed:', err);
    } finally {
      scanningProject = false;
    }
  }

  async function selectProject(project) {
    console.log('selectProject called:', project?.id);
    selectedProject = project;
    sprints = [];
    domains = [];
    try {
      if (window.go?.main?.App?.SetActiveProject) {
        await withTimeout(window.go.main.App.SetActiveProject(project.id), 5000);
      }
      if (window.go?.main?.App?.GetSprints) {
        sprints = await withTimeout(window.go.main.App.GetSprints(), 5000) || [];
        console.log('GetSprints result:', sprints?.length);
      }
      if (window.go?.main?.App?.GetDomains) {
        domains = await withTimeout(window.go.main.App.GetDomains(), 5000) || [];
        console.log('GetDomains result:', domains?.length);
      }
    } catch (err) {
      console.error('selectProject error:', err);
    }
  }

  onMount(() => {
    console.log('Projects page mounted');
    console.log('window.go:', window.go);
    loadProjects();
  });
</script>

<div class="projects-page">
    <!-- PROJECT/SPRINT LIST VIEW -->
    <div class="list-layout">
      <div class="sidebar-panel">
        <div class="panel-header">
          <h3>Projects</h3>
          <Button variant="secondary" size="small" on:click={() => showAddProjectModal = true}>+ Add</Button>
        </div>
        <div class="project-list">
          {#each projects as project}
            <button
              class="project-item"
              class:selected={selectedProject?.id === project.id}
              on:click={() => selectProject(project)}
            >
              <span class="project-icon">📁</span>
              <span class="project-name">{project.name}</span>
              {#if selectedProject?.id === project.id}
                <button
                  class="delete-btn"
                  on:click|stopPropagation={() => removeProject(project.id)}
                  title="Remove project"
                >🗑️</button>
              {/if}
            </button>
          {:else}
            <p class="empty">No projects yet</p>
          {/each}
        </div>
      </div>

      <div class="main-content">
        <div class="content-header">
          <h1 class="page-title">{selectedProject ? selectedProject.name : 'Select a Project'}</h1>
          {#if selectedProject}
            <Button variant="secondary" size="small" on:click={scanAndImport} disabled={scanningProject} loading={scanningProject}>
              {scanningProject ? 'Scanning...' : '🔄 Scan for Exams'}
            </Button>
          {/if}
        </div>

        <!-- Add Project Modal -->
        {#if showAddProjectModal}
          <div class="modal-overlay">
            <button class="modal-backdrop" type="button" on:click={() => showAddProjectModal = false} aria-label="Close modal"></button>
            <div class="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
              <h3 id="modal-title">Add Project</h3>
              <p>Enter the path to a project directory containing exam files:</p>
              <input
                type="text"
                bind:value={newProjectPath}
                placeholder="/path/to/project or ~/project"
                on:keydown={(e) => e.key === 'Enter' && addProject()}
              />
              {#if addProjectError}
                <p class="error">{addProjectError}</p>
              {/if}
              <div class="modal-actions">
                <Button variant="secondary" on:click={() => showAddProjectModal = false} disabled={addingProject}>Cancel</Button>
                <Button variant="primary" on:click={addProject} disabled={addingProject} loading={addingProject}>
                  {addingProject ? 'Adding...' : 'Add Project'}
                </Button>
              </div>
            </div>
          </div>
        {/if}

        {#if selectedProject}
          <Card title="Sprints">
            <div class="sprints-list">
              {#each sprints as sprint}
                <div class="sprint-item" class:passed={sprint.status === 'passed'}>
                  <div class="sprint-status">
                    {#if sprint.status === 'passed'}✓{:else}○{/if}
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
                </div>
              {:else}
                <p class="empty">No sprints available. Click "Scan for Exams" to import.</p>
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
</div>

<style>
  .projects-page {
    max-width: 1200px;
    margin: 0 auto;
  }

  .list-layout {
    display: grid;
    grid-template-columns: 280px 1fr 280px;
    gap: var(--spacing-lg);
  }

  @media (max-width: 1000px) {
    .list-layout {
      grid-template-columns: 250px 1fr;
    }
  }

  /* Sidebar panel for projects */
  .sidebar-panel {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    padding: var(--spacing-md);
    height: fit-content;
    position: sticky;
    top: var(--spacing-lg);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-md);
  }

  .panel-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }

  .project-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
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
    width: 100%;
  }

  .project-item:hover {
    background: var(--bg-hover);
  }

  .project-item.selected {
    border-color: var(--primary-500);
    background: var(--primary-900);
  }

  .project-icon {
    font-size: 16px;
  }

  .project-name {
    flex: 1;
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .delete-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 12px;
    padding: 4px;
    border-radius: var(--radius-sm);
    opacity: 0.6;
    transition: all 0.15s;
  }

  .delete-btn:hover {
    opacity: 1;
    background: var(--accent-red);
  }

  .main-content {
    min-width: 0;
  }

  .content-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
  }

  /* Project management */
  .project-actions {
    display: flex;
    gap: var(--spacing-xs);
    margin-left: auto;
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-backdrop {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    border: none;
    cursor: pointer;
  }

  .modal {
    position: relative;
    z-index: 1;
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    min-width: 400px;
    max-width: 90vw;
  }

  .modal h3 {
    margin: 0 0 var(--spacing-sm);
    font-size: 18px;
  }

  .modal p {
    color: var(--text-secondary);
    margin-bottom: var(--spacing-md);
  }

  .modal input {
    width: 100%;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--bg-hover);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 14px;
    margin-bottom: var(--spacing-md);
  }

  .modal .error {
    color: var(--accent-red);
    font-size: 13px;
    margin-bottom: var(--spacing-sm);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
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
</style>
