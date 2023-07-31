<script>
	import init, { plan_as_json } from 'string-bean';
	import { onMount } from 'svelte';

	onMount(async () => {
		await init();

		const num_chords = 1000;
		const line_opacity = 0.2;
		const num_anchors = 188;
		const num_anchor_gap = 20;
		const radius = 1000000;
		const penalty = 500;
		const start_anchor = 0;

		const imageUpload = document.createElement('input');
		const imagePreview = document.createElement('canvas');
		const imageDraw = document.createElement('canvas');

		document.body.append(imageUpload, imagePreview, imageDraw);

		imageUpload.type = 'file';

		imageUpload.onchange = function (evt) {
			// @ts-ignore
			const files = evt.target.files;

			// FileReader support
			if (FileReader && files && files.length) {
				var fr = new FileReader();
				fr.onload = () => showImage(fr.result);
				fr.readAsDataURL(files[0]);
			}
		};

		// @ts-ignore
		function showImage(imageData) {
			const imageView = document.createElement('img');
			imageView.onload = () => {
				getImageData(imageView);
				imageView.remove();
			};
			imageView.src = imageData;
		}

		// @ts-ignore
		function getImageData(img) {
			imagePreview.width = img.width;
			imagePreview.height = img.height;
			imageDraw.width = img.width;
			imageDraw.height = img.height;

			/** @type CanvasRenderingContext2D  */
			const imagePreviewCtx = imagePreview.getContext('2d');
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
			const imageDrawCtx = imageDraw.getContext('2d');
			imageDrawCtx.strokeStyle = `rgba(0,0,0,${line_opacity})`;

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
	});
</script>

<svelte:head>
	<title>String-Bean</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section />
