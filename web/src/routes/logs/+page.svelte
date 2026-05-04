<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import { fetchLogs, type LogEntry } from '$lib/api';
	import { Heading, Button, Select, Toggle } from 'flowbite-svelte';

	const POLL_MS = 2000;
	const TAIL_INITIAL = 100;
	const MAX_VISIBLE = 500;

	let entries: LogEntry[] = $state([]);
	let lastId: number | null = $state(null);
	let level: string = $state('INFO');
	let paused: boolean = $state(false);
	let autoscroll: boolean = $state(true);
	let error: string | null = $state(null);
	let loading: boolean = $state(false);
	let logContainer: HTMLDivElement | undefined = $state();

	const LEVELS = [
		{ value: 'TRACE', name: 'Trace' },
		{ value: 'DEBUG', name: 'Debug' },
		{ value: 'INFO', name: 'Info' },
		{ value: 'WARN', name: 'Warn' },
		{ value: 'ERROR', name: 'Error' }
	];

	// In-flight guard: refuses to fire a new fetch while one is pending.
	// Without this, a slow poll + a fast next-tick can race and append
	// entries out of order.
	let inFlight = false;
	// Each fetch carries the level it was started with; if `level` changes
	// while a fetch is in flight, we discard that response on arrival.
	let pollTimer: ReturnType<typeof setInterval> | null = null;

	async function fetchOnce(opts: { tail?: number; reset: boolean }) {
		if (inFlight) return;
		const requestedLevel = level;
		const reset = opts.reset;
		inFlight = true;
		if (reset) loading = true;
		try {
			const r = await fetchLogs({
				tail: opts.tail,
				level: requestedLevel,
				since: reset ? undefined : (lastId ?? undefined)
			});
			// If level changed mid-flight, ignore the response — a fresh
			// reset fetch is already on its way.
			if (level !== requestedLevel) return;
			if (reset) {
				entries = r.entries;
				lastId = r.last_id;
			} else if (r.entries.length > 0) {
				const next = entries.length + r.entries.length > MAX_VISIBLE
					? [...entries, ...r.entries].slice(-MAX_VISIBLE)
					: [...entries, ...r.entries];
				entries = next;
				lastId = r.last_id;
			}
			if (error) error = null;
			if (r.entries.length > 0 || reset) await scheduleScroll();
		} catch (e) {
			error = e instanceof Error ? e.message : 'load failed';
		} finally {
			inFlight = false;
			if (reset) loading = false;
		}
	}

	async function scheduleScroll() {
		if (!autoscroll) return;
		await tick();
		if (logContainer) {
			logContainer.scrollTop = logContainer.scrollHeight;
		}
	}

	function clearLogs() {
		entries = [];
	}

	function levelClass(lvl: string): string {
		switch (lvl) {
			case 'ERROR': return 'text-red-400';
			case 'WARN':  return 'text-yellow-400';
			case 'INFO':  return 'text-blue-300';
			case 'DEBUG': return 'text-gray-400';
			case 'TRACE': return 'text-gray-500';
			default:      return 'text-gray-200';
		}
	}

	function formatTime(at: string): string {
		try {
			return new Date(at).toLocaleTimeString();
		} catch {
			return at;
		}
	}

	function copyAll() {
		const text = entries
			.map(e => `${e.at} ${e.level} ${e.target}: ${e.message}`)
			.join('\n');
		navigator.clipboard.writeText(text).catch(() => {});
	}

	function shouldPoll(): boolean {
		if (paused) return false;
		// Don't poll when the tab is hidden — the user isn't watching, and
		// it lets the daemon's broadcast channel cool down.
		if (typeof document !== 'undefined' && document.hidden) return false;
		return true;
	}

	// Reload from scratch whenever the level changes (or on first mount).
	let prevLevel: string | null = null;
	$effect(() => {
		if (prevLevel === level) return;
		prevLevel = level;
		entries = [];
		lastId = null;
		fetchOnce({ tail: TAIL_INITIAL, reset: true });
	});

	onMount(() => {
		pollTimer = setInterval(() => {
			if (shouldPoll()) fetchOnce({ reset: false });
		}, POLL_MS);
	});

	onDestroy(() => {
		if (pollTimer) {
			clearInterval(pollTimer);
			pollTimer = null;
		}
	});
</script>

<svelte:head>
	<title>wshm — Logs</title>
</svelte:head>

<div class="mb-4">
	<Heading tag="h2" class="text-xl mb-1">Daemon logs</Heading>
	<p class="text-sm text-gray-500">Tail of the in-memory log buffer (resets on daemon restart). Polls every {POLL_MS / 1000}s when this tab is visible.</p>
</div>

<div class="flex flex-wrap items-center gap-3 mb-3">
	<div class="flex items-center gap-2">
		<span class="text-xs text-gray-400">Min level</span>
		<Select bind:value={level} items={LEVELS} class="w-28" size="sm" />
	</div>

	<Toggle bind:checked={paused}>Pause</Toggle>
	<Toggle bind:checked={autoscroll}>Autoscroll</Toggle>

	<div class="ml-auto flex gap-2">
		<Button color="alternative" size="xs" disabled={loading} onclick={() => fetchOnce({ tail: TAIL_INITIAL, reset: true })}>
			{loading ? 'Loading…' : 'Reload'}
		</Button>
		<Button color="alternative" size="xs" onclick={copyAll}>Copy all</Button>
		<Button color="alternative" size="xs" onclick={clearLogs}>Clear view</Button>
	</div>
</div>

{#if error}
	<div class="rounded border border-red-700 bg-red-900/40 px-3 py-2 text-xs text-red-300 mb-3">
		{error}
	</div>
{/if}

<div
	bind:this={logContainer}
	class="rounded border border-gray-700 bg-gray-950 p-3 font-mono text-xs leading-5 overflow-auto"
	style="height: calc(100vh - 200px); min-height: 280px; max-height: 75vh;"
>
	{#if entries.length === 0}
		<div class="text-gray-500">{loading ? 'Loading…' : 'No log entries yet.'}</div>
	{:else}
		{#each entries as entry (entry.id)}
			<div class="flex gap-2 hover:bg-gray-900/50 px-1">
				<span class="text-gray-500 shrink-0">{formatTime(entry.at)}</span>
				<span class="font-semibold w-12 shrink-0 {levelClass(entry.level)}">{entry.level}</span>
				<span class="text-gray-400 shrink-0 max-w-[260px] truncate" title={entry.target}>{entry.target}</span>
				<span class="text-gray-200 break-all">{entry.message}</span>
			</div>
		{/each}
	{/if}
</div>

<div class="mt-2 text-xs text-gray-500">
	{entries.length} entries · last id {lastId ?? '—'} · {paused ? 'paused' : 'live'}
</div>
