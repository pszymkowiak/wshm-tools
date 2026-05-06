<script lang="ts">
	import { Button, Select } from 'flowbite-svelte';

	interface Props {
		total: number;
		limit: number;
		offset: number;
		storageKey?: string;
		onChange: (next: { limit: number; offset: number }) => void;
	}

	let { total, limit, offset, storageKey, onChange }: Props = $props();

	const sizes = [25, 50, 100, 250, 500];

	function setLimit(next: number) {
		if (storageKey) {
			try {
				localStorage.setItem(storageKey, String(next));
			} catch {
				/* private mode etc. */
			}
		}
		onChange({ limit: next, offset: 0 });
	}

	function prev() {
		const next = Math.max(0, offset - limit);
		onChange({ limit, offset: next });
	}

	function next() {
		const candidate = offset + limit;
		if (candidate >= total) return;
		onChange({ limit, offset: candidate });
	}

	const start = $derived(total === 0 ? 0 : offset + 1);
	const end = $derived(Math.min(total, offset + limit));
	const canPrev = $derived(offset > 0);
	const canNext = $derived(offset + limit < total);
</script>

<div class="flex items-center justify-between gap-3 text-xs text-gray-400 mt-3">
	<div class="flex items-center gap-2">
		<span>Per page</span>
		<Select
			class="w-24 py-1 text-xs"
			size="sm"
			value={String(limit)}
			onchange={(e) => setLimit(Number((e.target as HTMLSelectElement).value))}
		>
			{#each sizes as s (s)}
				<option value={String(s)}>{s}</option>
			{/each}
		</Select>
	</div>

	<div>
		{#if total === 0}
			0 items
		{:else}
			{start}–{end} of {total}
		{/if}
	</div>

	<div class="flex items-center gap-2">
		<Button size="xs" color="alternative" disabled={!canPrev} onclick={prev}>Prev</Button>
		<Button size="xs" color="alternative" disabled={!canNext} onclick={next}>Next</Button>
	</div>
</div>
