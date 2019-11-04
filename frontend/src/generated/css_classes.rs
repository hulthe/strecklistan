use css_typegen::css_typegen;

// Generate rust types for css-classes.
// Used for autocompletion and extra compile-time checks.
css_typegen!("frontend/static/styles.css", "frontend/static/tailwind.css");
