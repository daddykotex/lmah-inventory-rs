use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::server::{
    models::{PageAdmin, config::ExtraLargeAmounts},
    templates::utils::*,
};

fn signin(url: &str) -> Markup {
    let url = format!("/signin?redirect_url={}", url);
    html! {
        body."text-center" {
            form method="POST" action=(url) {
                div."form-signin" {
                    button."btn btn-lg btn-primary btn-block" type="submit" {
                        "Se connecter"
                    }
                }
            }
        }
    }
}

fn head_signin() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no";
            meta name="description" content="";
            meta name="author" content="";
            link rel="icon" href="/favicon.ico";
            title {
                "Bienvenue"
            }
            (bootstrap_css())
            style {
                "html,body {
                    height: 100%;
                }
                body {
                    display: -ms-flexbox;
                    display: -webkit-box;
                    display: flex;
                    -ms-flex-align: center;
                    -ms-flex-pack: center;
                    -webkit-box-align: center;
                    align-items: center;
                    -webkit-box-pack: center;
                    justify-content: center;
                    padding-top: 40px;
                    padding-bottom: 40px;
                    background-color: #f5f5f5;
                }
                .form-signin {
                    width: 100%;
                    max-width: 330px;
                    padding: 15px;
                    margin: 0 auto;
                }"
            }
        }
    }
}

pub fn page_signin(url: &str) -> Markup {
    html! {
        (DOCTYPE)
        html lang="fr" {
            (head_signin())

            body {
                (signin(url))
            }
        }
    }
}

fn page(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="fr" {
            (head(title))

            body {
                (content)
            }
        }
    }
}

pub fn page_admin(page_data: PageAdmin) -> Markup {
    let sort_table_script = PreEscaped(
        r#"
        $(document).ready(function(){
            $("table.admin-find-facture").tablesorter({
                theme : "bootstrap",
                widthFixed: true,
                sortList: [[1, 1]]
            });
        });
    "#,
    );
    let content = html! {
        main role="main" {
            div."container-fluid" {
                div."row" {
                    div."col-12 col-md-6" {
                        h2 {
                            "Rapport de paiements"
                        }
                        p {
                            "Ce rapport contient l'information de toutes les transactions. Il n'inclut pas les factures sans paiements."
                        }
                        a href="/admin/paiements-report" {
                            "Télécharger le fichier CSV"
                        }
                    }
                    div."col-12 col-md-6" {
                        h2 {
                            "Rapport de solde à payer"
                        }
                        p {
                            "Ce rapport contient toutes factures. Il n'inclut pas les paiements."
                        }
                        a href="/admin/factures-report" {
                            "Télécharger le fichier CSV"
                        }
                    }
                }
                hr;
                div."row" {
                    div."col-12" {
                        h2 {
                            "Factures"
                        }
                        table."table table-sm table-striped admin-find-facture" {
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
                                        "Nom du client"
                                    }
                                    th scope="col" {
                                        "Date facture"
                                    }
                                }
                            }
                            tbody {
                                @for fc in &page_data.factures {
                                    tr {
                                        td {
                                            button."generate-print btn btn-success" type="button" data-facture-id=(fc.facture.id) {
                                                "Visualiser"
                                            }
                                        }
                                        td."no-facture" data-search-no-facture=(fc.facture.id) {
                                            (fc.facture.id)
                                        }
                                        td."no-facture" data-search-no-facture=(fc.facture.id) {
                                            @if let Some(pr) = &fc.facture.paper_ref {
                                                (pr)
                                            }
                                        }
                                        td {
                                            (fc.client.name())
                                        }
                                        td {
                                            @if let Some(d) = &fc.facture.date {
                                                (d)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    let body = html! {
        (navbar(MenuConstants::Admin))
        (content)
        (footer())
        script type="text/javascript" {
            (sort_table_script)
            (generate_print_js())
        }
        (generate_print_js())
    };
    page("Administration", body)
}

pub fn page_help(event_types: Vec<String>, extra: ExtraLargeAmounts) -> Markup {
    let version = option_env!("VERSION").unwrap_or("dev-build");
    let content = html! {
        main role="main" {
            div."container-fluid help" {
                div."row" {
                    div."col-12 col-md-6" {
                        h2 {
                            "Info"
                        }
                        pre {
                            code {
                                "Version: " (version) "\n"
                                "Types d'évènements: \n"
                                @for et in event_types {
                                    (et) "\n"
                                }
                                "Taille forte: \n"
                                "   Robe de mariées: " (&extra.wedding) "\n"
                                "   Autres: " (&extra.others)
                            }
                        }
                    }
                    div."col-12 col-md-6" {
                        h2 {
                            "Lien utiles"
                        }
                        ul {
                            li {
                                a href="/admin" {
                                    "Administration"
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    let body = html! {
        (navbar(MenuConstants::Help))
        (content)
        (footer())
    };
    page("Aide", body)
}

pub fn page_wait() -> Markup {
    html! {
        (DOCTYPE)
        html lang="fr" {
            (head("Veuillez patienter"))

            body {
                main role="main" {
                    div."container-fluid" {
                        div."row" {
                            div."col-12 col-md-6" {
                                h1 {
                                    "Veuillez patienter"
                                }
                            }
                        }
                    }
                }
                (footer())
                script {
                    "showSpinner();"
                }
            }
        }
    }
}
