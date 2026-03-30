use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::server::{models::clients::ClientView, templates::utils::*};

fn find_clients(
    container_id: &str,
    input_id: &str,
    table_selector: &str,
    clear_selector: Option<&str>,
) -> Markup {
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
        (find_table(container_id, input_id, table_selector, clear_selector))
    }
}

fn action_col(client: &ClientView) -> Markup {
    let url = format!("/clients/{}", client.id);
    html! {
        a."btn btn-sm btn-primary" href=(url) {
            "Voir"
        }
    }
}

fn clients_table(clients: Vec<ClientView>) -> Markup {
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

struct ClientFormMarkup {
    body: Markup,
    javascript: Markup,
}

fn new_client_form(path: &str, maybe_client: Option<ClientView>) -> ClientFormMarkup {
    let maybe_first_name = maybe_client.clone().map(|s| s.first_name);
    let maybe_last_name = maybe_client.clone().map(|s| s.last_name);
    let maybe_street = maybe_client.clone().and_then(|s| s.street);
    let maybe_city = maybe_client.clone().and_then(|s| s.city);
    let maybe_phone = maybe_client.clone().map(|s| s.phone1);
    let maybe_phone2 = maybe_client.clone().and_then(|s| s.phone2);

    let body = html! {
        form."client-form" autocomplete="false" action=(path) method="POST" {
            div."form-row" {
                div."col-12 col-md-6 form-group" {
                    label for="firstname" {
                        "Prénom"
                    }
                    input."form-control" id="firstname" type="text" value=[maybe_first_name] name="firstname" required;
                    div."invalid-feedback" {
                        "Veuillez entrer un prénom"
                    }
                }
                div."col-12 col-md-6 form-group" {
                    label for="lastname" {
                        "Nom de famille"
                    }
                    input."form-control" id="lastname" required type="text" value=[maybe_last_name] name="lastname";
                    div."invalid-feedback" {
                        "Veuillez entrer un nom de famille"
                    }
                }
            }
            div."form-row" {
                div."col-12 col-md-6 form-group" {
                    label for="street" {
                        "Rue"
                    }
                    input."form-control" id="street" name="street" value=[maybe_street] type="text";
                    div."invalid-feedback" {
                        "Veuillez entrer la rue"
                    }
                }
                div."col-12 col-md-6 form-group" {
                    label for="city" {
                        "Ville"
                    }
                    input."form-control" id="city" value=[maybe_city] name="city" type="text";
                    div."invalid-feedback" {
                        "Veuillez entrer la ville"
                    }
                }
            }
            div."form-row" {
                div."col-12 col-md-6 form-group" {
                    label for="phone1" {
                        "Téléphone"
                    }
                    input."validate-phone form-control" id="phone1" value=[maybe_phone] name="phone1" required type="text";
                    div."invalid-feedback" {
                        "Veuillez entrer un téléphone valide: (555) 555-5555, poste 1234"
                    }
                }
                div."col-12 col-md-6 form-group" {
                    label for="phone2" {
                        "Téléphone #2"
                    }
                    input."validate-phone form-control" id="phone2" value=[maybe_phone2] name="phone2" type="text";
                    div."invalid-feedback" {
                        "Veuillez entrer un téléphone valide: (555) 555-5555, poste 1234"
                    }
                }
            }
            br;
            button."btn btn-primary btn-lg btn-block" type="submit" {
                "Sauvegarder"
            }
        }

    };
    let javascript = html! {
        (custom_form_validation())
        (client_form_helper())
    };
    ClientFormMarkup { body, javascript }
}

fn client_form_helper() -> Markup {
    html! {
        script type="text/javascript" src="/static/js/jquery.maskedinput.min.js" integrity="sha384-ATbYjrywZ6+DvHhy1i703oHUPV8MJxzAiIqrZpvMnwUNpsykpgEt8W3BkMhnHe7a" {}
        script type="text/javascript" {
            (PreEscaped(r#"
                $(document).ready(function() {
                    $('.client-form').each(function() {
                        var form = $(this);
                        customFormValidation(form);
                        $('#phone1', form).mask("(999) 999-9999?, poste: 99999");
                        $('#phone2', form).mask("(999) 999-9999?, poste: 99999");
                    });
                });
            "#))
        }
    }
}

fn new_client(form: Markup) -> Markup {
    html! {
        main role="main" {
            div."container-fluid" {
                div."row" {
                    div."col-12" {
                        (form)
                    }
                }
            }
        }
    }
}

fn list_clients(clients: Vec<ClientView>) -> Markup {
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
                        (clients_table(clients))
                    }
                }
            }
        }
    }
}

fn page(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="fr" {
            (head(title))

            body {
                (body)
            }
        }
    }
}

pub fn page_clients(clients: Vec<ClientView>) -> Markup {
    let body = html! {
        (navbar(MenuConstants::Clients))
        (list_clients(clients))
        (footer())
        (find_clients("clients-actions", "search", "table.find-client", None))
    };
    page("Clients", body)
}

pub fn page_one_client(client: ClientView) -> Markup {
    let update_url = format!("/clients/{}/update", client.id);
    let ClientFormMarkup {
        body: form_body,
        javascript,
    } = new_client_form(&update_url, Some(client));

    let body = html! {
        (navbar(MenuConstants::Clients))
        (new_client(form_body))
        (footer())
        (javascript)
    };
    page("Nouveau client", body)
}

pub fn page_new_client() -> Markup {
    let ClientFormMarkup {
        body: form_body,
        javascript,
    } = new_client_form("/clients/new", None);
    let body = html! {
        (navbar(MenuConstants::Clients))
        (new_client(form_body))
        (footer())
        (javascript)
    };
    page("Nouveau client", body)
}
