use maud::{DOCTYPE, Markup, html};

use crate::server::templates::utils::*;

pub fn page(count: i64) -> Markup {
    html! {
        (DOCTYPE)
        html lang="fr" {
            (head("Clients"))

            p { "we got " (count) " clients"}

            (footer())
        }
    }
}
