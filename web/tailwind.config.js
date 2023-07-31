// tailwind.config.js
module.exports = {
    mode: 'jit',
    purge: ['./src/**/*.svelte'],
    darkMode: false, // or 'media' or 'class'
    theme: {},
    variants: {},
    plugins: [require('daisyui')],
};