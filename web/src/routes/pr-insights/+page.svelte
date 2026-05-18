<script lang="ts">
	import { onMount } from 'svelte';
	import { selectedRepo } from '$lib/stores';
	import { fetchPrInsights, type PrInsightItem, type PrInsights } from '$lib/api';
	import { t } from '$lib/i18n';
	import {
		Tabs,
		TabItem,
		Table,
		TableHead,
		TableHeadCell,
		TableBody,
		TableBodyRow,
		TableBodyCell,
		Badge,
		Card,
		Button
	} from 'flowbite-svelte';
	import DoughnutChart, { type Segment } from '$lib/components/DoughnutChart.svelte';

	let translate = $state<(k: string) => string>((k) => k);
	t.subscribe((fn) => (translate = fn));

	let data = $state<PrInsights | null>(null);
	let error: string | null = $state(null);
	let proLocked = $state(false);
	let loading = $state(true);

	let loadToken = 0;
	async function load() {
		const myToken = ++loadToken;
		loading = true;
		error = null;
		proLocked = false;
		try {
			const res = await fetchPrInsights();
			if (myToken !== loadToken) return;
			data = res;
		} catch (e) {
			if (myToken !== loadToken) return;
			const msg = e instanceof Error ? e.message : 'Failed to load';
			// fetchPrInsights surfaces a 403 from the Pro gate as "API error: 403 ...".
			if (msg.includes('403')) proLocked = true;
			else error = msg;
		}
		loading = false;
	}

	onMount(() => {
		load();
		const unsub = selectedRepo.subscribe(() => load());
		return unsub;
	});

	let items = $derived(data?.items ?? []);
	let defaults = $derived(new Set(data?.default_branches ?? []));

	let conflictItems = $derived(items.filter((p) => p.status === 'conflict'));
	let mergeableItems = $derived(items.filter((p) => p.status === 'mergeable'));
	let unknownCount = $derived(items.filter((p) => p.status === 'unknown').length);

	let toDefaultCount = $derived(
		items.filter((p) => p.base_ref != null && defaults.has(p.base_ref)).length
	);
	let toOtherCount = $derived(items.length - toDefaultCount);

	// Target-branch breakdown: count per base_ref, default branches first.
	type BranchGroup = { branch: string; count: number; isDefault: boolean };
	let branchGroups = $derived.by<BranchGroup[]>(() => {
		const counts = new Map<string, number>();
		for (const p of items) {
			const b = p.base_ref ?? '(unknown)';
			counts.set(b, (counts.get(b) ?? 0) + 1);
		}
		const groups = [...counts.entries()].map(([branch, count]) => ({
			branch,
			count,
			isDefault: defaults.has(branch)
		}));
		groups.sort((a, b) => {
			if (a.isDefault !== b.isDefault) return a.isDefault ? -1 : 1;
			return b.count - a.count;
		});
		return groups;
	});

	const BRANCH_PALETTE = [
		'#3b82f6', '#8b5cf6', '#ec4899', '#f59e0b', '#14b8a6',
		'#06b6d4', '#a3e635', '#f97316', '#64748b'
	];

	let mergeabilitySegments = $derived<Segment[]>(
		[
			{ label: translate('prInsights.legend.mergeable'), value: mergeableItems.length, color: '#22c55e' },
			{ label: translate('prInsights.legend.conflict'), value: conflictItems.length, color: '#ef4444' },
			{ label: translate('prInsights.legend.unknown'), value: unknownCount, color: '#6b7280' }
		].filter((s) => s.value > 0)
	);

	let branchSegments = $derived<Segment[]>(
		branchGroups.map((g, i) => ({
			label: g.branch,
			value: g.count,
			color: BRANCH_PALETTE[i % BRANCH_PALETTE.length]
		}))
	);

	// "By branch" tab filter (empty = all branches).
	let branchFilter = $state<string>('');
	let byBranchItems = $derived(
		branchFilter ? items.filter((p) => (p.base_ref ?? '(unknown)') === branchFilter) : items
	);

	function riskColor(risk: string | null): 'green' | 'yellow' | 'red' | 'gray' {
		if (risk === 'low') return 'green';
		if (risk === 'medium') return 'yellow';
		if (risk === 'high') return 'red';
		return 'gray';
	}

	function ciColor(ci: string | null): 'green' | 'red' | 'yellow' | 'gray' {
		if (ci === 'passing') return 'green';
		if (ci === 'failing') return 'red';
		if (ci === 'pending') return 'yellow';
		return 'gray';
	}
</script>

<svelte:head>
	<title>wshm - PR Insights</title>
</svelte:head>

<div class="mb-6">
	<h2 class="text-xl font-semibold text-gray-100 mb-1">{translate('prInsights.title')}</h2>
	<p class="text-sm text-gray-500">{translate('prInsights.subtitle')}</p>
</div>

{#snippet statCard(label: string, value: number | string, accent: string)}
	<Card class="bg-gray-800 border-gray-700 text-center max-w-none">
		<div class="text-xs uppercase tracking-wider text-gray-500 mb-2">{label}</div>
		<div class="text-3xl font-bold mono {accent}">{value}</div>
	</Card>
{/snippet}

{#snippet prTable(rows: PrInsightItem[])}
	<div class="w-full overflow-x-auto">
		<Table striped hoverable class="w-full">
			<TableHead class="text-xs uppercase text-gray-400">
				<TableHeadCell class="px-2 py-1.5 w-[70px]">{translate('prInsights.table.pr')}</TableHeadCell>
				<TableHeadCell class="px-2 py-1.5">{translate('prInsights.table.repo')}</TableHeadCell>
				<TableHeadCell class="px-2 py-1.5">{translate('prInsights.table.title')}</TableHeadCell>
				<TableHeadCell class="px-2 py-1.5">{translate('prInsights.table.author')}</TableHeadCell>
				<TableHeadCell class="px-2 py-1.5">{translate('prInsights.table.target')}</TableHeadCell>
				<TableHeadCell class="px-2 py-1.5 w-[80px]">{translate('prInsights.table.ci')}</TableHeadCell>
				<TableHeadCell class="px-2 py-1.5 w-[80px]">{translate('prInsights.table.risk')}</TableHeadCell>
			</TableHead>
			<TableBody>
				{#each rows as pr (pr.repo + '#' + pr.number)}
					<TableBodyRow>
						<TableBodyCell class="px-2 py-1.5 mono">
							{#if pr.url}
								<a href={pr.url} target="_blank" rel="noopener" class="text-blue-400 hover:text-blue-300">#{pr.number}</a>
							{:else}
								#{pr.number}
							{/if}
						</TableBodyCell>
						<TableBodyCell class="px-2 py-1.5 mono text-xs text-gray-400">{pr.repo}</TableBodyCell>
						<TableBodyCell class="px-2 py-1.5 truncate max-w-[420px]">{pr.title}</TableBodyCell>
						<TableBodyCell class="px-2 py-1.5 text-gray-400">{pr.author ?? '-'}</TableBodyCell>
						<TableBodyCell class="px-2 py-1.5">
							{#if pr.base_ref}
								<Badge color={defaults.has(pr.base_ref) ? 'blue' : 'gray'}>{pr.base_ref}</Badge>
							{:else}
								<span class="text-gray-600">-</span>
							{/if}
						</TableBodyCell>
						<TableBodyCell class="px-2 py-1.5">
							{#if pr.ci_status}
								<Badge color={ciColor(pr.ci_status)}>{pr.ci_status}</Badge>
							{:else}
								<span class="text-gray-600">-</span>
							{/if}
						</TableBodyCell>
						<TableBodyCell class="px-2 py-1.5">
							{#if pr.risk_level}
								<Badge color={riskColor(pr.risk_level)}>{pr.risk_level}</Badge>
							{:else}
								<span class="text-gray-600">-</span>
							{/if}
						</TableBodyCell>
					</TableBodyRow>
				{:else}
					<TableBodyRow>
						<TableBodyCell colspan={7} class="text-center text-gray-600 py-8">
							{translate('prInsights.empty')}
						</TableBodyCell>
					</TableBodyRow>
				{/each}
			</TableBody>
		</Table>
	</div>
{/snippet}

{#if loading}
	<p class="text-sm text-gray-500">{translate('prInsights.loading')}</p>
{:else if proLocked}
	<Card class="bg-gray-800 border-blue-700 max-w-xl">
		<h3 class="text-lg font-semibold text-gray-100 mb-2">{translate('prInsights.proOnly.title')}</h3>
		<p class="text-sm text-gray-400">{translate('prInsights.proOnly.body')}</p>
	</Card>
{:else if error}
	<Card class="border-red-500 bg-gray-800 max-w-none">
		<p class="text-red-400">{error}</p>
	</Card>
{:else}
	<Tabs tabStyle="underline">
		<TabItem open title={translate('prInsights.tab.overview')}>
			<div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4 mb-6">
				{@render statCard(translate('prInsights.stat.total'), items.length, 'text-gray-100')}
				{@render statCard(translate('prInsights.stat.mergeable'), mergeableItems.length, 'text-green-400')}
				{@render statCard(translate('prInsights.stat.conflict'), conflictItems.length, 'text-red-400')}
				{@render statCard(translate('prInsights.stat.unknown'), unknownCount, 'text-gray-400')}
				{@render statCard(translate('prInsights.stat.toDefault'), toDefaultCount, 'text-blue-400')}
				{@render statCard(translate('prInsights.stat.toOther'), toOtherCount, 'text-purple-400')}
			</div>
			<div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
				<Card class="bg-gray-800 border-gray-700 max-w-none">
					<h3 class="text-sm font-semibold text-gray-200 mb-3">
						{translate('prInsights.chart.mergeability')}
					</h3>
					{#if items.length > 0}
						<DoughnutChart segments={mergeabilitySegments} centerLabel={String(items.length)} />
					{:else}
						<p class="text-sm text-gray-600 py-8 text-center">{translate('prInsights.empty')}</p>
					{/if}
				</Card>
				<Card class="bg-gray-800 border-gray-700 max-w-none">
					<h3 class="text-sm font-semibold text-gray-200 mb-3">
						{translate('prInsights.chart.targetBranch')}
					</h3>
					{#if items.length > 0}
						<DoughnutChart segments={branchSegments} centerLabel={String(branchGroups.length)} />
					{:else}
						<p class="text-sm text-gray-600 py-8 text-center">{translate('prInsights.empty')}</p>
					{/if}
				</Card>
			</div>
		</TabItem>

		<TabItem title="{translate('prInsights.tab.conflicts')} ({conflictItems.length})">
			{@render prTable(conflictItems)}
		</TabItem>

		<TabItem title="{translate('prInsights.tab.mergeable')} ({mergeableItems.length})">
			{@render prTable(mergeableItems)}
		</TabItem>

		<TabItem title={translate('prInsights.tab.byBranch')}>
			<div class="flex flex-wrap gap-2 mb-4">
				<Button
					size="xs"
					color={branchFilter === '' ? 'blue' : 'alternative'}
					onclick={() => (branchFilter = '')}
				>
					{translate('prInsights.allBranches')} ({items.length})
				</Button>
				{#each branchGroups as g (g.branch)}
					<Button
						size="xs"
						color={branchFilter === g.branch ? 'blue' : 'alternative'}
						onclick={() => (branchFilter = g.branch)}
					>
						{g.branch} ({g.count}){#if g.isDefault}<span class="ml-1 text-[0.6rem] uppercase text-blue-300">{translate('prInsights.defaultBadge')}</span>{/if}
					</Button>
				{/each}
			</div>
			{@render prTable(byBranchItems)}
		</TabItem>
	</Tabs>
{/if}
