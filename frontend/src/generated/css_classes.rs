use css_typegen::css_typegen;

// NOTE: Remember to edit index.html when adding new css-files!

// Generate rust types for css-classes.
// Used for autocompletion and extra compile-time checks.
css_typegen!(
    "frontend/static/styles/common.css",
    "frontend/static/styles/left_panel.css",
    "frontend/static/styles/ripple_spinner.css",
    "frontend/static/styles/filter_menu.css",
    "frontend/static/styles/charts.css",
    "frontend/static/styles/notifications.css",
    "frontend/static/styles/penguin.css",
    "frontend/static/styles/inventory.scss",
);
