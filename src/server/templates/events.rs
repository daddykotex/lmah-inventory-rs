use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::server::{
    models::{FactureAndClient, events::EventView},
    templates::utils::*,
};

fn find_events(
    container_id: &str,
    input_id: &str,
    table_selector: &str,
    clear_selector: Option<&str>,
) -> Markup {
    html! {
        script type="text/javascript" {
            (PreEscaped(r#"
                $(document).ready(function(){
                    $("table.find-event").tablesorter({
                        theme : "bootstrap",
                        widthFixed: true
                    });
                });
            "#))
        }
        (find_table(container_id, input_id, table_selector, clear_selector))
    }
}

fn action_col(event: &EventView) -> Markup {
    let url = format!("/events/{}", event.id);
    html! {
        a."btn btn-sm btn-primary" href=(url) {
            "Voir"
        }
    }
}

pub fn events_table<F>(events: Vec<EventView>, action_col_f: F) -> Markup
where
    F: Fn(&EventView) -> Markup,
{
    html! {
        table."table table-sm find-event" {
            thead {
                tr {
                    th scope="col" {
                        "Actions"
                    }
                    th scope="col" {
                        "Type"
                    }
                    th scope="col" {
                        "Nom"
                    }
                    th scope="col" {
                        "Date"
                    }
                }
            }
            tbody {
                @for event in events {
                    tr {
                        td {
                            (action_col_f(&event))
                        }
                        td {
                            (event.event_type)
                        }
                        td {
                            (event.name)
                        }
                        td {
                            (event.date)
                        }
                    }
                }
            }
        }
    }
}

pub struct EventFormMarkup {
    pub body: Markup,
    pub javascript: Markup,
}

pub fn new_event_form(
    path: &str,
    maybe_event: Option<EventView>,
    event_types: &Vec<String>,
) -> EventFormMarkup {
    let maybe_name = maybe_event.clone().map(|s| s.name);
    let maybe_date = maybe_event.clone().map(|s| s.date);
    let maybe_type = maybe_event.clone().map(|s| s.event_type);

    let body = html! {
        form."evenement-form" autocomplete="false" action=(path) method="POST" {
            div."form-row form-group" {
                div."col-12" {
                    label for="type" {
                        "Type"
                    }
                    select."custom-select" id="type" name="type" {
                        @for event_type in event_types {
                            @let selected = if maybe_type.as_ref().is_some_and(|et| et == event_type)  { Some(true) } else { None };
                            option value=(event_type) selected=[selected] {
                                (event_type)
                            }
                        }
                    }
                }
            }
            div."form-row form-group" {
                div."col-12" {
                    label for="name" {
                        "Nom"
                    }
                    input."form-control" id="name" type="text" name="name" value=[maybe_name] required;
                }
            }
            div."form-row form-group" {
                div."col-12" {
                    label for="date" {
                        "Date"
                    }
                    input."form-control date-picker" id="date" type="text" data-min-date="true" name="date" value=[maybe_date] autocomplete="false" required;
                }
            }
            br;
            button."btn btn-primary" type="submit" {
                "Sauvegarder"
            }
        }

    };
    let javascript = html! {
        (custom_form_validation())
    };
    EventFormMarkup { body, javascript }
}

fn related_factures(items: Vec<FactureAndClient>) -> Markup {
    html! {
        @if items.is_empty() {
            p { "Aucune factures liées" }
        } @else {
            h3 {
                "Factures liées:"
            }
            table."table table-sm" {
                thead {
                    tr {
                        th scope="col" {
                            "Actions"
                        }
                        th scope="col" {
                            "No. de facture"
                        }
                        th scope="col" {
                            "Date facture"
                        }
                        th scope="col" {
                            "Nom du client"
                        }
                    }
                }
                tbody {
                    @for item in items {
                        @let url = format!("/factures/{}/items", item.facture.id);
                        tr {
                            td {
                                a."btn btn-sm btn-primary" href=(url) {
                                    "Voir"
                                }
                            }
                            td {
                                (item.facture.id)
                            }
                            td {
                                @if let Some(date) = item.facture.date {
                                    (date)
                                }
                            }
                            td {
                                (item.client.name())
                            }
                        }
                    }
                }
            }
        }
    }
}

fn new_event(form: Markup, related_factures: Markup) -> Markup {
    html! {
        main role="main" {
            div."container-fluid" {
                div."row" {
                    div."col-12" {
                        h3 {
                            "Détails d'un événement"
                        }
                    }
                }
                div."row" {
                    div."col-12" {
                        (form)
                    }
                }
                hr;
                div."row" {
                    div."col-12" {
                        (related_factures)
                    }
                }
            }
        }
    }
}

fn list_events(events: Vec<EventView>) -> Markup {
    html! {
        main role="main" {
            div."container-fluid" {
                div."row actions sticky-top" id="events-actions" {
                    div."col-12 col-sm-6 col-md-3" {
                        h4 {
                            "Liste des événements"
                        }
                    }
                    div."col-12 col-sm-6 col-md-9" {
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
                div."row" {
                    div."col-12" {
                        (events_table(events, action_col))
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

pub fn page_events(events: Vec<EventView>) -> Markup {
    let body = html! {
        (navbar(MenuConstants::Evenements))
        (list_events(events))
        (footer())
        (find_events("events-actions", "search", "table.find-event", None))
    };
    page("Événements", body)
}

pub fn page_one_event(
    event: EventView,
    event_types: Vec<String>,
    factures: Vec<FactureAndClient>,
) -> Markup {
    let event_name = event.name.clone();
    let update_url = format!("/events/{}/update", event.id);
    let EventFormMarkup {
        body: form_body,
        javascript,
    } = new_event_form(&update_url, Some(event), &event_types);

    // TODO retrieve related factures
    let related_factures = related_factures(factures);

    let body = html! {
        (navbar(MenuConstants::Evenements))
        (new_event(form_body, related_factures))
        (footer())
        (javascript)
    };
    let title = format!("{} - Événements", event_name);
    page(&title, body)
}

pub fn page_new_event(event_types: Vec<String>) -> Markup {
    let EventFormMarkup {
        body: form_body,
        javascript,
    } = new_event_form("/events/new", None, &event_types);
    let related_factures = related_factures(Vec::new());
    let body = html! {
        (navbar(MenuConstants::Evenements))
        (new_event(form_body, related_factures))
        (footer())
        (javascript)
    };
    page("Nouvel événement", body)
}
