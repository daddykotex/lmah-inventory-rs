use maud::{Markup, html};

pub fn page(count: i64) -> Markup {
    html! {
        p { "got" (count) "clients"; }
    }
}