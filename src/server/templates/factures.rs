use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::server::{
    models::{
        FactureDashboardData, FactureItemEntry, FactureItemsData, PageFactureItemsData,
        facture_items::{FactureItemType, FactureItemView},
        products::ProductView,
        statuts::StateView,
    },
    templates::utils::*,
};

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

fn the_items_action_col(
    facture_type: &Option<String>,
    location_item_id: i64,
    alteration_item_id: i64,
) -> Markup {
    let default = String::from("Product");
    let f = facture_type.as_ref().unwrap_or(&default);
    let f = f.as_str();
    let url = match f {
        "Location" => format!("/factures/rec123/add-item/{}", location_item_id),
        "Alteration" => format!("/factures/rec123/add-item/{}", alteration_item_id),
        _ => "/factures/rec123/add-item".to_string(),
    };
    html! {
        a."btn btn-primary" href=(url) {
            "Ajouter un item"
        }
    }
}

fn list_the_items_row(entry: FactureItemEntry) -> Markup {
    match entry.item.value {
        FactureItemType::FactureItemProduct(value) => {
            let item_url = format!("/factures/{}/items/{}", value.facture_id, value.id);
            let delete_url = format!("{}/delete", item_url);
            html! {
                tr {
                    td scope="row" {
                        a."btn btn-sm btn-primary" href=(item_url) {
                            "Voir"
                        }
                        form."inline-button" method="POST" action=(delete_url) {
                            button."btn btn-sm btn-danger" type="submit" {
                                "Retirer"
                            }
                        }
                    }
                    td {
                        (value.quantity)
                    }
                    td {
                        @if let Some(b) = value.beneficiary {
                            (b)
                        }
                    }
                    td {
                        (entry.product.name)
                    }
                    td {
                        @if let Some(p) = value.price {
                            (p)
                        }
                    }
                    td {
                        @if let Some(p) = value.rebate_percent {
                            (p) "%"
                        }
                    }
                    td {
                        @if let Some(ex) = value.extra_large_size {
                            (ex) "$"
                        }
                    }
                    td {
                        (state(entry.state.state.value(), Some(entry.state.state.label()), None))
                    }
                }
            }
        }
        FactureItemType::FactureItemLocation(value) => html! {},
        FactureItemType::FactureItemAlteration(value) => html! {},
    }
}

fn list_the_items(facture_data: FactureItemsData) -> Markup {
    let default = String::from("Product");
    let facture_type = facture_data
        .facture
        .facture_type
        .as_ref()
        .unwrap_or(&default)
        .as_str();
    let header = match facture_type {
        "Location" => html! {
            thead {
                tr {
                    th scope="col" {
                        "Actions"
                    }
                    th scope="col" {
                        "Quantité"
                    }
                    th scope="col" {
                        "Bénéficiaire"
                    }
                    th scope="col" {
                        "Prix unitaire"
                    }
                    th scope="col" {
                        "Assurances"
                    }
                    th scope="col" {
                        "Autre frais"
                    }
                    th."file" scope="col" {
                        "Fichier"
                    }
                    th scope="col" {
                        "Statut"
                    }
                }
            }
        },
        "Alteration" => html! {
            thead {
                tr {
                    th scope="col" {
                        "Actions"
                    }
                    th scope="col" {
                        "Quantité"
                    }
                    th scope="col" {
                        "Nom"
                    }
                    th scope="col" {
                        "Prix unitaire"
                    }
                    th scope="col" {
                        "Rabais ($)"
                    }
                    th scope="col" {
                        "Statut"
                    }
                }
            }
        },
        _ => html! {
            thead {
                tr {
                    th scope="col" {
                        "Actions"
                    }
                    th scope="col" {
                        "Quantité"
                    }
                    th scope="col" {
                        "Bénéficiaire"
                    }
                    th scope="col" {
                        "Nom"
                    }
                    th scope="col" {
                        "Prix unitaire"
                    }
                    th scope="col" {
                        "Rabais (%)"
                    }
                    th scope="col" {
                        "Taille forte"
                    }
                    th scope="col" {
                        "Statut"
                    }
                }
            }
        },
    };
    html! {
        @if facture_data.items.iter().count() > 0 {
            table."table table-sm items" {
                (header)
                tbody {
                    @for entry in facture_data.items {
                        (list_the_items_row(entry))
                    }
                }
            }

        } @else {
            p { "Il n'y a aucun item dans la facture." }
        }
    }
}

fn the_items(page_data: PageFactureItemsData) -> Markup {
    html! {
        main role="main" {
            div."container-fluid" {
                div."row" {
                    div."col-12 order-12 col-lg-8 order-lg-1" {
                        div."row actions sticky-top" {
                            div."col-12" {
                                (the_items_action_col(&page_data.facture_data.facture.facture_type, page_data.location_product.id, page_data.alteration_product.id))
                            }
                        }
                        table."table table-sm items" {
                            thead {
                                tr {
                                    th scope="col" {
                                        "Actions"
                                    }
                                    th scope="col" {
                                        "Quantité"
                                    }
                                    th scope="col" {
                                        "Bénéficiaire"
                                    }
                                    th scope="col" {
                                        "Nom"
                                    }
                                    th scope="col" {
                                        "Prix unitaire"
                                    }
                                    th scope="col" {
                                        "Rabais (%)"
                                    }
                                    th scope="col" {
                                        "Taille forte"
                                    }
                                    th scope="col" {
                                        "Statut"
                                    }
                                }
                            }
                            tbody {
                                tr {
                                    td scope="row" {
                                        a."btn btn-sm btn-primary" href="/factures/rec123/items/rec123" {
                                            "Voir"
                                        }
                                        form."inline-button" method="POST" action="/factures/rec123/items/rec123/delete" {
                                            button."btn btn-sm btn-danger" type="submit" {
                                                "Retirer"
                                            }
                                        }
                                    }
                                    td {
                                        "1"
                                    }
                                    td {
                                        "Marie"
                                    }
                                    td {
                                        "Robe de mariée élégante"
                                    }
                                    td {
                                        "1500.00"
                                    }
                                    td {
                                        "0 %"
                                    }
                                    td {}
                                    td {
                                        span {
                                            div."state-tile state-color-red text-white" {
                                                "1"
                                            }
                                            "À commander"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div."col-12 order-1 col-lg-4 order-lg-12" {
                        div."card" {
                            div."card-body" {
                                h3."card-title" {
                                    "Détails de la facture"
                                }
                                h5."card-subtitle mb-2" {
                                    "Facture #1001"
                                }
                                div."card-text" {
                                    ul."ml-0 list-unstyled" {
                                        li {
                                            b {
                                                "Client:"
                                            }
                                            "Marie Tremblay"
                                        }
                                        li {
                                            b {
                                                "Ville:"
                                            }
                                            "Montréal"
                                        }
                                        li {
                                            b {
                                                "Téléphone:"
                                            }
                                            "514-555-1234"
                                        }
                                        li {
                                            b {
                                                "Téléphone #2"
                                            }
                                            "514-555-5678"
                                        }
                                        li {
                                            b {
                                                "Type de facture:"
                                            }
                                            "Produits"
                                        }
                                        li {
                                            b {
                                                "Réf. ancienne"
                                            }
                                            "F-2025-001"
                                        }
                                    }
                                    table."table table-sm" {
                                        tbody {
                                            tr {
                                                th {
                                                    "Sous-total:"
                                                }
                                                td."text-right" {
                                                    "1500.00"
                                                }
                                            }
                                            tr {
                                                th {
                                                    "TPS 5%:"
                                                }
                                                td."text-right" {
                                                    "75.00"
                                                }
                                            }
                                            tr {
                                                th {
                                                    "TVQ 9.975%:"
                                                }
                                                td."text-right" {
                                                    "149.44"
                                                }
                                            }
                                            tr {
                                                th {
                                                    "Total:"
                                                }
                                                td."text-right" {
                                                    "1724.44"
                                                }
                                            }
                                            tr {
                                                th {
                                                    "Total des paiements enregistrés:"
                                                }
                                                td."text-right" {
                                                    "862.22"
                                                }
                                            }
                                            tr {
                                                th {
                                                    "Total des remboursements enregistrés:"
                                                }
                                                td."text-right" {
                                                    "0.00"
                                                }
                                            }
                                            tr {
                                                th {
                                                    "Solde à payer:"
                                                }
                                                td."text-right" {
                                                    "862.22"
                                                }
                                            }
                                        }
                                    }
                                    br;
                                    ul."list-group list-group-flush" {
                                        li."list-group-item py-1" {
                                            button."btn btn-primary" data-toggle="modal" data-target="#update-facture" {
                                                "Modifier la facture"
                                            }
                                        }
                                        li."list-group-item py-1" {
                                            a."btn btn-primary" href="/factures/rec123/transactions" {
                                                "Voir les transactions"
                                            }
                                        }
                                        li."list-group-item py-1" {
                                            button."generate-print btn btn-success" type="button" data-facture-id="rec123" {
                                                "Visualiser"
                                            }
                                        }
                                        li."list-group-item py-1" {
                                            form action="/factures/rec123/cancel" method="POST" {
                                                button."btn btn-danger" type="submit" {
                                                    "Annuler la facture"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div."card mt-1" {
                            div."card-body" {
                                h3."card-title" {
                                    "Détails de l'événement"
                                }
                                h5."card-subtitle mb-2" {
                                    "Mariage de Marie - 2025-06-15"
                                }
                                p."card-text" {
                                    span {
                                        b {
                                            "Type:"
                                        }
                                        "Mariage"
                                    }
                                }
                                div."row" {
                                    div."col-auto" {
                                        form action="/factures/rec123/unlink-event" method="POST" {
                                            button."btn btn-danger" type="submit" {
                                                "Dissocier de l'événement"
                                            }
                                        }
                                    }
                                    div."col-auto" {
                                        a."btn btn-primary" href="/events/rec123" {
                                            "Voir l'événement"
                                        }
                                    }
                                }
                            }
                        }
                        div."modal fade" id="update-facture" aria-hidden="true" aria-labelledby="update-facture-label" role="dialog" tabindex="-1" {
                            div."modal-dialog" role="document" {
                                form action="/factures/rec123/update" method="POST" {
                                    div."modal-content" {
                                        div."modal-header" {
                                            h5."modal-title" id="make-payment-modal-label" {
                                                "Modification de la facture"
                                            }
                                            button."close" aria-label="Close" data-dismiss="modal" type="button" {
                                                span aria-hidden="true" {
                                                    "&times;"
                                                }
                                            }
                                        }
                                        div."modal-body" {
                                            div."facture-form" {
                                                div."form-row form-group" {
                                                    div."col-12" {
                                                        label for="date" {
                                                            "Date"
                                                        }
                                                        input."form-control date-picker" id="date" value="2025-01-10" type="text" name="date" autocomplete="false";
                                                    }
                                                }
                                                div."form-row form-group" {
                                                    div."col-12" {
                                                        label for="date" {
                                                            "Ref. Ancienne"
                                                        }
                                                        input."form-control" id="paper-ref" type="text" value="F-2025-001" autocomplete="false" name="paper-ref";
                                                    }
                                                }
                                            }
                                        }
                                        div."modal-footer" {
                                            button."btn btn-secondary" type="button" data-dismiss="modal" {
                                                "Annuler"
                                            }
                                            button."btn btn-danger" type="submit" {
                                                "Modifier"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div."modal fade" #alteration-modal tabindex="-1" role="dialog" aria-labelledby="alteration-modal-label" aria-hidden="true" {
                            div."modal-dialog" role="document" {
                                form action="/factures/rec123/linked-facture" method="POST" {
                                    div."modal-content" {
                                        div."modal-header" {
                                            h5."modal-title" id="make-payment-modal-label" {
                                                "Créer une facture d'altération"
                                            }
                                            button."close" data-dismiss="modal" aria-label="Close" type="button" {
                                                span aria-hidden="true" {
                                                    "&times;"
                                                }
                                            }
                                        }
                                        div."modal-body" {
                                            input value="recClient1" type="hidden" name="selected-client";
                                            div."alert alert-danger" role="alert" {
                                                "Attention: le solde de la facture est présentement de 862.22"
                                            }
                                            "La nouvelle facture sera associée à la facture #1001 du client"
                                            b {
                                                "Marie Tremblay"
                                            }
                                            "."
                                        }
                                        div."modal-footer" {
                                            button."btn btn-secondary" type="button" data-dismiss="modal" {
                                                "Annuler"
                                            }
                                            button."btn btn-danger" type="submit" {
                                                "Créer quand même"
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
    }
}

fn list_factures(factures: Vec<FactureDashboardData>) -> Markup {
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

pub fn page_factures(factures: Vec<FactureDashboardData>) -> Markup {
    let body = html! {
        (navbar(MenuConstants::Factures))
        (list_factures(factures))
        (footer())
        (find_factures())
    };
    page("Factures", body)
}

pub fn page_facture_items(page_data: PageFactureItemsData) -> Markup {
    let body = html! {
        (navbar(MenuConstants::Factures))
        (the_items(page_data))
        (footer())
        (find_factures())
    };
    page("Items de la facture", body)
}
