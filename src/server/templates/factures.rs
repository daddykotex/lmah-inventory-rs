use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::server::{models::FactureWithRelatedData, templates::utils::*};

fn find_factures() -> Markup {
    html! {
        script type="text/javascript" {
            (PreEscaped(r#"
                $(document).ready(function() {
                    $("table.find-facture").tablesorter({
                        theme : "bootstrap",
                        widthFixed: true,
                        sortList: [[1, 1]]
                    });
                });
                function searchByNoFacture(value) {
                    $("table.find-facture tr:has(> td)").filter(function() {
                        var match =
                            $.makeArray($('td[data-search-no-facture]', $(this)))
                             .every(t => $(t).attr("data-search-no-facture").startsWith(value));

                        $(this).toggle(match);
                    });
                }
                document.searchByNoFacture = searchByNoFacture;
            "#))
        }
        (find_table("facture-actions", "search", "table.find-facture", Some("#search-no-facture")))
        (find_table_with("facture-actions", "search-no-facture", "searchByNoFacture", Some("#search")))
    }
}

fn seamstresses(data: Vec<String>) -> Markup {
    match data.as_slice() {
        [] => html! {},
        [one] => html! { (one) },
        all => html! {
            ul."list-unstyled ml-0" {
                @for s in all {
                    li {
                        (s)
                    }
                }
            }
        },
    }
}

fn list_factures(factures: Vec<FactureWithRelatedData>) -> Markup {
    html! {
        main role="main" {
            div."container-fluid" {
                div."row actions sticky-top" id="facture-actions" {
                    div."col-12" {
                        div."row mb-1" {
                            div."col-auto" {
                                h4 {
                                    "Liste de factures"
                                }
                            }
                            div."col-auto" {
                                button."btn btn-primary btn-sm" data-toggle="modal" data-target="#create-facture" {
                                    "Créer une nouvelle facture"
                                }
                            }
                        }
                        div."row" {
                            div."col-12 col-sm-4" {
                                input."form-control" id="search-no-facture" type="text" placeholder="Atteindre facture";
                            }
                            div."col-12 col-sm-8" {
                                input."form-control" id="search" type="text" placeholder="Filtre";
                            }
                        }
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
                        table."table table-sm table-striped find-facture" {
                            thead."sticky-top" {
                                tr {
                                    th scope="col" {
                                        "Actions"
                                    }
                                    th scope="col" {
                                        "No. de facture"
                                    }
                                    th scope="col" {
                                        "Ref. Ancienne"
                                    }
                                    th scope="col" {
                                        "Type"
                                    }
                                    th scope="col" {
                                        "Nom du client"
                                    }
                                    th scope="col" {
                                        "Date facture"
                                    }
                                    th scope="col" {
                                        "Couturière"
                                    }
                                    th scope="col" {
                                        "Statut"
                                    }
                                    th scope="col" {
                                        "Date statut"
                                    }
                                    th scope="col" {
                                        "État des items"
                                    }
                                }
                            }
                            tbody {
                                @for facture_data in factures {
                                    @let _seamstresses = facture_data.seamstresses();
                                    @let smallest_state = facture_data.smallest_state();

                                    @let url = format!("/factures/{}/items", facture_data.facture.id);
                                    @let facture_number =  facture_data.facture.id;
                                    @let facture_type =  facture_data.facture.facture_type;
                                    @let facture_date =  facture_data.facture.date;
                                    @let paper_ref = facture_data.facture.paper_ref;
                                    @let client_name = format!("{} {}", facture_data.client.first_name, facture_data.client.last_name);
                                    tr {
                                        td {
                                            a."btn btn-sm btn-primary" href=(url) {
                                                "Voir"
                                            }
                                        }
                                        td."no-facture" data-search-no-facture=(facture_number) {
                                            (facture_number)
                                        }
                                        td."no-facture" data-search-no-facture=[&paper_ref] {
                                            (&paper_ref.unwrap_or("".to_string()))
                                        }
                                        td {
                                            (facture_type.unwrap_or("N/A".to_string()))
                                        }
                                        td {
                                            (client_name)
                                        }
                                        td {
                                            (facture_date.unwrap_or("N/A".to_owned()))
                                        }
                                        td {
                                            (seamstresses(_seamstresses))
                                        }
                                        @match smallest_state {
                                            Some(s) =>  {
                                                td {
                                                    (state(s.state.value(), Some(s.state.label()), None))
                                                }
                                                td {
                                                    @if let Some(date) = s.state.date() {
                                                        (date)
                                                    }
                                                }
                                            },
                                            None => {
                                                td { }
                                                td { }
                                            },
                                        }
                                        td {
                                            @for (_, s) in facture_data.state_per_item {
                                                (state(s.state.value(), None, Some("ml-1")))
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div."modal fade" id="create-facture" aria-hidden="true" aria-labelledby="create-facture-label" tabindex="-1" role="dialog" {
                div."modal-dialog" role="document" {
                    div."modal-content" {
                        div."modal-header" {
                            h5."modal-title" id="create-facture-label" {
                                "Type de facture"
                            }
                            button."close" type="button" aria-label="Close" data-dismiss="modal" {
                                span aria-hidden="true" {
                                    (PreEscaped("&times;"))
                                }
                            }
                        }
                        div."modal-footer" {
                            button."btn btn-secondary" type="button" data-dismiss="modal" {
                                "Annuler"
                            }
                            a."btn btn-primary" href="/factures/new?facture-type=alteration" {
                                "Altération"
                            }
                            a."btn btn-primary" href="/factures/new?facture-type=products" {
                                "Produits"
                            }
                            a."btn btn-primary" href="/factures/new?facture-type=location" {
                                "Location"
                            }
                        }
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

pub fn page_factures(factures: Vec<FactureWithRelatedData>) -> Markup {
    let body = html! {
        (navbar(MenuConstants::Factures))
        (list_factures(factures))
        (footer())
        (find_factures())
    };
    page("Factures", body)
}
