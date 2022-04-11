use actix_web::HttpResponse;

pub async fn get_newsletter_form() -> HttpResponse {
    HttpResponse::Ok().finish()
}
