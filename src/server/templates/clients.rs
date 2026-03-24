use maud::{Markup, html};

use crate::server::templates::utils::*;

pub fn page(count: i64) -> Markup {
    html! {
        html lang="fr" {
            (head("Clients"))

            p { "we got " (count) " clients"}

            (footer())
        }
    }
}
