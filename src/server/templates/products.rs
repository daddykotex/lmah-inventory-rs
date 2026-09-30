use maud::{DOCTYPE, Markup, html};

use crate::server::{
    models::products::{ProductAdminListEntry, ProductImageView, ProductView},
    services::products::ProductImagesPage,
    templates::utils::*,
    utils::money::format_cents,
};

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

fn shell(content: Markup) -> Markup {
    html! {
        (navbar(MenuConstants::Admin))
        main role="main" {
            div."container-fluid" {
                (content)
            }
        }
        (footer())
    }
}

pub fn page_admin_products_list(entries: Vec<ProductAdminListEntry>) -> Markup {
    let body = shell(html! {
        div."row actions sticky-top" id="products-actions" {
            div."col-12" {
                div."row mb-1" {
                    div."col-auto" {
                        h4 { "Produits" }
                    }
                    div."col-auto" {
                        a."btn btn-primary btn-sm" href="/admin/products/add-product" {
                            "Nouveau produit"
                        }
                    }
                }
                div."row" {
                    div."col-12" {
                        input."form-control" id="search" type="text" placeholder="Filtre";
                    }
                }
            }
        }
        div."row" {
            div."col-12" {
                table."table table-sm find-product" {
                    thead {
                        tr {
                            th { "Actions" }
                            th { "Nom" }
                            th { "Types" }
                            th { "Visible" }
                        }
                    }
                    tbody {
                        @for e in &entries {
                            tr {
                                td {
                                    a."btn btn-sm btn-primary mr-1" href=(format!("/admin/products/{}", e.id)) { "Éditer" }
                                    a."btn btn-sm btn-secondary" href=(format!("/admin/products/{}/images", e.id)) { "Images" }
                                }
                                td { (e.name) }
                                td { (e.types.join(", ")) }
                                td {
                                    @if e.visible_on_site { "Oui" } @else { "Non" }
                                }
                            }
                        }
                    }
                }
            }
        }
        (find_table("products-actions", "search", "table.find-product", None))
    });
    page("Produits", body)
}

fn product_form(
    action: &str,
    name: &str,
    price_display: &Option<String>,
    visible: bool,
    liquidation: bool,
    selected_types: &[String],
    all_types: &[String],
    submit_label: &str,
) -> Markup {
    html! {
        form."product-admin-form" action=(action) method="POST" {
            div."form-group" {
                label for="name" { "Nom" }
                input."form-control" id="name" name="name" type="text" required value=(name);
            }
            div."form-group" {
                label for="price" { "Prix (optionnel)" }
                div."input-group" {
                    div."input-group-prepend" {
                        span."input-group-text" { "$" }
                    }
                    input."form-control" id="price" name="price" type="text" value=[price_display.as_deref()];
                }
            }
            div."form-group" {
                div."form-check" {
                    input."form-check-input" id="visible-on-site" name="visible-on-site" type="checkbox" checked[visible];
                    label."form-check-label" for="visible-on-site" { "Visible sur le site" }
                }
                div."form-check" {
                    input."form-check-input" id="liquidation" name="liquidation" type="checkbox" checked[liquidation];
                    label."form-check-label" for="liquidation" { "Liquidation" }
                }
            }
            div."form-group" {
                label { "Types de produit" }
                @for t in all_types {
                    @let checked = selected_types.iter().any(|s| s == t);
                    @let id = format!("type-{}", t);
                    div."form-check" {
                        input."form-check-input" id=(id) name="types" type="checkbox" value=(t) checked[checked];
                        label."form-check-label" for=(id) { (t) }
                    }
                }
            }
            button."btn btn-primary" type="submit" { (submit_label) }
        }
    }
}

pub fn page_admin_product_new(all_types: Vec<String>) -> Markup {
    let body = shell(html! {
        div."row" {
            div."col-12 col-md-8" {
                h4 { "Nouveau produit" }
                (product_form(
                    "/admin/products/add-product",
                    "",
                    &None,
                    true,
                    false,
                    &[],
                    &all_types,
                    "Créer"
                ))
            }
        }
    });
    page("Nouveau produit", body)
}

pub fn page_admin_product_edit(
    product: ProductView,
    selected_types: Vec<String>,
    all_types: Vec<String>,
) -> Markup {
    let action = format!("/admin/products/{}", product.id);
    let images_url = format!("/admin/products/{}/images", product.id);
    let price_display = product.price.map(format_cents);
    let body = shell(html! {
        div."row" {
            div."col-12 col-md-8" {
                h4 { "Produit #" (product.id) }
                (product_form(
                    &action,
                    &product.name,
                    &price_display,
                    product.visible_on_site,
                    product.liquidation,
                    &selected_types,
                    &all_types,
                    "Sauvegarder"
                ))
                hr;
                a."btn btn-secondary" href=(images_url) { "Gérer les images" }
            }
        }
    });
    page("Produit", body)
}

fn image_card(position: &str, image: Option<&ProductImageView>) -> Markup {
    let label = match position {
        "front" => "Devant",
        "back" => "Derrière",
        other => other,
    };
    html! {
        div."col-12 col-md-6 mb-4" {
            h5 { (label) }
            @if let Some(img) = image {
                div."mb-2" {
                    img src=(img.url) alt=(label) style="max-width: 100%; max-height: 300px;";
                }
            } @else {
                div."alert alert-secondary" { "Aucune image" }
            }
            form action="" method="POST" enctype="multipart/form-data" {
                input type="hidden" name="position" value=(position);
                div."form-group" {
                    input."form-control-file" name="file" type="file" accept="image/*" required;
                }
                button."btn btn-primary btn-sm" type="submit" { "Téléverser" }
            }
        }
    }
}

pub fn page_admin_product_images(data: ProductImagesPage) -> Markup {
    let front = data.images.iter().find(|i| i.position == "front");
    let back = data.images.iter().find(|i| i.position == "back");
    let back_url = format!("/admin/products/{}", data.product.id);
    let body = shell(html! {
        div."row mb-2" {
            div."col-12" {
                h4 { "Images — " (data.product.name) }
                a."btn btn-link" href=(back_url) { "← Retour au produit" }
            }
        }
        div."row" {
            (image_card("front", front))
            (image_card("back", back))
        }
    });
    page("Images du produit", body)
}
