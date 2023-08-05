<script>
	import init, { plan_as_json } from 'string-bean';
	import { onMount } from 'svelte';

	/** @type HTMLCanvasElement */
	// @ts-ignore
	let imagePreview;
	/** @type HTMLCanvasElement */
	// @ts-ignore
	let imageDraw;
	/** @type HTMLImageElement */
	// @ts-ignore
	let imageSource;
	let maxRadius = 2000;

	/**
	 * Settings
	 */
	let num_chords = 500;
	let line_opacity = 0.2;
	let num_anchors = 188;
	let num_anchor_gap = 0;
	let radius = 0;
	let penalty = 100;
	let start_anchor = 0;

	/**
	 * @param fileData {Blob}
	 */
	async function imageUpload(fileData) {
		const imageURL = await new Promise((resolve, _) => {
			const fr = new FileReader();
			fr.onload = () => resolve(fr.result);
			fr.readAsDataURL(fileData);
		});

		await new Promise((resolve, _) => {
			imageSource.onload = () => {
				radius = maxRadius = Math.min(imageSource.height, imageSource.width);
				// block until finished
				resolve(null);
			};
			imageSource.src = imageURL;
		});
	}

	function draw() {
		const size = Math.min(window.innerWidth - 100, window.innerHeight / 2 - 50);

		imagePreview.width = size;
		imagePreview.height = size;
		/** @type CanvasRenderingContext2D  */
		// @ts-ignore
		const imagePreviewCtx = imagePreview.getContext('2d');
		imagePreviewCtx.clearRect(0, 0, imagePreview.width, imagePreview.height);
		imagePreviewCtx.drawImage(imageSource, 0, 0, imagePreview.width, imagePreview.height);
		const imageBufferTemp = new Uint8Array(
			imagePreviewCtx.getImageData(0, 0, imagePreview.width, imagePreview.height).data.buffer
		);

		// pick the bytes affecting the brightness
		const imageBuffer = Uint8Array.from(
			Array.from({ length: size * size }, (_, i) => imageBufferTemp[i * 4])
		);

		const moves = JSON.parse(
			plan_as_json(
				num_chords,
				line_opacity,
				num_anchors,
				num_anchor_gap,
				radius,
				penalty,
				size,
				size,
				imageBuffer,
				start_anchor
			)
		);

		console.log(moves);

		const x_mid = size / 2;
		const y_mid = size / 2;

		const draw_radius = Math.min(x_mid, y_mid);
		const degrees_per_anchor = (2 * Math.PI) / num_anchors;

		imageDraw.width = size;
		imageDraw.height = size;
		/** @type CanvasRenderingContext2D  */
		// @ts-ignore
		const imageDrawCtx = imageDraw.getContext('2d');
		imageDrawCtx.strokeStyle = `rgba(0,0,0,${line_opacity})`;

		imageDrawCtx.clearRect(0, 0, imageDraw.width, imageDraw.height);
		imageDrawCtx.beginPath();
		for (let i = 0; i < moves.length; i++) {
			const [anchor1, anchor2] = [moves[i - 1], moves[i]];
			const deg1 = degrees_per_anchor * anchor1;
			const deg2 = degrees_per_anchor * anchor2;
			const x0 = x_mid + draw_radius * Math.cos(deg1);
			const y0 = y_mid + draw_radius * Math.sin(deg1);
			const x1 = x_mid + draw_radius * Math.cos(deg2);
			const y1 = y_mid + draw_radius * Math.sin(deg2);
			imageDrawCtx.moveTo(x0, y0);
			imageDrawCtx.lineTo(x1, y1);
		}
		imageDrawCtx.stroke();
	}

	const STATES = { UPLOAD: 0, CONFIGURE: 1 };
	let state = STATES.UPLOAD;

	onMount(async () => await init());
</script>

<svelte:head>
	<title>string-bean</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<!-- Canvas Elements -->
<div class="max-w-fit h-5/6 m-auto {state == STATES.UPLOAD ? 'hidden' : ''}">
	<div class="join join-vertical bg-blue-400">
		<div class="join-item indicator">
			<span class="indicator-item badge badge-primary">input</span>
			<canvas bind:this={imagePreview} />
		</div>
		<div class="join-item indicator">
			<span class="indicator-item badge badge-secondary">output</span>
			<canvas bind:this={imageDraw} />
		</div>
	</div>
</div>

<!-- svelte-ignore a11y-img-redundant-alt -->
<img bind:this={imageSource} class="hidden" alt="image-holder" />

{#if state == STATES.UPLOAD}
	<div class="w-screen h-screen flex p-4">
		<label
			class="m-auto p-10 flex transition border-2
		border-gray-300 border-dashed rounded-lg
		appearance-none cursor-pointer
		hover:border-gray-400 focus:outline-none"
		>
			<span class="flex items-center space-x-2">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="w-6 h-6 text-gray-200"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					stroke-width="2"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
					/>
				</svg>
				<span class="font-medium text-gray-400"> Upload an Image </span>
			</span>
			<input
				type="file"
				on:change={async (e) => {
					if (e.currentTarget.files) {
						await imageUpload(e.currentTarget.files[0]);
						draw();
						state = STATES.CONFIGURE;
					}
				}}
				class="hidden"
			/>
		</label>
	</div>
{:else if state == STATES.CONFIGURE}
	<div class="drawer z-50">
		<input id="my-drawer" type="checkbox" class="drawer-toggle" />
		<div class="drawer-content">
			<div class="grid">
				<label for="my-drawer" class="btn btn-primary drawer-button">Open drawer</label>
			</div>
		</div>
		<div class="drawer-side">
			<label for="my-drawer" class="drawer-overlay" />
			<ul class="menu p-4 w-lg h-full bg-base-200 text-base-content">
				<div class="max-w-2xl m-auto flex flex-col gap-2">
					<div>
						<div>
							<div class="flex gap-5 items-center">
								<h1 class="max-w-xl">Anchor Count</h1>
								<input
									type="range"
									min="0"
									max="500"
									bind:value={num_anchors}
									class="range range-primary"
								/>
								<input
									type="number"
									min="0"
									max="500"
									bind:value={num_anchors}
									class="input input-primary"
								/>
							</div>

							<div class="flex gap-5 items-center">
								<h1 class="max-w-xl">Anchor Gap Count</h1>
								<input
									type="range"
									min="0"
									max={num_anchors / 2 - 1}
									bind:value={num_anchor_gap}
									class="range range-primary"
								/>
								<input
									type="number"
									min="0"
									max={num_anchors / 2 - 1}
									bind:value={num_anchor_gap}
									class="input input-primary"
								/>
							</div>

							<div class="flex gap-5 items-center">
								<h1 class="max-w-xl">Image Radius</h1>
								<input
									type="range"
									min="0"
									max={maxRadius}
									bind:value={radius}
									class="range range-primary"
								/>
								<input
									type="number"
									min="0"
									max={maxRadius}
									bind:value={radius}
									class="input input-primary"
								/>
							</div>

							<div class="flex gap-5 items-center">
								<h1 class="max-w-xl">Chord Count</h1>
								<input
									type="range"
									min="0"
									max="2000"
									bind:value={num_chords}
									class="range range-primary"
								/>
								<input
									type="number"
									min="0"
									max="2000"
									bind:value={num_chords}
									class="input input-primary"
								/>
							</div>

							<div class="flex gap-5 items-center">
								<h1 class="max-w-xl">Line Opacity</h1>
								<input
									type="range"
									min="0"
									max="1"
									step="0.01"
									bind:value={line_opacity}
									class="range range-primary"
								/>
								<input
									type="number"
									min="0"
									max="1"
									bind:value={line_opacity}
									class="input input-primary"
								/>
							</div>
							<div class="grid">
								<button class="btn btn-primary" on:click={draw}>Primary</button>
							</div>
						</div>
					</div>
				</div>
			</ul>
		</div>
	</div>
{/if}
