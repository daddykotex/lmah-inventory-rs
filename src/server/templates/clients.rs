use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::server::templates::utils::*;

fn find_clients(
    container_id: &str,
    input_id: &str,
    table_selector: &str,
    clear_selector: Option<&str>,
) -> Markup {
    let selector_arg = match clear_selector {
        Some(selector) => format!("'{}'", selector),
        None => String::from("null"),
    };
    let search_script = format!(
        r#"
            $(document).ready(function() {{
                setupSearch('{}', '{}', '{}', {})
            }});
        "#,
        container_id, input_id, table_selector, selector_arg
    );
    html! {
        script type="text/javascript" {
            (PreEscaped(r#"
                $(document).ready(function(){
                    $("table.find-client").tablesorter({
                        theme : "bootstrap",
                        sortList: [[1,0], [2,0]]
                    });
                });
            "#))
        }
        script type="text/javascript" {
            (PreEscaped(search_script))
        }
    }
}

struct Client {
    id: String,
    first_name: String,
    last_name: String,
}

fn action_col(client: &Client) -> Markup {
    let url = format!("/clients/{}", client.id);
    html! {
        a."btn btn-sm btn-primary" href=(url) {
            "Voir"
        }
    }
}

fn clients_table(count: i64) -> Markup {
    let mut clients = Vec::with_capacity(count.try_into().ok().unwrap());
    for i in 0..count {
        clients.push(Client {
            id: format!("id-{}", i),
            first_name: format!("fname-{}", i),
            last_name: format!("lname-{}", i),
        })
    }

    html! {
        table."table table-sm find-client" {
            thead {
                tr {
                    th scope="col" {
                        "Actions"
                    }
                    th scope="col" {
                        "Nom"
                    }
                    th scope="col" {
                        "Prénom"
                    }
                }
            }
            tbody {
                @for client in clients {
                    tr {
                        td {
                            (action_col(&client))
                        }
                        td {
                            (client.last_name)
                        }
                        td {
                            (client.first_name)
                        }
                    }
                }
            }
        }
    }
}

fn main(count: i64) -> Markup {
    html! {
        main role="main" {
            div."container-fluid" {
                div."row actions sticky-top" id="clients-actions" {
                    div."col-12" {
                        div."row mb-1" {
                            div."col-auto" {
                                h4 {
                                    "Liste de clients"
                                }
                            }
                            div."col-auto" {
                                a."btn btn-primary btn-sm" href="/clients/new" {
                                    "Nouveau client"
                                }
                            }
                        }
                        div."row" {
                            div."col-12" {
                                input."form-control" id="search" type="text" placeholder="Filtre";
                            }
                            div."filtered-warning col-12 d-none" {
                                span {
                                    b {
                                        "Affichage filtré"
                                    }
                                }
                            }
                        }
                    }
                }
                div."row" {
                    div."col-12" {
                        (clients_table(count))
                    }
                }
            }
        }
    }
}

pub fn page(count: i64) -> Markup {
    html! {
        (DOCTYPE)
        html lang="fr" {
            (head("Clients"))

            body {
                (navbar(MenuConstants::Clients))
                (main(count))
                (footer())
                (find_clients("clients-actions", "search", "table.find-client", None))
            }
        }
    }
}
