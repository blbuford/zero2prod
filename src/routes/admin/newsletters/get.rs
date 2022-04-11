use actix_web::HttpResponse;
use actix_web::http::header::ContentType;

pub async fn get_newsletter_form() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta http-equiv="content-type" content="text/html; charset=utf-8">
<title>Change Password</title>
</head>
<body>
<form action="/admin/newletters" method="post">
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
        )
}
