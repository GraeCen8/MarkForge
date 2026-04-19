//this is used to add css styles that can make it look cleaner
const CSS: &str = include_str!("styles.css");

pub fn wrap_html(content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
{}
</style>
</head>
<body>
{}
</body>
</html>"#,
        CSS, content
    )
}
