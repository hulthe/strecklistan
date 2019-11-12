use css_typegen::css_typegen;

// NOTE: Remember to edit index.html when adding new css-files!

// Generate rust types for css-classes.
// Used for autocompletion and extra compile-time checks.
css_typegen!(
    "frontend/static/styles.css",
    "frontend/static/tailwind.css",
    "frontend/static/heart_spinner.css",
);
