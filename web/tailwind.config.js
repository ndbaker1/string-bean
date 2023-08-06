// tailwind.config.js
module.exports = {
    mode: 'jit',
    content: ['./src/**/*.{html,js,svelte,ts}'],
    darkMode: false, // or 'media' or 'class'
    theme: {},
    daisyui: {
        themes: ["night"],
    },
    variants: {},
    plugins: [require('daisyui')],
};