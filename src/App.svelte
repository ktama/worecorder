<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/api/dialog';
  import { invoke } from '@tauri-apps/api/tauri';

  interface RecordEntry {
    id: number;
    projectId: string;
    projectName: string;
    category: string;
    task: string;
    start: string;
    end?: string;
  }

  let projectId = '';
  let projectName = '';
  let category = '';
  let task = '';

  let records: RecordEntry[] = [];
  let selectedDate = new Date().toISOString().substring(0, 10);
  let projectFilter = '';
  let currentId = 0;
  let saveDir = '';
  let projectMap: Record<string, string> = {};

  interface EditEntry {
    id: number;
    projectId: string;
    projectName: string;
    category: string;
    task: string;
    start: string;
    end: string;
  }
  let editing: EditEntry | null = null;

  interface TotalEntry {
    projectId: string;
    projectName: string;
    category: string;
    ms: number;
    tasks: { name: string; ms: number }[];
  }

  function computeProjectMap() {
    projectMap = {};
    for (const r of records) {
      projectMap[r.projectId] = r.projectName;
    }
  }

  function onProjectIdInput() {
    if (projectMap[projectId]) {
      projectName = projectMap[projectId];
    }
  }

  function onEditProjectIdInput() {
    if (editing && projectMap[editing.projectId]) {
      editing.projectName = projectMap[editing.projectId];
    }
  }

  onMount(async () => {
    const stored = localStorage.getItem('records');
    if (stored) {
      records = JSON.parse(stored);
      currentId =
        records.reduce((max, r) => (r.id > max ? r.id : max), 0) + 1;
    }
    const dir = localStorage.getItem('saveDir');
    if (dir) {
      saveDir = dir;
      try {
        const loaded = await invoke<string>('load_records', { path: `${saveDir}/records.json` });
        if (loaded) {
          records = JSON.parse(loaded);
          currentId =
            records.reduce((max, r) => (r.id > max ? r.id : max), 0) + 1;
        }
      } catch (e) {
        console.error(e);
      }
    }
    computeProjectMap();
  });

  function save() {
    localStorage.setItem('records', JSON.stringify(records));
    if (saveDir) {
      invoke('save_records', {
        req: { path: `${saveDir}/records.json`, data: JSON.stringify(records) }
      }).catch((e) => console.error(e));
    }
  }

  function startRecord() {
    const now = new Date().toISOString();
    const ongoing = records.find((r) => !r.end);
    if (ongoing) {
      ongoing.end = now;
    }
    records = [
      ...records,
      {
        id: currentId++,
        projectId,
        projectName,
        category,
      task,
      start: now,
    },
    ];
    computeProjectMap();
    save();
  }

  function stopRecord() {
    const now = new Date().toISOString();
    const ongoing = records.find((r) => !r.end);
    if (ongoing) {
      ongoing.end = now;
      records = [...records];
      save();
    }
  }

  async function chooseDirectory() {
    const selected = await open({ directory: true });
    if (typeof selected === 'string') {
      saveDir = selected;
      localStorage.setItem('saveDir', saveDir);
      // load existing file if any
      try {
        const loaded = await invoke<string>('load_records', { path: `${saveDir}/records.json` });
        records = JSON.parse(loaded);
        currentId = records.reduce((m, r) => (r.id > m ? r.id : m), 0) + 1;
        computeProjectMap();
        save();
      } catch (e) {
        console.error(e);
      }
    }
  }

  function beginEdit(r: RecordEntry) {
    editing = {
      id: r.id,
      projectId: r.projectId,
      projectName: r.projectName,
      category: r.category,
      task: r.task,
      start: r.start.slice(0, 16),
      end: r.end ? r.end.slice(0, 16) : '',
    };
  }

  function saveEdit() {
    if (!editing) return;
    const idx = records.findIndex((r) => r.id === editing!.id);
    if (idx !== -1) {
      records[idx] = {
        id: editing.id,
        projectId: editing.projectId,
        projectName: editing.projectName,
        category: editing.category,
        task: editing.task,
        start: new Date(editing.start).toISOString(),
        end: editing.end ? new Date(editing.end).toISOString() : undefined,
      };
      records = [...records];
      computeProjectMap();
      save();
    }
    editing = null;
  }

  function cancelEdit() {
    editing = null;
  }

  function duration(start: string, end?: string) {
    if (!end) return 0;
    return new Date(end).getTime() - new Date(start).getTime();
  }

  function computeTotals(recs: RecordEntry[]): TotalEntry[] {
    const map = new Map<
      string,
      { projectId: string; projectName: string; category: string; ms: number; tasks: Map<string, number> }
    >();
    for (const r of recs) {
      if (!r.end) continue;
      const key = `${r.projectId}|${r.category}`;
      const ms = duration(r.start, r.end);
      if (map.has(key)) {
        const entry = map.get(key)!;
        entry.ms += ms;
        entry.tasks.set(r.task, (entry.tasks.get(r.task) || 0) + ms);
      } else {
        const t = new Map<string, number>();
        t.set(r.task, ms);
        map.set(key, {
          projectId: r.projectId,
          projectName: r.projectName,
          category: r.category,
          ms,
          tasks: t,
        });
      }
    }
    return Array.from(map.values()).map((v) => ({
      projectId: v.projectId,
      projectName: v.projectName,
      category: v.category,
      ms: v.ms,
      tasks: Array.from(v.tasks.entries()).map(([name, ms]) => ({ name, ms })),
    }));
  }

  function formatDuration(ms: number) {
    const totalSec = Math.floor(ms / 1000);
    const h = Math.floor(totalSec / 3600);
    const m = Math.floor((totalSec % 3600) / 60);
    return `${h}h ${m}m`;
  }

  $: dailyRecords = records.filter(
    (r) =>
      r.start.startsWith(selectedDate) &&
      (!projectFilter ||
        r.projectId.includes(projectFilter) ||
        r.projectName.includes(projectFilter))
  );
  $: totals = computeTotals(dailyRecords);
</script>

<main>
  <h1>Work Time Tracker</h1>

  <div class="save-dir">
    <button on:click={chooseDirectory}>Choose Save Directory</button>
    {#if saveDir}
      <span class="path">{saveDir}</span>
    {/if}
  </div>

  <div class="inputs">
    <input bind:value={projectId} placeholder="Project ID" on:input={onProjectIdInput} />
    <input bind:value={projectName} placeholder="Project Name" />
    <input bind:value={category} placeholder="Category" />
    <input bind:value={task} placeholder="Task" />
  </div>

  <button on:click={startRecord}>Start</button>
  <button on:click={stopRecord}>Stop</button>

  <div class="date-select">
    <label>
      Date <input type="date" bind:value={selectedDate} />
    </label>
  </div>

  <div class="filter">
    <input
      placeholder="Filter by project"
      bind:value={projectFilter}
    />
  </div>

  <h2>Records</h2>
  <table>
    <thead>
      <tr>
        <th>Project</th>
        <th>Category</th>
        <th>Task</th>
        <th>Start</th>
        <th>End</th>
        <th></th>
      </tr>
    </thead>
    <tbody>
      {#each dailyRecords as r}
        {#if editing && editing.id === r.id}
          <tr>
            <td>
              <input bind:value={editing.projectName} />
              <div class="sub">
                <input bind:value={editing.projectId} on:input={onEditProjectIdInput} />
              </div>
            </td>
            <td><input bind:value={editing.category} /></td>
            <td><input bind:value={editing.task} /></td>
            <td>
              <input type="datetime-local" bind:value={editing.start} />
            </td>
            <td>
              <input type="datetime-local" bind:value={editing.end} />
            </td>
            <td>
              <button on:click={saveEdit}>Save</button>
              <button on:click={cancelEdit}>Cancel</button>
            </td>
          </tr>
        {:else}
          <tr>
            <td>{r.projectName} ({r.projectId})</td>
            <td>{r.category}</td>
            <td>{r.task}</td>
            <td>{new Date(r.start).toLocaleTimeString()}</td>
            <td>{r.end ? new Date(r.end).toLocaleTimeString() : '-'}</td>
            <td><button on:click={() => beginEdit(r)}>Edit</button></td>
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>

  <h2>Total by Project & Category</h2>
  <ul>
    {#each totals as t}
      <li>
        {t.projectId} ({t.projectName}) {t.category}: {formatDuration(t.ms)}
        <br />
        {t.tasks
          .map((tk) => `${tk.name}(${formatDuration(tk.ms)})`)
          .join('、')}
      </li>
    {/each}
  </ul>
</main>

<style>
  main {
    max-width: 800px;
    margin: 0 auto;
    padding: 1em;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th,
  td {
    border: 1px solid #ccc;
    padding: 0.25em 0.5em;
  }
  .inputs input {
    margin-right: 0.5em;
  }
  .filter {
    margin: 0.5em 0;
  }
  .save-dir {
    margin-bottom: 0.5em;
  }
  .save-dir .path {
    margin-left: 0.5em;
    font-size: 0.9em;
    color: #555;
  }
  .sub input {
    margin-top: 0.25em;
    width: 100%;
  }
</style>
