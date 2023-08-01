<script>
	import init, { plan_as_json } from 'string-bean';
	import { onMount } from 'svelte';

	/** @type HTMLCanvasElement */
	// @ts-ignore
	let imagePreview;
	/** @type HTMLCanvasElement */
	// @ts-ignore
	let imageDraw;

	let num_chords = 1000;
	let line_opacity = 0.2;
	let num_anchors = 188;
	let num_anchor_gap = 20;
	let radius = 1000000;
	let penalty = 500;
	let start_anchor = 0;

	/**
	 * @param fileData {Blob}
	 */
	function imageUpload(fileData) {
		console.assert(!!FileReader);
		const fr = new FileReader();
		fr.onload = () => showImage(fr.result);
		fr.readAsDataURL(fileData);
	}

	// @ts-ignore
	function showImage(imageData) {
		const imageView = document.createElement('img');
		imageView.onload = () => {
			getImageData(imageView);
			imageView.remove();
		};
		imageView.src = imageData;
	}

	/**
	 * @param img {HTMLImageElement}
	 */
	function getImageData(img) {
		imagePreview.width = img.width;
		imagePreview.height = img.height;
		imageDraw.width = img.width;
		imageDraw.height = img.height;

		/** @type CanvasRenderingContext2D  */
		// @ts-ignore
		const imagePreviewCtx = imagePreview.getContext('2d');
		imagePreviewCtx.clearRect(0, 0, imagePreview.width, imagePreview.height);
		imagePreviewCtx.drawImage(img, 0, 0);

		const imageData = new Uint8Array(
			imagePreviewCtx.getImageData(0, 0, img.width, img.height).data.buffer
		);

		const tempImageData = [];
		for (let i = 0; i < img.width * img.height; i++) {
			tempImageData.push(imageData[i * 4]);
		}
		const newImageData = Uint8Array.from(tempImageData);

		const moves = JSON.parse(
			plan_as_json(
				num_chords,
				line_opacity,
				num_anchors,
				num_anchor_gap,
				radius,
				penalty,
				img.width,
				img.height,
				newImageData,
				start_anchor
			)
		);

		console.log(moves);

		const x_mid = imagePreview.width / 2;
		const y_mid = imagePreview.height / 2;

		const draw_radius = Math.min(x_mid, y_mid);
		const degrees_per_anchor = (2 * Math.PI) / num_anchors;

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

		hide = false;
	}

	let hide = true;

	onMount(init);
</script>

<svelte:head>
	<title>String-Bean</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<div class="w-screen h-screen flex">
	<label
		class="m-auto p-20 flex transition border-2
		border-gray-300 border-dashed rounded-lg
		appearance-none cursor-pointer
		hover:border-gray-400 focus:outline-none"
	>
		<span class="flex items-center space-x-2">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="w-6 h-6 text-gray-600"
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
			<span class="font-medium text-gray-400"> Upload an Image to Start </span>
		</span>
		<input
			type="file"
			on:change={(e) => {
				if (e.currentTarget.files) imageUpload(e.currentTarget.files[0]);
			}}
			class="hidden"
		/>
	</label>
</div>

<div class="max-w-2xl m-auto flex flex-col gap-2 hidden">
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
					max={num_anchors}
					bind:value={num_anchor_gap}
					class="range range-primary"
				/>
				<input
					type="number"
					min="0"
					max="2000"
					bind:value={num_anchor_gap}
					class="input input-primary"
				/>
			</div>

			<div class="flex gap-5 items-center">
				<h1 class="max-w-xl">Image Radius</h1>
				<input type="range" min="0" max="2000" bind:value={radius} class="range range-primary" />
				<input type="number" min="0" max="2000" bind:value={radius} class="input input-primary" />
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
		</div>
	</div>
</div>

<div class="max-w-fit flex m-auto {hide ? 'hidden' : ''}">
	<div>
		<div class="join bg-blue-400">
			<div class="indicator">
				<span class="indicator-item badge badge-primary">input</span>
				<canvas bind:this={imagePreview} class="join-item" />
			</div>
			<div class="indicator">
				<span class="indicator-item badge badge-secondary">output</span>
				<canvas bind:this={imageDraw} class="join-item" />
			</div>
		</div>
	</div>
</div>
