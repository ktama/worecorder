<script lang="ts">
  import { onMount } from 'svelte';

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
  let currentId = 0;

  onMount(() => {
    const stored = localStorage.getItem('records');
    if (stored) {
      records = JSON.parse(stored);
      currentId =
        records.reduce((max, r) => (r.id > max ? r.id : max), 0) + 1;
    }
  });

  function save() {
    localStorage.setItem('records', JSON.stringify(records));
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

  function duration(start: string, end?: string) {
    if (!end) return 0;
    return new Date(end).getTime() - new Date(start).getTime();
  }

  function computeTotals(recs: RecordEntry[]) {
    const map = new Map<
      string,
      { projectId: string; projectName: string; category: string; ms: number }
    >();
    for (const r of recs) {
      if (!r.end) continue;
      const key = `${r.projectId}|${r.category}`;
      const ms = duration(r.start, r.end);
      if (map.has(key)) {
        map.get(key)!.ms += ms;
      } else {
        map.set(key, {
          projectId: r.projectId,
          projectName: r.projectName,
          category: r.category,
          ms,
        });
      }
    }
    return Array.from(map.values());
  }

  function formatDuration(ms: number) {
    const totalSec = Math.floor(ms / 1000);
    const h = Math.floor(totalSec / 3600);
    const m = Math.floor((totalSec % 3600) / 60);
    return `${h}h ${m}m`;
  }

  $: dailyRecords = records.filter((r) =>
    r.start.startsWith(selectedDate)
  );
  $: totals = computeTotals(dailyRecords);
</script>

<main>
  <h1>Work Time Tracker</h1>

  <div class="inputs">
    <input bind:value={projectId} placeholder="Project ID" />
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

  <h2>Records</h2>
  <table>
    <thead>
      <tr>
        <th>Project</th>
        <th>Category</th>
        <th>Task</th>
        <th>Start</th>
        <th>End</th>
      </tr>
    </thead>
    <tbody>
      {#each dailyRecords as r}
        <tr>
          <td>{r.projectName} ({r.projectId})</td>
          <td>{r.category}</td>
          <td>{r.task}</td>
          <td>{new Date(r.start).toLocaleTimeString()}</td>
          <td>{r.end ? new Date(r.end).toLocaleTimeString() : '-'}</td>
        </tr>
      {/each}
    </tbody>
  </table>

  <h2>Total by Project & Category</h2>
  <ul>
    {#each totals as t}
      <li>
        {t.projectId} ({t.projectName}) {t.category}: {formatDuration(t.ms)}
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
</style>
