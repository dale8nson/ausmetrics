module.exports = {
  purge: [],
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {},
  },
  content: {
    files: ["*.html", "./src/**/*.rs"], // Add this line to include Rust files
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
