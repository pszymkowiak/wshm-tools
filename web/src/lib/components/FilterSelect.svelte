<script lang="ts">
	interface Props {
		options: string[];
		value: string;
		onchange?: (v: string) => void;
		placeholder?: string;
	}

	let { options, value = $bindable(''), onchange, placeholder = 'All' }: Props = $props();

	function handle(e: Event) {
		const v = (e.target as HTMLSelectElement).value;
		value = v;
		onchange?.(v);
	}
</script>

<select
	{value}
	onchange={handle}
	class="w-full rounded border border-gray-600 bg-gray-700 px-1 py-0.5 text-xs text-gray-200 focus:border-blue-500 focus:outline-none"
>
	<option value="">{placeholder}</option>
	{#each options as o (o)}
		<option value={o}>{o}</option>
	{/each}
</select>
