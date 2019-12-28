use css_typegen::css_typegen;

// NOTE: Remember to edit index.html when adding new css-files!

// Generate rust types for css-classes.
// Used for autocompletion and extra compile-time checks.
css_typegen!(
    "frontend/static/styles.css",
    "frontend/static/left_panel.css",
    "frontend/static/heart_spinner.css",
    "frontend/static/filter_menu.css",
    "frontend/static/tailwind.css",
);
