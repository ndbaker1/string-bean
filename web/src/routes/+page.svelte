<script>
	import { onMount } from 'svelte';

	import init, { json_plan } from 'string-bean';

	import { circle_anchors } from '$lib/utils';
	import { rectangle_anchors } from '../lib/utils';

	/** @type HTMLCanvasElement */
	// @ts-ignore
	let imagePreview;
	/** @type HTMLCanvasElement */
	// @ts-ignore
	let imageMonochrome;
	/** @type HTMLCanvasElement */
	// @ts-ignore
	let imageDraw;
	/** @type HTMLImageElement */
	// @ts-ignore
	let imageSource;

	/**
	 * Settings
	 */
	let line_count = 500;
	let line_opacity = 0.2;
	let num_anchors = 188;
	let num_anchor_gap = 0;
	let penalty = 100;
	let start_anchor = 0;

	$: num_anchor_gap = Math.min(num_anchor_gap, num_anchors / 2 - 1);

	const SHAPES = {
		CIRCLE: 'Circle',
		RECTANGLE: 'Rectangle'
	};
	let shape = SHAPES.CIRCLE;

	/**
	 * @param {number} width
	 * @param {number} height
	 */
	function getAnchors(width, height) {
		switch (shape) {
			case SHAPES.CIRCLE:
				return circle_anchors(num_anchors, width, height, Math.min(width, height) / 2);
			case SHAPES.RECTANGLE:
				return rectangle_anchors(num_anchors, width, height);
		}
	}

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
				// block until finished
				resolve(null);
			};
			imageSource.src = imageURL;
		});
	}

	function draw() {
		const maxSize = Math.max(
			250,
			(windowRatio > 1 ? window.innerHeight / 3 : window.innerWidth / 3) - 100
		);

		const imageRatio = imageSource.height / imageSource.width;

		const width = imageRatio > 1 ? maxSize * imageRatio : maxSize;
		const height = imageRatio > 1 ? maxSize : maxSize * imageRatio;

		imagePreview.width = imageMonochrome.width = imageDraw.width = width;
		imagePreview.height = imageMonochrome.height = imageDraw.height = height;

		/** @type CanvasRenderingContext2D  */
		// @ts-ignore
		const imagePreviewCtx = imagePreview.getContext('2d');
		imagePreviewCtx.clearRect(0, 0, width, height);
		imagePreviewCtx.drawImage(imageSource, 0, 0, width, height);

		// update imagedata buffer to make image monochrome
		const imageData = imagePreviewCtx.getImageData(0, 0, width, height);
		for (let i = 0; i < imageData.data.length; i += 4) {
			const sum = (imageData.data[i] + imageData.data[i + 1] + imageData.data[i + 2]) / 3;
			imageData.data[i] = imageData.data[i + 1] = imageData.data[i + 2] = sum;
			imageData.data[i + 3] = 255;
		}

		/** @type CanvasRenderingContext2D  */
		// @ts-ignore
		const imageMonochromeCtx = imageMonochrome.getContext('2d');
		imageMonochromeCtx.clearRect(0, 0, width, height);
		imageMonochromeCtx.putImageData(imageData, 0, 0);

		const monochromeImageBuffer = new Uint8Array(imageData.data.filter((_, i) => i % 4 == 0));

		const anchorCoords = getAnchors(width, height);

		const moves = json_plan(
			line_count,
			line_opacity,
			new Float64Array(anchorCoords.flat()),
			num_anchor_gap,
			penalty,
			width,
			height,
			monochromeImageBuffer,
			start_anchor
		);

		/** @type CanvasRenderingContext2D  */
		// @ts-ignore
		const imageDrawCtx = imageDraw.getContext('2d');
		imageDrawCtx.strokeStyle = `rgba(0,0,0,${line_opacity})`;

		imageDrawCtx.clearRect(0, 0, width, height);
		imageDrawCtx.beginPath();
		for (let i = 1; i < moves.length; i++) {
			const [x0, y0] = anchorCoords[moves[i - 1]];
			const [x1, y1] = anchorCoords[moves[i]];
			imageDrawCtx.moveTo(x0, y0);
			imageDrawCtx.lineTo(x1, y1);
		}
		imageDrawCtx.stroke();
	}

	const STATES = { UPLOAD: 0, CONFIGURE: 1 };
	let state = STATES.UPLOAD;

	let windowRatio = 1;

	onMount(async () => {
		await init();
		windowRatio = window.innerHeight / window.innerWidth;
	});
</script>

<svelte:head>
	<title>string-bean</title>
	<meta name="description" content="thread art generator demonstration" />
</svelte:head>

<!-- Canvas Elements -->
<div class="max-w-fit m-auto {state == STATES.UPLOAD ? 'hidden' : ''}">
	<div class="join {windowRatio < 1 ? 'join' : 'join-vertical'} bg-white m-8">
		<div class="join-item indicator">
			<span class="indicator-item badge badge-primary">input</span>
			<canvas bind:this={imagePreview} />
		</div>
		<div class="join-item indicator">
			<span class="indicator-item badge badge-primary">monochrome</span>
			<canvas bind:this={imageMonochrome} />
		</div>
		<div class="join-item indicator">
			<span class="indicator-item badge badge-secondary">output</span>
			<canvas bind:this={imageDraw} />
		</div>
	</div>
</div>

<!-- Image Source Element -->
<!-- svelte-ignore a11y-img-redundant-alt -->
<img bind:this={imageSource} class="hidden" alt="image-holder" />

{#if state == STATES.UPLOAD}
	<div class="w-screen h-screen flex p-4">
		<label
			class="m-auto px-10 py-4 flex transition border-2
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
	<div class="flex flex-col px-8 gap-10 items-center">
		<div class="join">
			<button class="btn btn-info join-item" on:click={() => (state = STATES.UPLOAD)}>
				Upload Another
			</button>
			<button class="btn btn-primary join-item" on:click={() => draw()}> Redraw </button>
		</div>

		<div>
			<div class="grid max-w-3xl gap-5">
				<div class="flex gap-5 items-center">
					<kbd class="kbd">Anchor Count</kbd>
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
						class="input input-primary w-48"
					/>
				</div>

				<div class="flex gap-5 items-center">
					<kbd class="kbd">Anchor Gaps</kbd>
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
						max={Math.round(num_anchors / 2) - 1}
						bind:value={num_anchor_gap}
						class="input input-primary w-48"
					/>
				</div>

				<div class="flex gap-5 items-center">
					<kbd class="kbd">Chord Count</kbd>
					<input
						type="range"
						min="0"
						max="2000"
						bind:value={line_count}
						class="range range-primary"
					/>
					<input
						type="number"
						min="0"
						max="2000"
						bind:value={line_count}
						class="input input-primary w-48"
					/>
				</div>

				<div class="flex gap-5 items-center">
					<kbd class="kbd">Line Opacity</kbd>
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
						class="input input-primary w-48"
					/>
				</div>

				<div class="flex gap-5 items-center">
					<kbd class="kbd">Shape</kbd>
					<select bind:value={shape} class="select select-bordered w-full max-w-xs">
						{#each Object.values(SHAPES) as s}
							<option>{s}</option>
						{/each}
					</select>
				</div>
			</div>
		</div>
	</div>
{/if}

<footer class="p-5" />
