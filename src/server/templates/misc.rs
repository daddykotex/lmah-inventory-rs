use maud::{DOCTYPE, Markup, html};

use crate::server::templates::utils::{bootstrap_css};


fn signin() -> Markup {
    html! {
        body."text-center" {
            div."form-signin" {
                a."btn btn-lg btn-primary btn-block" href="/signin" {
                    "Se connecter"
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


pub fn page_signin() -> Markup {
    html! {
        (DOCTYPE)
        html lang="fr" {
            (head_signin())

            body {
                (signin())
            }
        }
    }
}
