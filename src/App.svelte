<script lang="ts">
  import { onMount } from 'svelte';

  interface RecordItem {
    projectId: string;
    projectName: string;
    category: string;
    content: string;
    start: string;
    end?: string;
  }

  let projectId = '';
  let projectName = '';
  let category = '';
  let content = '';
  let records: RecordItem[] = [];
  let current: RecordItem | null = null;
  let selectedDate = new Date().toISOString().substring(0,10);

  onMount(() => {
    const stored = localStorage.getItem('records');
    if (stored) {
      records = JSON.parse(stored);
    }
  });

  function save() {
    localStorage.setItem('records', JSON.stringify(records));
  }

  function start() {
    if (current && !current.end) {
      current.end = new Date().toISOString();
      records = [...records, current];
    }
    current = {
      projectId,
      projectName,
      category,
      content,
      start: new Date().toISOString()
    };
    save();
  }

  function stop() {
    if (current) {
      current.end = new Date().toISOString();
      records = [...records, current];
      current = null;
      save();
    }
  }

  function recordsFor(dateStr: string) {
    return records.filter(r => r.start.startsWith(dateStr));
  }

  function durationMinutes(start: string, end?: string) {
    const s = new Date(start).getTime();
    const e = new Date(end ?? new Date().toISOString()).getTime();
    return Math.round((e - s) / 60000);
  }

  function summary(dateStr: string) {
    const result: { [key: string]: { total: number; details: { [key: string]: number }; item: RecordItem } } = {};
    for (const r of recordsFor(dateStr)) {
      const key = `${r.projectId}-${r.category}`;
      const min = durationMinutes(r.start, r.end);
      if (!result[key]) {
        result[key] = { total: 0, details: {}, item: r };
      }
      result[key].total += min;
      result[key].details[r.content] = (result[key].details[r.content] || 0) + min;
    }
    return Object.values(result);
  }
</script>

<main>
  <h1>Worecorder</h1>

  <div>
    <label>
      プロジェクトID
      <input bind:value={projectId} />
    </label>
    <label>
      プロジェクト名
      <input bind:value={projectName} />
    </label>
    <label>
      カテゴリ
      <input bind:value={category} />
    </label>
    <label>
      作業内容
      <input bind:value={content} />
    </label>
    <button on:click={start}>開始</button>
    <button on:click={stop}>終了</button>
  </div>

  {#if current}
    <p>作業中: {current.content} ({current.start})</p>
  {/if}

  <div>
    <label>
      日付
      <input type="date" bind:value={selectedDate} />
    </label>
  </div>

  <h2>記録一覧 ({selectedDate})</h2>
  <ul>
    {#each recordsFor(selectedDate) as r}
      <li>{r.start} - {r.end ?? '...'} {r.projectName} {r.category} {r.content}</li>
    {/each}
  </ul>

  <h2>日別集計</h2>
  <ul>
    {#each summary(selectedDate) as s}
      <li>{s.item.projectId} {s.item.category}: {s.total}分 - {Object.entries(s.details).map(([k,v]) => `${k}(${v}分)`).join('、')}</li>
    {/each}
  </ul>
</main>

<style>
  main {
    padding: 1em;
    font-family: sans-serif;
  }
  label {
    display: block;
    margin-bottom: 0.5em;
  }
  input {
    margin-left: 0.5em;
  }
  button {
    margin-right: 0.5em;
  }
</style>
