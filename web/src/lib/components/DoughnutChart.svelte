<script lang="ts">
	import { Chart, ArcElement, DoughnutController, Tooltip, Legend } from 'chart.js';

	// chart.js is tree-shakeable: only the doughnut pieces are registered.
	Chart.register(ArcElement, DoughnutController, Tooltip, Legend);

	export type Segment = { label: string; value: number; color: string };

	let {
		segments,
		centerLabel = ''
	}: { segments: Segment[]; centerLabel?: string } = $props();

	let canvas: HTMLCanvasElement | undefined = $state();
	let chart: Chart | null = null;

	function render() {
		if (!canvas) return;
		const data = {
			labels: segments.map((s) => s.label),
			datasets: [
				{
					data: segments.map((s) => s.value),
					backgroundColor: segments.map((s) => s.color),
					borderColor: '#1f2937', // gray-800, matches card background
					borderWidth: 2
				}
			]
		};
		if (chart) {
			chart.data = data;
			chart.update();
			return;
		}
		chart = new Chart(canvas, {
			type: 'doughnut',
			data,
			options: {
				responsive: true,
				maintainAspectRatio: false,
				cutout: '62%',
				plugins: {
					legend: {
						position: 'bottom',
						labels: {
							color: '#9ca3af', // gray-400
							font: { size: 11 },
							boxWidth: 12,
							padding: 12
						}
					},
					tooltip: {
						callbacks: {
							label: (ctx) => ` ${ctx.label}: ${ctx.parsed}`
						}
					}
				}
			}
		});
	}

	$effect(() => {
		// Reading `segments` here makes the effect re-run when data changes.
		void segments;
		render();
		return () => {
			chart?.destroy();
			chart = null;
		};
	});
</script>

<div class="relative h-[260px]">
	<canvas bind:this={canvas}></canvas>
	{#if centerLabel}
		<div
			class="pointer-events-none absolute inset-x-0 top-[42%] -translate-y-1/2 text-center"
		>
			<div class="text-2xl font-bold text-gray-100 mono">{centerLabel}</div>
		</div>
	{/if}
</div>
