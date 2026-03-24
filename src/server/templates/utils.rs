use maud::{Markup, PreEscaped, html};

const BOOTSTRAP_VERSION: &str = "4.4.1";

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
        (spinner())
    }
}
