use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn get_newsletter_form(
    flash_messages: IncomingFlashMessages,
) -> HttpResponse {
    let mut message_html = String::new();
    for m in flash_messages.iter() {
        writeln!(message_html, "<p><i>{}</i></p>", m.content()).unwrap()
    }
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta http-equiv="content-type" content="text/html; charset=utf-8">
<title>Send a news letter</title>
</head>
<body>
{message_html}
<form action="/admin/newsletters" method="post">
<label>Title
<input
type="text"
placeholder="Enter newsletter title"
name="title"
>
</label>
<br>
<label>HTML Content
<input
type="text"
placeholder="Enter HTML content"
name="html"
>
</label>
<br>
<label>Enter text content
<input
type="text"
placeholder="Enter text content"
name="text"
>
</label>
<br>
<button type="submit">Send newsletter</button>
</form>
<p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#,
        ))
}
