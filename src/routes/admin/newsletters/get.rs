use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;
use uuid::Uuid;

pub async fn get_newsletter_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let idempotency_key = Uuid::new_v4().to_string();
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
<input hidden type="text" name="idempotency_key" value="{idempotency_key}">
<button type="submit">Publish</button>
</form>
<p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#,
        ))
}
