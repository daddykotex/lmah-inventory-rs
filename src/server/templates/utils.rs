use maud::{Markup, PreEscaped, html};

const BOOTSTRAP_VERSION: &str = "4.4.1";

pub fn price_input(id: &str, label: &str, value: &Option<i64>, required: bool) -> Markup {
    let required = if required { Some(true) } else { None };
    html! {
        label for=(id) {
            (label)
        }
        div."input-group" {
            div."input-group-prepend" {
                span."input-group-text" {
                    "$"
                }
            }
            input."form-control" id=(id) type="text" name=(id) value=[value] required=[required];
        }
    }
}

pub fn ask_transition(value: &str) -> String {
    match value {
        "RecordingOutDate" => "Enregistrer une date de sortie".to_string(),
        "RecordingTransfertToSeamstressDate" => {
            "Enregistrer une date de remise à la couturière".to_string()
        }
        "PlaceOrder" => "Enregister une date de commande".to_string(),
        "RecordExpectedDeliveryDate" => "Enregistrer une date attendue de livraison".to_string(),
        "RecordReceptionDate" => "Enregister une date de réception".to_string(),
        "RecordAdjustDate" => "Enregistrer une date de prise d'ajustements".to_string(),
        "RecordingOutForLocationDate" => "Enregister une date de sortie pour location".to_string(),
        "RecordingClientReturnDate" => "Enregistrer une date retour de location".to_string(),
        "TransfertToAlteration" => "Enregistrer une date de transfert en altération".to_string(),
        "RecordingBackFromSeamstressDate" => "Enregistrer une date de couture terminé".to_string(),
        "RecordingBackOrderDate" => "Enregistrer un avis Back Order".to_string(),
        "RecordingCancelDate" => "Enregistrer une date d'abandon".to_string(),
        _ => "Inconnue".to_string(),
    }
}

fn bootstrap_css() -> Markup {
    let url = format!(
        "https://stackpath.bootstrapcdn.com/bootstrap/{}/css/bootstrap.min.css",
        BOOTSTRAP_VERSION
    );
    html! {
        link href=(url) crossorigin="anonymous" integrity="sha384-Vkoo8x4CGsO3+Hhxv8T/Q5PaXtkKtu6ug5TOeNV6gBiFeWPGFN9MuhOf23Q9Ifjh" rel="stylesheet";
    }
}

fn bootstrap_js() -> Markup {
    let url = format!(
        "https://stackpath.bootstrapcdn.com/bootstrap/{}/js/bootstrap.min.js",
        BOOTSTRAP_VERSION
    );
    html! {
        script type="text/javascript" crossorigin="anonymous" integrity="sha384-wfSDF2E50Y2D1uUdj0O3uMBJnjuUD4Ih7YwaYd1iqfktj0Uod8GCExl3Og8ifwB6" src=(url) {}
    }
}

pub fn sidebar_info_box(title: &str, subtitle: Option<&str>, content: Markup) -> Markup {
    html! {
        div."card" {
            div."card-body" {
                h3."card-title" {
                    (title)
                }
                @if let Some(st) = subtitle {
                    h5."card-subtitle mb-2" {
                        (st)
                    }
                }
                div."card-text" {
                    (content)
                }
            }
        }
    }
}

pub fn find_table_with(
    container_id: &str,
    input_id: &str,
    search_fn: &str,
    clear_selector: Option<&str>,
) -> Markup {
    let selector_arg = match clear_selector {
        Some(selector) => format!("'{}'", selector),
        None => String::from("null"),
    };
    let search_script = format!(
        r#"
            $(document).ready(function() {{
                setupSearch({}, '{}', '{}', {})
            }});
        "#,
        search_fn, container_id, input_id, selector_arg
    );
    html! {
        script type="text/javascript" {
            (PreEscaped(search_script))
        }
    }
}

pub fn state(value: u8, label: Option<&str>, span_class: Option<&str>) -> Markup {
    let color = match value {
        1 => "red",
        2 => "orange",
        3 => "yellow",
        4 => "pink",
        5 => "blue",
        6 => "lightgreen",
        7 => "darkgreen",
        8 => "cyan",
        _ => "red,",
    };
    let text_color = match value {
        1 | 4 | 5 | 6 | 7 => "white",
        _ => "black",
    };
    let spacing = if label.is_none() { "" } else { "mr-1" };
    let class = format!(
        "state-tile state-color-{} text-{} {}",
        color, text_color, spacing
    );
    html! {
        span class=[span_class] {
            div class=(class) {
                (value)
            }
            @if let Some(label) = label {
                (label)
            }
        }
    }
}

pub fn find_table(
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
                function searchAll(value) {{
                    $(`{} tr:has(> td)`).filter(function() {{
                        $(this).toggle($(this).text().toLowerCase().indexOf(value) > -1);
                    }});
                }};
                setupSearch(searchAll, '{}', '{}', {})
            }});
        "#,
        table_selector, container_id, input_id, selector_arg
    );
    html! {
        script type="text/javascript" {
            (PreEscaped(search_script))
        }
    }
}

fn spinner() -> Markup {
    html! {
        div id="loading-overlay" {
            div."loading-overlay-image" {
                    img src="/static/icons/spinner.gif";
                }
            }
        script type="text/javascript" {
            r#"
                function showSpinner() {
                    $('#loading-overlay').show();
                }

                function hideSpinner() {
                    $('#loading-overlay').hide();
                }
            "#
        }
    }
}
#[derive(PartialEq)]
pub enum MenuConstants {
    Factures,
    Clients,
    Evenements,
    Help,
    Admin,
}

pub fn navbar_item(is_active: bool, url: &str, label: &str) -> Markup {
    let class = format!("nav-item {}", if is_active { "active" } else { "" });
    html! {
        li class=(class) {
            a."nav-link" href=(url) {
                (label)
            }
        }
    }
}

pub fn navbar(current_menu: MenuConstants) -> Markup {
    html! {
        nav."navbar navbar-expand-md navbar-dark bg-dark" {
            a."navbar-brand" href="/" {
                "La Mariée à l'Honneur"
            }
            button."navbar-toggler" aria-controls="menu" type="button" aria-label="Toggle navigation" aria-expanded="false" data-toggle="collapse" data-target="id="menu"" {
                span."navbar-toggler-icon" {}
            }
            div."collapse navbar-collapse" id="menu" {
                ul."navbar-nav mr-auto" {
                    (navbar_item(current_menu == MenuConstants::Clients, "/clients", "Clients"))
                    (navbar_item(current_menu == MenuConstants::Factures, "/factures", "Factures"))
                    (navbar_item(current_menu == MenuConstants::Evenements, "/events", "Événements"))
                }
                ul."navbar-nav flex-row" {
                    li."nav-item" {
                        a."nav-link" href="/help" {
                            "Aide"
                        }
                    }
                    li."nav-item" {
                        form method="POST" action="/signout" {
                            button."btn btn-link nav-link" type="submit" {
                                "Se déconnecter"
                            }
                        }
                    }
                }
                div."d-none" id="main-success" {
                    div."alert alert-success alert-dismissible fade show" role="alert" {
                        strong {
                            "Bravo! "
                        }
                        "L'action s'est terminé avec succès."
                        button."close" data-dismiss="alert" aria-label="Close" type="button" {
                            span aria-hidden="true" {
                                (PreEscaped("&times;"))
                            }
                        }
                    }
                }
                div."d-none" id="main-error" {
                    div."alert alert-danger alert-dismissible fade show" role="alert" {
                        strong {
                            "Oops"
                        }
                        span id="error-msg" {}
                        button."close" aria-label="Close" data-dismiss="alert" type="button" {
                            span aria-hidden="true" {
                                "&times;"
                            }
                        }
                    }
                }
            }
            script type="text/javascript" {
            (PreEscaped(r#"
                var urlParams = new URLSearchParams(window.location.search);
                var successParam = urlParams.get('success');
                if (successParam) {
                    document.getElementById("main-success").classList.remove("d-none");
                }

                var errorParam = urlParams.get('errorMsg');
                if (errorParam) {
                    document.getElementById("main-error").classList.remove("d-none");
                    document.getElementById("error-msg").innerText = errorParam;
                }
                setTimeout(function() {
                    var elem = document.getElementById("main-success");
                    if (elem) {
                    document.getElementById("main-success").parentNode.removeChild(elem);
                    }
                }, 5 * 1000);
            "#))
        }
        }
    }
}

pub fn head(title: &str) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no";
            link rel="shortcut icon" href="/static/icons/favicon.ico";
            link href="/static/icons/apple-touch-icon-57x57.png" sizes="57x57" rel="apple-touch-icon";
            link sizes="114x114" rel="apple-touch-icon" href="/static/icons/apple-touch-icon-114x114.png";
            link href="/static/icons/apple-touch-icon-72x72.png" rel="apple-touch-icon" sizes="72x72";
            link href="/static/icons/apple-touch-icon-144x144.png" sizes="144x144" rel="apple-touch-icon";
            link href="/static/icons/apple-touch-icon-60x60.png" sizes="60x60" rel="apple-touch-icon";
            link href="/static/icons/apple-touch-icon-120x120.png" rel="apple-touch-icon" sizes="120x120";
            link rel="apple-touch-icon" href="/static/icons/apple-touch-icon-76x76.png" sizes="76x76";
            link href="/static/icons/apple-touch-icon-152x152.png" rel="apple-touch-icon" sizes="152x152";
            link rel="icon" sizes="196x196" href="/static/icons/favicon-196x196.png" type="image/png";
            link type="image/png" sizes="160x160" rel="icon" href="/static/icons/favicon-160x160.png";
            link href="/static/icons/favicon-96x96.png" rel="icon" type="image/png" sizes="96x96";
            link type="image/png" sizes="16x16" href="/static/icons/favicon-16x16.png" rel="icon";
            link type="image/png" href="/static/icons/favicon-32x32.png" sizes="32x32" rel="icon";
            meta name="msapplication-TileColor" content="#ffffff";
            meta name="msapplication-TileImage" content="/static/icons/mstile-144x144.png";
            meta name="msapplication-square70x70logo" content="/static/icons/mstile-70x70.png";
            meta name="msapplication-square150x150logo" content="/static/icons/mstile-150x150.png";
            meta name="msapplication-square310x310logo" content="/static/icons/mstile-310x310.png";
            meta name="msapplication-wide310x150logo" content="/static/icons/mstile-310x150.png";
            title {
                (title)
            }

            (bootstrap_css())

            link href="https://code.jquery.com/ui/1.12.1/themes/base/jquery-ui.css" crossorigin="anonymous" rel="stylesheet" integrity="sha384-xewr6kSkq3dBbEtB6Z/3oFZmknWn7nHqhLVLrYgzEFRbU/DHSxW7K3B44yWUN60D";
            link href="https://cdnjs.cloudflare.com/ajax/libs/jquery.tablesorter/2.31.1/css/theme.bootstrap_4.min.css" crossorigin="anonymous" rel="stylesheet" integrity="sha384-llRRffiUCHjRjm6N1bzQQPdo1sd3zGB9VT6ZgrR1cI7rm/HuUxEdPjlpjJEITKS6";
            link rel="stylesheet" href="/static/css/style.css";
        }
    }
}

pub fn footer() -> Markup {
    html! {
        script src="https://code.jquery.com/jquery-3.4.1.min.js" crossorigin="anonymous" integrity="sha384-vk5WoKIaW/vJyUAd9n/wmopsmNhiy+L2Z+SBxGYnUkunIxVxAv/UtMOhba/xskxh" {}
        script src="https://cdn.jsdelivr.net/npm/popper.js@1.16.0/dist/umd/popper.min.js" integrity="sha384-Q6E9RHvbIyZFJoft+2mJbHaEWldlvI9IOYy5n3zV9zzTtmI3UksdQRVvoxMfooAo" crossorigin="anonymous" {}
        (bootstrap_js())
        script crossorigin="anonymous" integrity="sha384-JPbtLYL10d/Z1crlc6GGGGM3PavCzzoUJ1UxH0bXHOfguWHQ6XAWrIzW+MBGGXe5" src="https://code.jquery.com/ui/1.12.1/jquery-ui.js" type="text/javascript" {}
        script type="text/javascript" crossorigin="anonymous" integrity="sha384-QnFIXbEfAgO7z63b/aNzTSdSNLoNiNX/mk/Ok+NpUNMnvriWLo5cOtB0OUCAdKNu" src="https://cdnjs.cloudflare.com/ajax/libs/jquery.tablesorter/2.31.1/js/jquery.tablesorter.min.js" {}
        script type="text/javascript" src="/static/js/jquery-datepicker-fr.min.js" integrity="sha384-0l985W/1tDeBvpfVBzP0SeRoEpBScevfjJoBE5vKLb1/5TKQlDAks0Mj03AnIhAJ" {}
        script type="text/javascript" {
            (PreEscaped(r#"
            $(document).ready(function() {
                $('.date-picker').each(function (i, e) {
                var minDate = $(e).attr("data-min-date") === "true" ? new Date() : undefined;
                $(e).datepicker({
                    dateFormat: "yy-mm-dd",
                    minDate: minDate
                });
                });
            });
            "#))
        }
        script type="text/javascript" {
            (PreEscaped(r#"
                function setupSearch(searchFn, containerId, inputId, clearSelector) {
                    function setClass(inputLength) {
                        if (inputLength > 0) {
                            $(`#${containerId} .filtered-warning`).removeClass('d-none');
                            $(`#${containerId}`).addClass('colored-actions');
                        } else {
                            $(`#${containerId}`).removeClass('colored-actions');
                            $(`#${containerId} .filtered-warning`).addClass('d-none');
                        }
                    }

                    $(`#${inputId}`).on("keyup", function() {
                        var value = $(this).val().toLowerCase();
                        if (clearSelector !== null && value !== null) {
                            $(clearSelector).val('');
                        }

                        searchFn($(this).val().toLowerCase());
                        setClass(value.length);
                    });


                    searchFn($(`#${inputId}`).val().toLowerCase());
                    setClass($(`#${inputId}`).val().toLowerCase().length);
                }
            "#))
        }
        (spinner())
    }
}

pub fn custom_form_validation() -> Markup {
    html! {
        script type="text/javascript" {
            (PreEscaped(r#"
                function customFormValidation($form) {
                    var form = $form[0];
                    $form.submit(function(event) {
                        if (!form || form.checkValidity() === false) {
                            event.preventDefault();
                            event.stopPropagation();
                        }
                        $form.addClass('was-validated');
                    });
                };
            "#))
        }
    }
}
