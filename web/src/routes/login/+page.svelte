<script lang="ts">
	import { Button, Input, Label, Alert, Heading, P, Dropdown, DropdownItem } from 'flowbite-svelte';
	import { locale, t, LOCALES, type Locale } from '$lib/i18n';

	let username = $state('');
	let password = $state('');
	let submitting = $state(false);
	let error = $state<string | null>(null);
	let currentLocale: Locale = $state('en');
	locale.subscribe((v) => (currentLocale = v));
	let translate = $state<(k: string) => string>((k) => k);
	t.subscribe((fn) => (translate = fn));
	let activeLocale = $derived(LOCALES.find((l) => l.code === currentLocale) ?? LOCALES[0]);

	async function handleSubmit(event: Event) {
		event.preventDefault();
		submitting = true;
		error = null;
		try {
			const res = await fetch('/api/v1/auth/login', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ username, password })
			});
			if (!res.ok) {
				const body = await res.json().catch(() => ({}));
				error = body.error ?? `${translate('login.failed')} (${res.status})`;
				submitting = false;
				return;
			}
			window.location.replace('/');
		} catch (e) {
			error = e instanceof Error ? e.message : translate('login.failed');
			submitting = false;
		}
	}

	function setLocale(l: Locale) {
		locale.set(l);
	}

	const stars = Array.from({ length: 60 }, (_, i) => ({
		id: i,
		top: Math.random() * 100,
		left: Math.random() * 100,
		size: 1 + Math.random() * 2.5,
		delay: Math.random() * 4,
		dur: 2.5 + Math.random() * 3.5
	}));
</script>

<svelte:head>
	<title>Sign in to wshm</title>
</svelte:head>

<div
	class="min-h-screen flex items-center justify-center relative overflow-hidden text-gray-100 px-4 py-12"
>
	<!-- Wizard-themed mesh gradient + twinkling stars -->
	<div class="pointer-events-none absolute inset-0 -z-10">
		<div class="absolute inset-0 bg-gradient-to-br from-indigo-950 via-purple-900 to-fuchsia-950"></div>
		<div
			class="absolute top-[-15%] left-[-10%] w-[560px] h-[560px] rounded-full bg-violet-500/30 blur-3xl"
		></div>
		<div
			class="absolute bottom-[-15%] right-[-10%] w-[600px] h-[600px] rounded-full bg-fuchsia-500/25 blur-3xl"
		></div>
		<div
			class="absolute top-1/3 right-1/4 w-[360px] h-[360px] rounded-full bg-amber-400/15 blur-3xl"
		></div>
		<div
			class="absolute bottom-1/3 left-1/4 w-[300px] h-[300px] rounded-full bg-indigo-400/20 blur-3xl"
		></div>

		<!-- Stars -->
		<div class="absolute inset-0">
			{#each stars as s (s.id)}
				<span
					class="absolute rounded-full bg-white twinkle"
					style="top: {s.top}%; left: {s.left}%; width: {s.size}px; height: {s.size}px; animation-delay: {s.delay}s; animation-duration: {s.dur}s;"
				></span>
			{/each}
		</div>

		<!-- Faint magic-circle decoration -->
		<svg
			class="absolute -bottom-32 -right-32 w-[500px] h-[500px] opacity-[0.08] text-amber-200"
			viewBox="0 0 200 200"
			fill="none"
			stroke="currentColor"
			aria-hidden="true"
		>
			<circle cx="100" cy="100" r="95" stroke-width="0.4" />
			<circle cx="100" cy="100" r="78" stroke-width="0.4" stroke-dasharray="2 3" />
			<circle cx="100" cy="100" r="60" stroke-width="0.4" />
			<polygon
				points="100,15 145,160 25,72 175,72 55,160"
				stroke-width="0.4"
			/>
		</svg>
	</div>

	<!-- Locale picker (top-right dropdown) -->
	<div class="absolute top-4 right-4 z-10">
		<Button
			id="locale-trigger"
			color="alternative"
			size="xs"
			class="!bg-white/10 !border-white/20 !text-white hover:!bg-white/20 backdrop-blur-md flex items-center gap-1.5"
		>
			<span class="text-base leading-none">{activeLocale.flag}</span>
			<span class="text-xs">{activeLocale.label}</span>
			<svg class="w-3 h-3 ms-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
			</svg>
		</Button>
		<Dropdown
			triggeredBy="#locale-trigger"
			class="z-50 max-h-80 overflow-y-auto"
			placement="bottom-end"
		>
			{#each LOCALES as l}
				<DropdownItem onclick={() => setLocale(l.code)}>
					<span class="me-2 text-base leading-none">{l.flag}</span>
					<span class="text-sm">{l.label}</span>
					{#if currentLocale === l.code}
						<span class="ms-auto text-amber-500">✓</span>
					{/if}
				</DropdownItem>
			{/each}
		</Dropdown>
	</div>

	<div class="w-full max-w-sm relative">
		<div class="flex flex-col items-center mb-8">
			<div class="relative">
				<div class="absolute inset-0 rounded-full bg-amber-400/30 blur-2xl"></div>
				<img src="/wizard-icon.png" alt="" class="relative h-16 w-16 mb-3 drop-shadow-[0_0_18px_rgba(251,191,36,0.6)]" />
			</div>
			<Heading tag="h1" class="text-2xl font-semibold text-white">
				{translate('login.welcome')}
			</Heading>
			<P class="text-sm text-purple-200/80 mt-1">{translate('login.subtitle')}</P>
		</div>

		<div class="rounded-xl border border-white/10 bg-white/5 backdrop-blur-xl p-6 shadow-2xl shadow-black/40">
			<form onsubmit={handleSubmit} class="space-y-4">
				<div>
					<Label for="username" class="block text-xs uppercase tracking-wider text-purple-200/80 mb-1.5">
						{translate('login.username')}
					</Label>
					<Input
						id="username"
						type="text"
						autocomplete="username"
						bind:value={username}
						required
						placeholder={translate('login.usernamePlaceholder')}
					/>
				</div>

				<div>
					<Label for="password" class="block text-xs uppercase tracking-wider text-purple-200/80 mb-1.5">
						{translate('login.password')}
					</Label>
					<Input
						id="password"
						type="password"
						autocomplete="current-password"
						bind:value={password}
						required
					/>
				</div>

				{#if error}
					<Alert color="red" class="text-xs">{error}</Alert>
				{/if}

				<Button type="submit" color="purple" disabled={submitting} class="w-full">
					{submitting ? translate('login.signingIn') : translate('login.submit')}
				</Button>
			</form>

			<div class="flex items-center my-6">
				<div class="flex-1 h-px bg-white/10"></div>
				<span class="px-3 text-xs uppercase tracking-wider text-purple-200/60">
					{translate('common.or')}
				</span>
				<div class="flex-1 h-px bg-white/10"></div>
			</div>

			<Button
				href="/oauth2/start?rd=/"
				color="alternative"
				class="w-full !py-2.5 flex items-center justify-center gap-3 !bg-white/5 !border-white/15 !text-white hover:!bg-white/15"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					class="h-5 w-5"
					aria-hidden="true"
				>
					<path
						fill="#4285F4"
						d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
					/>
					<path
						fill="#34A853"
						d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84A11 11 0 0 0 12 23z"
					/>
					<path
						fill="#FBBC05"
						d="M5.84 14.09a6.6 6.6 0 0 1 0-4.18V7.07H2.18a11 11 0 0 0 0 9.86l3.66-2.84z"
					/>
					<path
						fill="#EA4335"
						d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1A11 11 0 0 0 2.18 7.07l3.66 2.84C6.71 7.31 9.14 5.38 12 5.38z"
					/>
				</svg>
				<span>{translate('login.googleButton')}</span>
			</Button>
		</div>

		<P class="text-center text-xs text-purple-200/60 mt-8 italic">
			{translate('login.tagline')}
		</P>
	</div>
</div>

<style>
	.twinkle {
		opacity: 0.2;
		animation-name: twinkle;
		animation-iteration-count: infinite;
		animation-timing-function: ease-in-out;
		box-shadow: 0 0 4px rgba(255, 255, 255, 0.6);
	}
	@keyframes twinkle {
		0%, 100% { opacity: 0.15; transform: scale(0.85); }
		50% { opacity: 0.95; transform: scale(1.15); }
	}
</style>
