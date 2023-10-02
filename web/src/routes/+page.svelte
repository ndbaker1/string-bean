<script>
	import { onMount } from 'svelte';

	import init, { planMoves } from 'string-bean';

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
	let num_anchors = 200;
	let num_anchor_gap = 10;
	let penalty = 500;
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
			default:
				return circle_anchors(num_anchors, width, height, Math.min(width, height) / 2);
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
		const isPortrait = windowRatio > 1;
		const maxSize = Math.max(
			250,
			(isPortrait ? window.innerHeight / 3 : window.innerWidth / 3) - 100
		);

		const imageRatio = imageSource.height / imageSource.width;

		// TODO: make this use the smaller dimension as the clamp
		const width = maxSize;
		const height = maxSize * imageRatio;

		imagePreview.width = imageMonochrome.width = imageDraw.width = width;
		imagePreview.height = imageMonochrome.height = imageDraw.height = height;

		/** @type CanvasRenderingContext2D  */
		// @ts-ignore
		const imagePreviewCtx = imagePreview.getContext('2d', { willReadFrequently: true });
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

		const moves = planMoves(
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

	/**
	 * @param {Event & { currentTarget: EventTarget & HTMLInputElement }} e
	 */
	async function upload(e) {
		if (e.currentTarget.files) {
			await imageUpload(e.currentTarget.files[0]);
			draw();
			state = STATES.CONFIGURE;
		}
	}
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
			<span class="indicator-item badge badge-secondary">monochrome</span>
			<canvas bind:this={imageMonochrome} />
		</div>
		<div class="join-item indicator">
			<span class="indicator-item badge badge-warning">output</span>
			<canvas bind:this={imageDraw} />
		</div>
	</div>
</div>

<!-- Image Source Element -->
<!-- svelte-ignore a11y-img-redundant-alt -->
<img bind:this={imageSource} class="hidden" alt="image-holder" />

{#if state == STATES.UPLOAD}
	<div class="w-screen h-screen flex p-4">
		<div class="m-auto flex flex-col">
			<h1 class="title text-5xl pb-10">🧵string-bean</h1>
			<div class="flex">
				<label for="upload-button" class="btn btn-outline btn-success m-auto">
					<span> Upload Image </span>
				</label>
				<input id="upload-button" type="file" on:change={upload} class="hidden" />
			</div>
		</div>
	</div>
{:else if state == STATES.CONFIGURE}
	<div class="flex flex-col p-8 gap-10 items-center">
		<div class="join">
			<label class="join-item btn btn-outline btn-success m-auto" for="upload-button">
				<span> Upload Another </span>
			</label>
			<input id="upload-button" type="file" on:change={upload} class="hidden" />
			<button class="join-item btn btn-outline btn-primary" on:click={() => draw()}>
				Redraw
			</button>
		</div>

		<div>
			<div class="grid max-w-3xl gap-2">
				<div class="flex gap-5 items-center">
					<div class="badge badge-info">
						<kbd class="whitespace-nowrap w-24 text-center">Anchor Count</kbd>
					</div>
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
						class="input input-sm input-bordered w-48"
					/>
				</div>

				<div class="flex gap-5 items-center">
					<div class="badge badge-info">
						<kbd class="whitespace-nowrap w-24 text-center">Anchor Gaps</kbd>
					</div>
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
						class="input input-sm input-bordered w-48"
					/>
				</div>

				<div class="flex gap-5 items-center">
					<div class="badge badge-info">
						<kbd class="whitespace-nowrap w-24 text-center">Line Count</kbd>
					</div>
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
						class="input input-sm input-bordered w-48"
					/>
				</div>

				<div class="flex gap-5 items-center">
					<div class="badge badge-info">
						<kbd class="whitespace-nowrap w-24 text-center">Line Opacity</kbd>
					</div>
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
						class="input input-sm input-bordered w-48"
					/>
				</div>

				<div class="flex gap-5 items-center">
					<div class="badge badge-info">
						<kbd class="whitespace-nowrap w-24 text-center"> Shape</kbd>
					</div>
					<select bind:value={shape} class="select select-bordered w-full max-w-xs">
						{#each Object.values(SHAPES) as s}
							<option>{s}</option>
						{/each}
					</select>
				</div>
			</div>
		</div>

		<h1 class="title">🧵string-bean</h1>
	</div>
{/if}

<style>
	.title {
		font-weight: bolder;
		background: -webkit-linear-gradient(#5fffc2, #0e5839);
		background-clip: text;
		-webkit-text-fill-color: transparent;
	}
</style>
