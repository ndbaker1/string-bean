// tailwind.config.js
module.exports = {
    mode: 'jit',
    content: ['./src/**/*.{html,js,svelte,ts}'],
    theme: {},
    daisyui: {
        themes: ["night"],
    },
    variants: {},
    plugins: [require('daisyui')],
};