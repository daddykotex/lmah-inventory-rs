use axum::response::{IntoResponse, Redirect, Response};
pub enum RedirectOr<T> {
    Redirect(String),
    Response(T),
}

impl<T> IntoResponse for RedirectOr<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            RedirectOr::Redirect(url) => Redirect::to(&url).into_response(),
            RedirectOr::Response(res) => res.into_response(),
        }
    }
}
