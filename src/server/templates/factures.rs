use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::server::{
    models::{
        FactureDashboardData, FactureInfo, FactureItemEntry, FactureItemFormConfig,
        FactureItemInfo, FactureItemsData, MaybeTransaction, PAYMENT_TYPES,
        PageAddOneFactureItemData, PageAddProduct, PageFactureItemsData, PageOneFactureItemData,
        PagePrintData, PageTransactionsData, REFUND_TYPES, TheTransaction, Transaction,
        clients::{ClientView, ClientViewFuzzySearch},
        config::{ExtraLargeAmounts, NoteTemplate},
        events::EventView,
        facture_items::{FactureComputed, FactureItemType, FactureItemValue, FactureItemView},
        factures::FactureView,
        initial_payment_amount,
        payments::{PaymentView, PreCalculatedPayment},
        product_types::ProductTypeView,
        products::{ProductInfo, ProductView},
        refunds::RefundView,
        statuts::{FLOOR_ITEM_INITIAL_TRANSITIONS, State, StateView},
    },
    templates::{
        clients::{clients_table, find_clients, new_client_form},
        events::{events_table, new_event_form},
        utils::*,
    },
};
struct TransactionPage {
    body: Markup,
    javascript: Markup,
}

struct FuzzySearch {
    body: Markup,
    javascript: Markup,
}

fn fuzzy_search_client(facture_type: &str, clients: Vec<ClientView>) -> FuzzySearch {
    let clients: Vec<ClientViewFuzzySearch> = clients
        .into_iter()
        .map(ClientViewFuzzySearch::from)
        .collect();
    let clients_serialized = serde_json::to_string(&clients).unwrap_or("[]".to_string());

    let clients = format!("var clients = {};", clients_serialized);
    let body = html! {
        div."d-none" id="result-template" {
            form action="/factures/new/select-client" method="POST" {
                input."selected-client" type="hidden" name="selected-client";
                input name="facture-type" type="hidden" value=(facture_type);
                div."card item" style="width: 15rem;" {
                    div."card-body" {
                        h5."card-title" {}
                        h6."card-subtitle text-muted" {}
                        a."card-link stretched-link" href="" {}
                    }
                }
            }
        }
        style {
            (PreEscaped(r#"
                #fuzzy-results {
                    display: flex;
                }
                #fuzzy-results .item {
                    flex-basis: 20%;
                    margin: 5px;
                }
            "#))
        }
        div."row" id="fuzzy-results" {}
    };
    let javascript = html! {
        script async src="https://cdnjs.cloudflare.com/ajax/libs/fuse.js/3.4.5/fuse.min.js" {}

        script type="text/javascript" {
            (PreEscaped(clients))
            (PreEscaped(r##"
                /*
                list is an array of json object
                inputIds is a json object representing selectors eg: `{"firstname": "Prenom", "lastname": "Nom"}`
                */
                function setupFuse(list, inputIds) {
                    var keys = Object.values(inputIds);
                    var options = {
                        shouldSort: true,
                        threshold: 0.3,
                        location: 0,
                        distance: 100,
                        maxPatternLength: 32,
                        minMatchCharLength: 1,
                        keys
                    };
                    var fuse = new Fuse(list, options); // "list" is the item array
                    function renderResults(results) {
                        var template$ = $('#result-template');
                        var node$ = $('#fuzzy-results');

                        node$.empty();

                        results.forEach(function (e) {
                            var cloned$ = template$.clone();
                            cloned$.removeClass("d-none");
                            $('.card-title', cloned$).html([e.Prenom, e.Nom].join(" "));
                            $('.card-subtitle', cloned$).html(e.Ville);
                            $('.selected-client', cloned$).attr('value', e.id);
                            $('.card-link', cloned$).unbind('click').click(function (e) {
                                e.preventDefault();
                                $('form', cloned$).submit();
                            });
                            node$.append(cloned$);
                        });
                    }

                    var selector = Object.keys(inputIds).map((k) => `#${k}`).join(",");

                    $(selector).on("keyup", function () {
                        var searchTerms = $.makeArray($(selector).map(function (i, o) { return $(o).val(); }));
                        var search = searchTerms.join(",");
                        renderResults(fuse.search(search));
                    });
                }
                window.addEventListener('load', function () { //wait for fuse
                  setupFuse(clients, {"firstname": "Prenom", "lastname": "Nom"});
                });
            "##))
        }
    };
    FuzzySearch { body, javascript }
}

fn search_products() -> Markup {
    html! {
        script type="text/javascript" {
            (PreEscaped(r##"
                $(document).ready(function () {
                    function setClass(inputLength) {
                        if (inputLength > 0) {
                            $('#facture-items-actions .filtered-warning').removeClass('d-none');
                            $('#facture-items-actions').addClass('colored-actions');
                        } else {
                            $('#facture-items-actions').removeClass('colored-actions');
                            $('#facture-items-actions .filtered-warning').addClass('d-none');
                        }
                    }
                    function search(value) {
                        $("div.products-cards .one-product-card:not(.d-none)").filter(function () {
                            var title = $('.card-title', $(this))
                                .text()
                                .toLowerCase()
                                .normalize('NFD')
                                .replace(/[\u0300-\u036f]/g, "");
                            var normalizedValue = value
                                .toLowerCase()
                                .normalize('NFD')
                                .replace(/[\u0300-\u036f]/g, "");
                            $(this).toggle(title.indexOf(normalizedValue) > -1);
                        });
                    }
                    $("#search").on("keyup", function () {
                        var value = $(this).val().toLowerCase();
                        search($(this).val().toLowerCase());
                        setClass(value.length);
                    });
                    search($("#search").val().toLowerCase());
                    setClass($("#search").val().toLowerCase().length);

                    function filterProducts() {
                        var showNonAvailable = $('input.product-availability-checkbox').map((i, o) => ($(o).prop("checked"))).toArray().every((v) => v);
                        var currentStatuses = $("input.product-type-checkbox").map((i, o) => ({ [$(o).val()]: $(o).prop("checked") })).toArray();
                        var statusByType = currentStatuses.reduce(function (p, c) {
                            var type = c[0];
                            var checked = c[1];
                            return {
                                ...p,
                                ...c
                            }
                            }, {}
                        );

                        $('div.products-cards .one-product-card').each(function (i, o) {
                            var rawTypes = $(o).attr('data-product-types');
                            var availability = $(o).attr('data-product-availability') === 'true';
                            var pTypes = rawTypes.split(",").map(s => s.trim())
                            var typeShow = pTypes.some(s => statusByType[s] === true);
                            var show = typeShow && (availability || showNonAvailable);
                            if (show) {
                                $(o).removeClass("d-none");
                            } else {
                                $(o).addClass("d-none");
                            }
                        });

                        search($("#search").val().toLowerCase());
                    }
                    
                    $("input.product-type-checkbox, input.product-availability-checkbox").on("change", function () {
                        filterProducts();
                    });

                    filterProducts();
                });
            "##))
        }
    }
}

fn generate_print_js(for_admin: bool) -> Markup {
    html! {
        script type="text/javascript" {
            (PreEscaped(format!(r#"
                $('.generate-print').click(function (e) {{
                    var factureId = $(e.target).data().factureId;
                    var waitingWindow = window.open("/wait", `waiting-${{factureId}}`, 'width=300,height=300');
                    $.post(`/factures/${{factureId}}/generate-print?admin=${}`)
                        .done(function (data, _statusText, xhr) {{
                            waitingWindow.location.href = data.url;
                        }})
                        .fail(function (err) {{
                            console.log(err);
                            window.location.href = `/factures/${{factureId}}/print`;
                        }});
                }});
            "#, for_admin)))
        }
    }
}

fn item_form_scripts() -> Markup {
    html! {
        script type="text/javascript" {
            (PreEscaped(r#"
                $(document).ready(function () {
                    $('form button.set-client-name').unbind('click').click(function() {
                        var name = $(this).attr('data-name');
                        if (name) {
                            $('.form-row.form-group input[name="beneficiary"]').val(name);
                        }
                    });
                    function toggleSeamstressSelector(toggle) {
                        if (toggle) {
                            $('div.seamstress').removeClass('d-none');
                        } else {
                            $('div.seamstress').addClass('d-none');
                        }
                    }

                    $('select.transition-selection').each(function (i, o) {
                        $(o).change(function () {
                            toggleSeamstressSelector($(0).val() === 'RecordingTransfertToSeamstressDate');
                        });
                    });

                    toggleSeamstressSelector($('select.transition-selection').val() === 'RecordingTransfertToSeamstressDate');
                    $('form.itemform #notes-formula').change(function () {
                        var notesInput = $('form.itemform #notes');
                        var previous = notesInput.val();
                        var spacer = previous.trim().length <= 1 ? '' : '\n';
                        notesInput.val(previous + spacer + $(this).val());
                        $(this).val(''); // reset to empty formule type.
                    });

                    $('form.itemform input[type!="hidden"]:first').each(function () {
                        $(this).trigger('focus');
                    });

                    //hide form fields when floor item is checked and show the status dropdown
                    function toggleFloorItemFields(toggle) {
                        $('.form-row.form-group input[name="size"], .form-row.form-group input[name="chest"], .form-row.form-group input[name="waist"], .form-row.form-group input[name="hips"], .form-row.form-group input[name="color"]').each(function () {
                            if (toggle) {
                                $(this).removeAttr('required');
                                $(this).parentsUntil('.form-row.form-group').parent().hide();
                            } else {
                                $(this).attr('required', '');
                                $(this).parentsUntil('.form-row.form-group').parent().show();
                            }
                        });
                        $('.form-row.form-group select[name="status"]').each(function () {
                            if (toggle) {
                                $(this).attr('required', '');
                                $(this).parentsUntil('.form-row.form-group').parent().show();
                            } else {
                                $(this).removeAttr('required');
                                $(this).parentsUntil('.form-row.form-group').parent().hide();
                            }
                        });
                    }
                    // dress with the checkbox
                    $('form input[name="floor-item"][type="checkbox"]').change(function () {
                        toggleFloorItemFields($(this).prop('checked'));
                    });
                    var checked = $('form input[name="floor-item"][type="checkbox"]').prop('checked');
                    // hidden when other type of product
                    var floorItemHidden = $('form input[name="floor-item"][type="hidden"]').val() === 'true';
                    toggleFloorItemFields(floorItemHidden || checked);
                });
            "#))
        }
    }
}

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

fn payment_form_script() -> Markup {
    let setup_transaction_validation = PreEscaped(
        r#"
        function setupTransactionValidation(id) {
            function computeFutureBalance(balance, newValue) {
                if (newValue.trim().length > 0) {
                var newBalance = Math.round((balance - parseFloat(newValue)) * 100) / 100;
                    $(`#${id}-future-balance > span`).html(`Future solde: ${newBalance}`);
                } else {
                    $(`#${id}-future-balance > span`).html(`Future solde: ${balance}`);
                }
            }

            function showExplicitAmount(newValue) {
                if (initalValue !== newValue) {
                    var initalValue = $(`#${id}-payment-amount`).val();
                    $(`#${id}-explicit-amount-alert`).hide();
                } else {
                    $(`#${id}-explicit-amount-alert`).show();
                }
            }

            function toggleSave(balance, newValue) {
                try {
                    var floatValue = parseFloat(newValue, 10);
                $('#payment-form form button[type="submit"]').attr("disabled", balance < floatValue);
                } catch (err) {

                }
            }

            function processNewAmount(balance, value) {
                computeFutureBalance(balance, value);
                showExplicitAmount(value);
                toggleSave(balance, value);
            }

            $(`#${id}-payment-amount`).change(function () {
                processNewAmount($(this).data("balance"), $(this).val());
            });
            $(`#${id}-payment-amount`).on('keyup', function () {
                processNewAmount($(this).data("balance"), $(this).val());
            });
            computeFutureBalance($(`#${id}-payment-amount`).data("balance"), $(`#${id}-payment-amount`).val());
        };
    "#,
    );
    html! {
        script type="text/javascript" {
            (setup_transaction_validation)
        }
    }
}
fn table_transaction_scripts() -> Markup {
    html! {
        script type="text/javascript" {
            (PreEscaped(r##"
                $(document).ready(function(){
                    $("table.payments-refunds").tablesorter({
                        theme : "bootstrap",
                        widthFixed: true,
                        sortList : [[3,0]]
                    });

                    $("button#add-payment").unbind('click').click(function() {
                        $("#payment-form").removeClass("d-none");
                        $("#refund-form").addClass("d-none");
                    });

                    $("button#add-refund").unbind('click').click(function() {
                        $("#payment-form").addClass("d-none");
                        $("#refund-form").removeClass("d-none");
                    });
                });
            "##))
        }
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
    facture_id: u64,
    facture_type: &Option<String>,
    location_item_id: u64,
    alteration_item_id: u64,
) -> Markup {
    let default = String::from("Product");
    let f = facture_type.as_ref().unwrap_or(&default);
    let f = f.as_str();
    let url = match f {
        "Location" => format!("/factures/{}/add-item/{}", facture_id, location_item_id),
        "Alteration" => format!("/factures/{}/add-item/{}", facture_id, alteration_item_id),
        _ => format!("/factures/{}/add-item", facture_id),
    };
    html! {
        a."btn btn-primary" href=(url) {
            "Ajouter un item"
        }
    }
}

fn item_row_action_col(facture_id: u64, facture_item_id: u64) -> Markup {
    let item_url = format!("/factures/{}/items/{}", facture_id, facture_item_id);
    let delete_url = format!("{}/delete", item_url);

    html! {
        a."btn btn-sm btn-primary" href=(item_url) {
            "Voir"
        }
        (" ")
        form."inline-button" method="POST" action=(delete_url) {
            button."btn btn-sm btn-danger" type="submit" {
                "Retirer"
            }
        }
    }
}

fn list_the_items_row(entry: &FactureItemEntry<FactureItemView>) -> Markup {
    match &entry.item.value {
        FactureItemValue::FactureItemProduct(value) => {
            html! {
                tr {
                    td scope="row" {
                        (item_row_action_col(entry.item.facture_id, entry.item.id))
                    }
                    td {
                        (value.quantity)
                    }
                    td {
                        @if let Some(b) = &value.beneficiary {
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
                        (state(entry.state.current_state.value(), Some(entry.state.current_state.label()), None))
                    }
                }
            }
        }
        FactureItemValue::FactureItemLocation(value) => html! {
            tr {
                td scope="row" {
                    (item_row_action_col(entry.item.facture_id, entry.item.id))
                }
                td {
                    (value.quantity)
                }
                td {
                    @if let Some(b) = &value.beneficiary {
                        (b)
                    }
                }
                td {
                    @if let Some(p) = value.price {
                        (p)
                    }
                }
                td {
                    @if let Some(ex) = value.insurance {
                        (ex) "$"
                    }
                }
                td {
                    @if let Some(ex) = value.other_costs {
                        (ex) "$"
                    }
                }
                td {
                    (state(entry.state.current_state.value(), Some(entry.state.current_state.label()), None))
                }
            }
        },
        FactureItemValue::FactureItemAlteration(value) => html! {

            tr {
                td scope="row" {
                    (item_row_action_col(entry.item.facture_id, entry.item.id))
                }
                td {
                    (value.quantity)
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
                    @if let Some(p) = value.rebate_dollar {
                        (p) "$"
                    }
                }
                td {
                    (state(entry.state.current_state.value(), Some(entry.state.current_state.label()), None))
                }
            }
        },
    }
}

fn list_the_items(facture_data: &FactureItemsData) -> Markup {
    let default = String::from("Product");
    let facture_type = facture_data
        .facture_info
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
                    @for entry in &facture_data.items {
                        (list_the_items_row(entry))
                    }
                }
            }

        } @else {
            p { "Il n'y a aucun item dans la facture." }
        }
    }
}

fn facture_type(facture_type: &Option<String>) -> String {
    let default = String::from("Product");
    let f = facture_type.as_ref().unwrap_or(&default);
    let f = f.as_str();
    match f {
        "Location" => "Location".to_string(),
        "Alteration" => "Altération".to_string(),
        _ => "Produits".to_string(),
    }
}

fn facture_info_client(facture: &FactureView, client: &ClientView) -> Markup {
    html! {
        ul."ml-0 list-unstyled" {
            li {
                b {
                    "Client: "
                }
                (client.name())
            }
            li {
                b {
                    "Ville: "
                }
                @if let Some(c) = &client.street {
                    (c)
                }
            }
            li {
                b {
                    "Téléphone: "
                }
                (client.phone1)
            }
            li {
                b {
                    "Téléphone #2: "
                }
                @if let Some(p) = &client.phone2 {
                    (p)
                }
            }
            li {
                b {
                    "Type de facture: "
                }
                (facture_type(&facture.facture_type))
            }
            @if let Some(p) = &facture.paper_ref {
                li {
                    b {
                        "Réf. ancienne: "
                    }
                    (p)
                }
            }
        }
    }
}

fn facture_info_total(facture_computed: &FactureComputed) -> Markup {
    html! {
        table."table table-sm" {
            tbody {
                tr {
                    th {
                        "Sous-total:"
                    }
                    td."text-right" {
                        (facture_computed.total)
                    }
                }
                tr {
                    th {
                        "TPS 5%:"
                    }
                    td."text-right" {
                        (facture_computed.tps)
                    }
                }
                tr {
                    th {
                        "TVQ 9.975%:"
                    }
                    td."text-right" {
                        (facture_computed.tvq)
                    }
                }
                tr {
                    th {
                        "Total:"
                    }
                    td."text-right" {
                        (facture_computed.tax_total)
                    }
                }
                tr {
                    th {
                        "Total des paiements enregistrés:"
                    }
                    td."text-right" {
                        (facture_computed.total_payments)
                    }
                }
                tr {
                    th {
                        "Total des remboursements enregistrés:"
                    }
                    td."text-right" {
                        (facture_computed.total_refunds)
                    }
                }
                tr {
                    th {
                        "Solde à payer:"
                    }
                    td."text-right" {
                        (facture_computed.balance)
                    }
                }
            }
        }
    }
}

fn facture_info_actions(
    facture_id: u64,
    show_items_button: bool,
    show_transactions: bool,
    has_event: bool,
    is_cancelled: bool,
) -> Markup {
    let url = format!("/factures/{}/items", facture_id);
    let transactions_url = format!("/factures/{}/transactions", facture_id);
    let event_url = format!("/factures/{}/select-event", facture_id);
    let cancel_url = format!("/factures/{}/cancel", facture_id);
    let uncancel_url = format!("/factures/{}/uncancel", facture_id);
    html! {
        ul."list-group list-group-flush" {
            @if show_items_button {
                li."list-group-item py-1" {
                    a."btn btn-primary" href=(url) {
                        "Voir les items"
                    }
                }
            }

            li."list-group-item py-1" {
                button."btn btn-primary" data-toggle="modal" data-target="#update-facture" {
                    "Modifier la facture"
                }
            }
            @if show_transactions {
                li."list-group-item py-1" {
                    a."btn btn-primary" href=(transactions_url) {
                        "Voir les transactions"
                    }
                }
            }
            @if !has_event {
                li."list-group-item py-1" {
                    a."btn btn-primary" href=(event_url) {
                        "Associer un événement"
                    }
                }
            }
            li."list-group-item py-1" {
                button."generate-print btn btn-success" type="button" data-facture-id=(facture_id) {
                    "Visualiser"
                }
            }
            @if is_cancelled {
                li."list-group-item py-1" {
                    form action=(uncancel_url) method="POST" {
                        button."btn btn-success" type="submit" {
                            "Restaurer la facture"
                        }
                    }
                }
            } @else {
                li."list-group-item py-1" {
                    form action=(cancel_url) method="POST" {
                        button."btn btn-danger" type="submit" {
                            "Annuler la facture"
                        }
                    }
                }
            }
        }
    }
}

fn event_details(facture_id: u64, event: &EventView) -> Markup {
    let unlink_url = format!("/factures/{}/unlink-event", facture_id);
    let event_url = format!("/events/{}", event.id);
    html! {
        div."card mt-1" {
            div."card-body" {
                h3."card-title" {
                    "Détails de l'événement"
                }
                h5."card-subtitle mb-2" {
                    (event.name) " - " (event.date)
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
                        form action=(unlink_url) method="POST" {
                            button."btn btn-danger" type="submit" {
                                "Dissocier de l'événement"
                            }
                        }
                    }
                    div."col-auto" {
                        a."btn btn-primary" href=(event_url) {
                            "Voir l'événement"
                        }
                    }
                }
            }
        }
    }
}

fn facture_form(facture: &FactureView) -> Markup {
    html! {
        div."facture-form" {
            div."form-row form-group" {
                div."col-12" {
                    label for="date" {
                        "Date"
                    }
                    input."form-control date-picker" id="date" value=[&facture.date] type="text" name="date" autocomplete="false";
                }
            }
            div."form-row form-group" {
                div."col-12" {
                    label for="paper-ref" {
                        "Ref. Ancienne"
                    }
                    input."form-control" id="paper-ref" autocomplete="false" type="text" name="paper-ref" value=[&facture.paper_ref];
                }
            }
        }
    }
}

fn the_items(page_data: &PageFactureItemsData) -> Markup {
    let facture_id = page_data.facture_data.facture_info.facture.id;
    let has_event = page_data.facture_data.facture_info.event.is_some();
    let box_content = html! {
        (facture_info_client(&page_data.facture_data.facture_info.facture, &page_data.facture_data.facture_info.client))
        br;
        (facture_info_total(&page_data.facture_data.facture_info.facture_computed))
        (facture_info_actions(page_data.facture_data.facture_info.facture.id, false, true, has_event, page_data.facture_data.facture_info.facture.cancelled))
    };
    let facture_title = format!("Facture #{}", facture_id);

    html! {
        main role="main" {
            div."container-fluid" {
                div."row" {
                    div."col-12 order-12 col-lg-8 order-lg-1" {
                        div."row actions sticky-top" {
                            div."col-12" {
                                (the_items_action_col(facture_id, &page_data.facture_data.facture_info.facture.facture_type, page_data.location_product.id, page_data.alteration_product.id))
                            }
                        }

                        (list_the_items(&page_data.facture_data))
                    }
                    div."col-12 order-1 col-lg-4 order-lg-12" {
                        (sidebar_info_box("Détails de la facture", Some(&facture_title), box_content))
                        @if let Some(event) = &page_data.facture_data.facture_info.event {
                            (event_details(facture_id, event))
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
                                                    (PreEscaped("&times;"))
                                                }
                                            }
                                        }
                                        div."modal-body" {
                                            (facture_form(&page_data.facture_data.facture_info.facture))
                                        }
                                        div."modal-footer" {
                                            button."btn btn-secondary" type="button" data-dismiss="modal" {
                                                "Annuler"
                                            }
                                            button."btn btn-success" type="submit" {
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
                                                    (PreEscaped("&times;"))
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
                                                    (state(s.current_state.value(), Some(s.current_state.label()), None))
                                                }
                                                td {
                                                    @if let Some(date) = s.current_state.date() {
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
                                                (state(s.current_state.value(), None, Some("ml-1")))
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
                            a."btn btn-primary" href="/factures/new?facture-type=Alteration" {
                                "Altération"
                            }
                            a."btn btn-primary" href="/factures/new?facture-type=Product" {
                                "Produits"
                            }
                            a."btn btn-primary" href="/factures/new?facture-type=Location" {
                                "Location"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn facture_item_form(
    facture_id: u64,
    url: &str,
    client: &ClientView,
    product: &ProductView,
    product_type: &ProductTypeView,
    value: &FactureItemType,
    form_config: &FactureItemFormConfig,
    is_update: bool,
) -> Markup {
    fn quantity(q: i64) -> Markup {
        html! {
            div."form-row form-group" {
                div."col-12" {
                    label for="quantity" {
                        "Quantité"
                    }
                    input."form-control" id="quantity" type="number" min="0" name="quantity" value=(q);
                }
            }
        }
    }

    fn price(p: &Option<i64>) -> Markup {
        html! {
            div."form-row form-group" {
                div."col-12" {
                    (price_input("price", "Prix unitaire", p, true))
                }
            }
        }
    }

    fn notes(notes: &Option<String>, note_templates: &Vec<NoteTemplate>) -> Markup {
        html! {
            div."form-row form-group" {
                div."col-12" {
                    label."w-100" for="notes" {
                        span {
                            "Notes"
                        }
                        @if note_templates.iter().count() > 0 {
                            div."pull-right" {
                                select #notes-formula {
                                    option value="" {
                                        "Formule type"
                                    }
                                    @for NoteTemplate { note_type: _, key, value } in note_templates {
                                        option value=(value) {
                                            (key)
                                        }
                                    }
                                }
                            }
                        }
                    }
                    textarea."form-control" id="notes" name="notes" rows="5" {
                        @if let Some(n) = notes {
                            (n)
                        }
                    }
                }
            }
        }
    }

    fn extra_large_amount(p: &ProductTypeView, ex: &ExtraLargeAmounts) -> Option<i64> {
        if p.is_dress() {
            Some(ex.wedding)
        } else {
            Some(ex.others)
        }
    }

    fn set_client_btn(client_name: &str) -> Markup {
        html! {
            button."btn btn-sm btn-primary set-client-name" type="button" data-name=(client_name) {
                "Utiliser " (client_name)
            }
        }
    }

    let client_name = client.name();
    match &value {
        FactureItemValue::FactureItemProduct(value) => html! {
            form."itemform" action=(url) method="POST" {
                input name="product-id" value=(&product.id) type="hidden";
                input type="hidden" name="facture-id" value=(facture_id);

                (quantity(value.quantity))

                (price(&value.price))

                div."form-row form-group" {
                    div."col-12" {
                        label for="rebatePercent" {
                            "Rabais (%)"
                        }
                        input."form-control" id="rebate-percent" name="rebate-percent" type="number" min="0" value=[value.rebate_percent] step="0.01" max="100";
                    }
                }

                div."form-row form-group" {
                    div."col-12" {
                        label for="beneficiary" {
                           "Bénéficiaire " (set_client_btn(&client_name))
                        }
                        input."form-control" id="beneficiary" type="text" value=[&value.beneficiary] name="beneficiary";
                    }
                }

                (notes(&value.notes, &form_config.note_templates))

                @if product_type.is_dress() {
                    @if let Some(extra_amount_config) = extra_large_amount(product_type, &form_config.extra_large_amount) {
                        @let extra_amount = value.extra_large_size.unwrap_or(extra_amount_config);
                        div."form-row form-group" {
                            div."col-12" {
                                div."form-check" {
                                    input type="hidden" value="false" name="extra-large-size";
                                    input #extra-large-size-override name="extra-large-size" type="hidden" value="false";
                                    input id="dress-extra-large-size" disabled class="form-check-input" name="extra-large-size" value="true" type="checkbox";
                                    input type="hidden" name="extra-large-size-amount" value=(extra_amount);
                                    label for="dress-extra-large-size" {
                                        "Taille forte (" (extra_amount) ")"
                                    }
                                }
                            }
                        }
                    }
                    div."form-row form-group" {
                        div."col-12" {
                            div."form-check" {
                                @let checked = if value.floor_item { Some(true) } else { None };
                                input value="false" type="hidden" name="floor-item";
                                input."form-check-input" id="dress-floor-item" type="checkbox" value="true" name="floor-item" checked=[checked];
                                label for="dress-floor-item" {
                                    "Modèle planché"
                                }
                            }
                        }
                    }
                    div."form-row form-group" {
                        div."col-12" {
                            label for="size" {
                                "Grandeur"
                            }
                            input."form-control" id="size" value=[&value.size] name="size" type="text";
                        }
                    }
                    div."form-row form-group" {
                        div."col-12" {
                            label for="chest" {
                                "Buste"
                            }
                            input."form-control" id="chest" value=[&value.chest] min="0" type="number" name="chest";
                        }
                    }
                    div."form-row form-group" {
                        div."col-12" {
                            label for="waist" {
                                "Taille"
                            }
                            input."form-control" id="waist" type="number" name="waist" value=[&value.waist] min="0";
                        }
                    }
                    div."form-row form-group" {
                        div."col-12" {
                            label for="hips" {
                                "Hanche"
                            }
                            input."form-control" id="hips" name="hips" type="number" value=[&value.hips] min="0";
                        }
                    }
                    div."form-row form-group" {
                        div."col-12" {
                            label for="color" {
                                "Couleur"
                            }
                            input."form-control" id="color" value=[&value.color] type="text" name="color";
                        }
                    }
                } @else if product_type.is_gaine() {
                    input name="extra-large-size" value="false" type="hidden";
                    input name="floor-item" value="true" type="hidden";

                    @let gaine_sizes = vec!["S", "M", "L", "XL", "2XL", "3XL", "4XL"];
                    @let gaine_sizes: Vec<(bool, &str)> = gaine_sizes
                        .into_iter()
                        .enumerate()
                        .map(|(i, e)| (value.size.as_ref().map_or(i == 0, |e2| e2 == &e), e))
                        .collect();

                    div."form-row form-group" {
                        div."col-12" {
                            label for="size" {
                                "Grandeur"
                            }
                            select id="size" name="size" {
                                @for (selected, gaine_size) in gaine_sizes {
                                    @if selected {
                                        option value=(gaine_size) selected {
                                            (gaine_size)
                                        }
                                    } @else {
                                        option value=(gaine_size) {
                                            (gaine_size)
                                        }
                                    }
                                }
                            }
                        }
                    }
                } @else {
                    input name="extra-large-size" value="false" type="hidden";
                    input name="floor-item" value="true" type="hidden";
                }

                @if !is_update {
                    // this section is hidden if floor-item is not checked
                    div."form-row form-group" {
                        div."col-12" {
                            label for="status" {
                                "Status"
                            }
                            div."pull-right"{
                                select id="status" name="status" {
                                    option value=("") {
                                        "Veuillez choisir une option"
                                    }

                                    @for s in FLOOR_ITEM_INITIAL_TRANSITIONS {
                                        option value=(s) {
                                            (ask_transition(s))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                br;
                button."btn btn-primary btn-lg btn-block" type="submit" {
                    "Sauvegarder"
                }
            }
        },
        FactureItemValue::FactureItemLocation(value) => html! {
            form."itemform" action=(url) method="POST" {
                input name="product-id" value=(product.id) type="hidden";
                input name="facture-id" value=(facture_id) type="hidden";

                (quantity(value.quantity))

                (price(&value.price))

                div."form-row form-group" {
                    div."col-12" {
                        @let default_insurance = Some(2500);
                        (price_input("insurance", "Assurances", &value.insurance.or(default_insurance), false))
                    }
                }

                div."form-row form-group" {
                    div."col-12" {
                        (price_input("other-costs", "Autres frais", &value.insurance, false))
                    }
                }

                div."form-row form-group" {
                    div."col-12" {
                        (set_client_btn(&client_name))
                        input."form-control" id="beneficiary" type="text" value=[&value.beneficiary] name="beneficiary";
                    }
                }

                (notes(&value.notes, &form_config.note_templates))

                br;
                button."btn btn-primary btn-lg btn-block" type="submit" {
                    "Sauvegarder"
                }
            }
        },
        FactureItemValue::FactureItemAlteration(value) => html! {
            form."itemform" action=(url) method="POST" {
                input name="product-id" value=(product.id) type="hidden";
                input name="facture-id" value=(facture_id) type="hidden";

                (quantity(value.quantity))

                (price(&value.price))

                div."form-row form-group" {
                    div."col-12" {
                        @let default_rebase = Some(0);
                        (price_input("rebate-dollar", "Rabais ($)", &value.rebate_dollar.or(default_rebase), false))
                    }
                }

                (notes(&value.notes, &form_config.note_templates))

                br;
                button."btn btn-primary btn-lg btn-block" type="submit" {
                    "Sauvegarder"
                }
            }
        },
    }
}

fn status_history_table(state: &StateView) -> Markup {
    let count = state.previous_states.iter().count();
    html! {
        table."table table-hover table-borderless" {
            @for (idx, st) in state.previous_states.iter().enumerate() {
                tr {
                    td {
                        (idx + 1) ". " (st.label_with_date())
                    }
                }
            }
            tr {
                td {
                    (count + 1) ". " (state.current_state.label_with_date())
                }
            }
        }
    }
}

fn state_modal(
    facture_id: u64,
    facture_item_id: u64,
    state: &StateView,
    seamstresses: &Vec<String>,
) -> Markup {
    let update_url = format!(
        "/factures/{}/items/{}/update-state",
        facture_id, facture_item_id
    );
    let target_id = format!("state-modal-{}", facture_item_id);
    let target_label = format!("label-{}", target_id);
    let type_id = format!("type-{}", facture_item_id);
    let date_id = format!("date-{}", facture_item_id);
    let seamstress_id = format!("seamstress-{}", facture_item_id);
    html! {
        div."modal fade" id=(target_id) role="dialog" aria-labelledby=(target_label) aria-hidden="true" tabindex="-1" {
            div."modal-dialog" role="document" {
                form action=(update_url) method="POST" {
                    div."modal-content" {
                        div."modal-header" {
                            h5."modal-title" id=(target_label) {
                                "Statut de l'item"
                            }
                            button."close" type="button" data-dismiss="modal" aria-label="Close" {
                                span aria-hidden="true" {
                                    (PreEscaped("&times;"))
                                }
                            }
                        }
                        div."modal-body" {
                            div."form-row form-group" {
                                div."col-12" {
                                    label for=(type_id) {
                                        "Transition"
                                    }
                                    select."transition-selection custom-select" id=(type_id) name="type" {
                                        @for t in state.available_transitions().unwrap_or(vec![]) {
                                            option value=(t) selected {
                                                (State::<String, String>::ask(t))
                                            }
                                        }
                                    }
                                }
                            }
                            div."form-row form-group" {
                                div."col-12" {
                                    label for=(date_id) {
                                        "Date"
                                    }
                                    input."form-control date-picker" id=(date_id) type="text" name="date" value="2026-03-23" required autocomplete="false";
                                }
                            }
                            div."form-row form-group d-none seamstress" {
                                div."col-12" {
                                    label for=(seamstress_id) {
                                        "Couturière"
                                    }
                                    select."custom-select" id=(seamstress_id) name="seamstress" {
                                        @for ss in seamstresses {
                                            option value=(ss) selected {
                                                (ss)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div."modal-footer" {
                            button."btn btn-secondary" type="button" data-dismiss="modal" {
                                "Annuler"
                            }
                            button."btn btn-primary" type="submit" {
                                "Sauvegarder"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn the_item(page_data: PageOneFactureItemData) -> Markup {
    let facture_id = page_data.facture.id;
    let facture_item_id = page_data.item.item.id;
    let is_update = true;
    let url = format!(
        "/factures/{}/items/{}/update",
        page_data.facture.id, facture_item_id
    );
    let items_url = format!("/factures/{}/items", facture_id);
    let statut_change_target_modal = format!("#state-modal-{}", facture_item_id);
    let available_transitions = page_data.item.state.available_transitions().iter().count();
    let statut_change_disabled = if available_transitions <= 0 {
        Some(true)
    } else {
        None
    };

    let statuts_history = html! {
        (status_history_table(&page_data.item.state))

        hr;
        button."btn btn-sm btn-warning" data-toggle="modal" disabled=[statut_change_disabled] data-target=(statut_change_target_modal) {
            "Changer statut"
        }
    };
    html! {
        main role="main" {
            div."container-fluid" {
                div."row" {
                    div."order-12 col-lg-8 order-lg-1" {
                        a href=(items_url) {
                            "Retour aux items de la facture"
                        }
                        br;
                        h3 {
                            (page_data.item.product.name)
                        }
                        (facture_item_form(facture_id, &url, &page_data.client, &page_data.item.product, &page_data.product_type, &page_data.item.item.value, &page_data.form_config, is_update))
                    }

                    div."order-1 col-lg-4 order-lg-12" {
                        (sidebar_info_box("Historique des statuts de l'item", None, statuts_history))
                    }

                    (state_modal(facture_id, facture_item_id, &page_data.item.state, &page_data.form_config.seamstresses))
                }
            }
        }
    }
}

fn make_client_table_action_col(facture_type: &str) -> impl Fn(&ClientView) -> Markup {
    move |client| {
        html! {
            form action="/factures/new/select-client" method="POST" {
                input name="selected-client" value=(client.id) type="hidden";
                input name="facture-type" value=(facture_type) type="hidden";
                button."btn btn-sm btn-primary" type="submit" {
                    "Choisir"
                }
            }
        }
    }
}

fn new_facture_the_client(facture_type: &str, clients: Vec<ClientView>) -> Markup {
    let action_col = make_client_table_action_col(facture_type);
    let url = format!("/factures/new/new-client?facture-type={}", facture_type);
    html! {
        main role="main" {
            div."container-fluid" {
                div."row actions sticky-top" id="client-actions" {
                    div."col-12 col-sm-4" {
                        div."row" {
                            div."col-auto" {
                                h4 {
                                    "Choisisser un client existant"
                                }
                            }
                            div."col-auto" {
                                a."btn btn-primary btn-sm" href=(url) {
                                    "Nouveau client"
                                }
                            }
                        }
                    }
                    div."col-12 col-sm-8" {
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
                        (clients_table(clients, action_col))
                    }
                }
            }
        }
    }
}

fn new_facture_new_client(client_form: Markup, fuzzy_search: Markup) -> Markup {
    html! {
        main role="main" {
            div."container-fluid mt-1" {
                div."row" {
                    div."col-12 col-md-6" {
                        (client_form)
                    }
                    div."col-12 col-md-6" {
                        (fuzzy_search)
                    }
                }
            }
        }
    }
}

fn make_event_table_action_col(url: &str) -> impl Fn(&EventView) -> Markup {
    move |event| {
        html! {
            form action=(url) method="POST" {
                input name="selected-event" value=(event.id) type="hidden";
                button."btn btn-sm btn-primary" type="submit" {
                    "Choisir"
                }
            }
        }
    }
}

fn new_facture_the_event(facture_id: u64, no_event_url: &str, events: Vec<EventView>) -> Markup {
    let new_event_url = format!("/factures/{}/select-event", facture_id);
    let action_col = make_event_table_action_col(&new_event_url);
    let url = format!("/factures/{}/new-event", facture_id);
    html! {
        main role="main" {
            div."container-fluid" {
                div."row actions sticky-top" id="events-actions" {
                    div."col-12 col-sm-5" {
                        div."row" {
                            div."col-auto" {
                                h4 {
                                    "Choisisser un événement existant"
                                }
                            }
                            div."col-auto" {
                                a."btn btn-primary btn-sm" href=(url) {
                                    "Nouvel événement"
                                }
                            }
                            div."col-auto" {
                                a."btn btn-warning btn-sm" href=(no_event_url) {
                                    "Aucun événement"
                                }
                            }
                        }
                    }
                    div."col-12 col-sm-8" {
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

fn new_facture_new_event(event_form: Markup) -> Markup {
    html! {
        main role="main" {
            div."container-fluid mt-1" {
                div."row" {
                    div."col-12" {
                        (event_form)
                    }
                }
            }
        }
    }
}

fn select_item(facture_id: u64, products: Vec<ProductInfo>) -> Markup {
    let add_product_url = format!("/factures/{}/add-product", facture_id);
    let add_item_form_url = format!("/factures/{}/add-item", facture_id);
    html! {
        main role="main" {
            div."container-fluid" {
                div."row actions sticky-top" id="facture-items-actions" {
                    div."col-2" {
                        a."btn btn-primary" href=(add_product_url) {
                            "Ajouter un produit"
                        }
                    }
                    div."col-10" {
                        input."form-control" id="search" type="text" placeholder="Filtre";
                    }
                    div."filtered-warning col-12 d-none" {
                        span {
                            b {
                                "Affichage filtré"
                            }
                        }
                    }
                    div."offset-2 col-10" {
                        div."form-group row" {
                            div."ml-2 form-check" {
                                label."form-check-label" {
                                    input."product-type-checkbox form-check-input" type="checkbox" checked="true" value="wedding";
                                    "Robe de mariée"
                                }
                            }
                            div."ml-2 form-check" {
                                label."form-check-label" {
                                    input."product-type-checkbox form-check-input" checked="true" type="checkbox" value="mom";
                                    "Robe de mère de la mariée"
                                }
                            }
                            div."ml-2 form-check" {
                                label."form-check-label" {
                                    input."product-type-checkbox form-check-input" type="checkbox" value="bal" checked="true";
                                    "Robe de bal"
                                }
                            }
                            div."ml-2 form-check" {
                                label."form-check-label" {
                                    input."product-type-checkbox form-check-input" type="checkbox" value="bouq" checked="true";
                                    "Robe de bouquetière"
                                }
                            }
                            div."ml-2 form-check" {
                                label."form-check-label" {
                                    input."product-type-checkbox form-check-input" value="other" checked="true" type="checkbox";
                                    "Autres"
                                }
                            }
                            div."ml-2 form-check" {
                                label."form-check-label" {
                                    input."product-type-checkbox form-check-input" checked="true" type="checkbox" value="non-available";
                                    "Affiché les articles non disponible en boutique"
                                }
                            }
                        }
                    }
                }
                br;
                div."row" {
                    div."col-12" {
                        form id="add-item" action=(add_item_form_url) method="POST" {
                            div."row products-cards" {
                                @for p in products {
                                    @let add_one_item_url = format!("/factures/{}/add-item/{}", facture_id, p.product.id);

                                    @let normalized_types: Vec<String> = (&p.types).iter().map(|pt|pt.normalized()).collect();
                                    @let normalized_types: Vec<&str> = normalized_types.iter().map(|pt|pt.as_str()).collect();
                                    @let normalized_types = normalized_types.join(",");

                                    @let types: Vec<&str> = (&p.types).into_iter().map(|pt|pt.name.as_str()).collect();
                                    @let types = types.join(", ");

                                    div."one-product-card col-6 col-sm-3 col-md-2" data-product-availability="true" data-product-types=(normalized_types) {
                                        div."card item mb-2" style="width: 15rem;" {
                                            div."card-body" {
                                                h5."card-title" {
                                                    (p.product.name) @if let Some(price) = p.product.price { " - " (price) "$" }
                                                }
                                                h6."card-subtitle text-muted" {
                                                    (types) @if let Some(price) = p.reduced_price() { " - Prix réduit "(price) "$" }
                                                }
                                                a."card-link stretched-link" href=(add_one_item_url) {}
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

fn add_item(page_data: PageAddOneFactureItemData) -> Markup {
    let facture_id = page_data.facture_info.facture.id;
    let items_url = format!("/factures/{}/items", facture_id);
    let is_update = false;
    let has_event = page_data.facture_info.event.is_some();

    let box_content = html! {
        (facture_info_client(&page_data.facture_info.facture, &page_data.facture_info.client))
        br;
        (facture_info_total(&page_data.facture_info.facture_computed))
        (facture_info_actions(facture_id, false, true, has_event, page_data.facture_info.facture.cancelled))
    };
    let facture_title = format!("Facture #{}", facture_id);

    html! {
        main role="main" {
            div."container-fluid" {
                div."row" {
                    div."order-12 col-lg-8 order-lg-1" {
                        a href=(items_url) {
                            "Retour aux items de la facture"
                        }
                        br;
                        h3 {
                            (page_data.item.product.name)
                        }
                        (facture_item_form(facture_id, &items_url, &page_data.facture_info.client, &page_data.item.product, &page_data.product_type, &page_data.item.item,&page_data.form_config , is_update))
                    }

                    div."order-1 col-lg-4 order-lg-12" {
                        (sidebar_info_box("Détails de la facture", Some(&facture_title), box_content))
                    }
                }
            }
        }
    }
}

fn transaction_form_fields(
    transaction: &MaybeTransaction,
    explicit_amount: Option<&PreCalculatedPayment>,
    balance: i64,
) -> Markup {
    let id = match transaction {
        TheTransaction::Payment(x) => x
            .as_ref()
            .map(|a| a.id.to_string())
            .unwrap_or("new".to_string()),
        TheTransaction::Refund(x) => x
            .as_ref()
            .map(|a| a.id.to_string())
            .unwrap_or("new".to_string()),
    };
    let t_type = match transaction {
        TheTransaction::Payment(_) => "payment",
        TheTransaction::Refund(_) => "refund",
    };

    let id_amount = format!("{}-{}-amount", id, t_type);
    let id_type = format!("{}-{}-type", id, t_type);
    let id_date = format!("{}-{}-date", id, t_type);
    let id_cheque = format!("{}-{}-cheque-no", id, t_type);

    let t_types = match transaction {
        TheTransaction::Payment(_) => Vec::from(PAYMENT_TYPES),
        TheTransaction::Refund(_) => Vec::from(REFUND_TYPES),
    };

    let explicit_amount_value = explicit_amount.and_then(|a| a.calculate());
    let explicit_amount_label = explicit_amount_value
        .zip(explicit_amount)
        .map(|(a, pa)| (pa.label(a), format!("{}-explicit-amount-alert", id)));

    html! {
        div {
            div."form-row form-group" {
                div."col-12" {
                    @if let Some((ea_label, ea_id)) = explicit_amount_label {
                        div."alert alert-info" id=(ea_id) {
                            span { (ea_label) }
                        }
                    }
                    label for=(id_amount) {
                        "Montant"
                    }
                    input."form-control" id=(id_amount) value=[transaction.amount()] type="text" name="amount" required autofocus data-balance=(balance);

                    @if transaction.is_payment() && transaction.is_none() {
                        div."alert alert-info" id=(format!("{}-future-balance", id))  {
                            span { "Solde à payer: " (balance) }
                        }
                    }
                }
            }

            div."form-row form-group" {
                div."col-12" {
                    label for=(id_type) {
                        "Type"
                    }
                    select."custom-select" id=(id_type) name="type" required {
                        option value="" {
                            "Veuillez choisir"
                        }
                        @for t in t_types {
                            @let selected = if transaction.t_type() == Some(t) { Some(true) } else { None };
                            option value=(t) selected=[selected]  {
                                (t)
                            }
                        }
                    }
                }
            }
            div."form-row form-group" {
                div."col-12" {
                    label for=(id_date) {
                        "Date"
                    }
                    input."form-control date-picker" id=(id_date) name="date" autocomplete="false" required value=[transaction.date()] type="text";
                }
            }
            @if transaction.is_refund() {
                div."form-row form-group" {
                    div."col-12" {
                        label for=(id_cheque) {
                            "No. de chèque"
                        }
                        input."form-control" id=(id_cheque) type="text" name="cheque-no" value=[transaction.cheque_number()];
                    }
                }
            }
        }
    }
}

fn transaction_form_modal(
    url: &str,
    transaction: &Transaction,
    facture_info: &FactureInfo,
) -> Markup {
    let pre_calculated_payment = match transaction {
        TheTransaction::Payment(_) => Some(initial_payment_amount(facture_info)),
        TheTransaction::Refund(_) => None,
    };
    let maybe_transaction: MaybeTransaction = match transaction {
        TheTransaction::Payment(p) => TheTransaction::Payment(&Some(p)),
        TheTransaction::Refund(p) => TheTransaction::Refund(&Some(p)),
    };
    html! {
        form action=(url) method="POST" {
            div."modal-header" {
                h5."modal-title" id="transaction-modal-label" {
                    "Modifier une transaction"
                }
                button."close" data-dismiss="modal" type="button" aria-label="Close" {
                    span aria-hidden="true" {
                        (PreEscaped("&times;"))
                    }
                }
            }
            div."modal-body" {
                (transaction_form_fields(&maybe_transaction, pre_calculated_payment.as_ref(), facture_info.facture_computed.balance))
            }
            div."modal-footer" {
                button."btn btn-secondary" type="button" data-dismiss="modal" {
                    "Annuler"
                }
                button."btn btn-primary" type="submit" {
                    "Sauvegarder"
                }
            }
        }
    }
}

fn table_transaction(
    facture_info: &FactureInfo,
    payments: &Vec<PaymentView>,
    refunds: &Vec<RefundView>,
) -> (Markup, Vec<Markup>) {
    let mut scripts = vec![];
    fn action_col(
        scripts: &mut Vec<Markup>,
        transaction: Transaction,
        facture_info: &FactureInfo,
    ) -> Markup {
        if transaction.is_payment() {
            scripts.push(jquery_ready(PreEscaped(format!(
                "setupTransactionValidation(\"{}\")",
                transaction.id()
            ))));
        }
        let base_url = match transaction {
            Transaction::Payment(payment_view) => format!(
                "/factures/{}/payments/{}",
                payment_view.facture_id, payment_view.id
            ),
            Transaction::Refund(refund_view) => format!(
                "/factures/{}/refunds/{}",
                refund_view.facture_id, refund_view.id
            ),
        };

        let delete_url = format!("{}/delete", base_url);
        let update_url = format!("{}/update", base_url);
        html! {
            button."btn btn-primary btn-sm" data-toggle="modal" data-target="#rec123-transaction-modal" {
                "Modifier"
            }
            (" ")
            form."inline-button" method="POST" action=(delete_url) {
                button."btn btn-sm btn-danger" type="submit" {
                    "Retirer"
                }
            }
            div."modal fade" id="rec123-transaction-modal" tabindex="-1" role="dialog" aria-hidden="true" aria-labelledby="transaction-modal-label" {
                div."modal-dialog" role="document" {
                    div."modal-content" {
                        (transaction_form_modal(&update_url, &transaction, facture_info))
                    }
                }
            }
        }
    }
    let body = html! {
        @if payments.iter().count() > 0 {
            table."table table-sm payments-refunds" {
                thead {
                    tr {
                        th scope="col" {
                            "Actions"
                        }
                        th scope="col" {
                            "Type"
                        }
                        th scope="col" {
                            "Montant"
                        }
                        th scope="col" {
                            "Date"
                        }
                        th scope="col" {
                            "Type paiement"
                        }
                    }
                }
                tbody {
                    @for p in payments {
                        tr {
                            td {
                                (action_col(&mut scripts, Transaction::Payment(p), facture_info))
                            }
                            td {
                                "Paiement"
                            }
                            td {
                                (p.amount)
                            }
                            td {
                                (p.date)
                            }
                            td {
                                (p.payment_type)
                            }
                        }

                    }
                    @for r in refunds {
                        tr {
                            td {
                                (action_col(&mut scripts, Transaction::Refund(r), facture_info))
                            }
                            td {
                                "Remboursement"
                            }
                            td {
                                (r.amount)
                            }
                            td {
                                (r.date)
                            }
                            td {
                                (r.refund_type)
                            }
                        }

                    }

                }
            }
        } @else {
            p { "Il n'y a aucune transaction reliée à la facture." }
        }
    };
    (body, scripts)
}

fn list_transactions(page_data: PageTransactionsData) -> TransactionPage {
    let payment_url = format!("/factures/{}/payments", page_data.facture_info.facture.id);

    let refunds_url = format!("/factures/{}/refunds", page_data.facture_info.facture.id);
    let pre_calculated_payment = initial_payment_amount(&page_data.facture_info);

    let facture_id = page_data.facture_info.facture.id;
    let has_event = page_data.facture_info.event.is_some();

    let box_content = html! {
        (facture_info_client(&page_data.facture_info.facture, &page_data.facture_info.client))
        br;
        (facture_info_total(&page_data.facture_info.facture_computed))
        (facture_info_actions(facture_id, false, true, has_event, page_data.facture_info.facture.cancelled))
    };
    let main_title = format!("Transactions pour la facture #{}", facture_id);
    let facture_title = format!("Facture #{}", facture_id);

    let (transaction_list_html, mut scripts) = table_transaction(
        &page_data.facture_info,
        &page_data.payments,
        &page_data.refunds,
    );

    scripts.insert(0, payment_form_script());
    scripts.push(jquery_ready(PreEscaped(format!(
        "setupTransactionValidation(\"{}\")",
        "new"
    ))));
    scripts.push(table_transaction_scripts());

    let body = html! {
        main role="main" {
            div."container-fluid" {
                div."row" {
                    div."col-12 order-12 col-lg-8 order-lg-1" {
                        div."row actions sticky-top" {
                            div."col-auto" {
                                h4 {
                                    (main_title)
                                }
                            }
                            div."col-auto" {
                                button."btn btn-primary btn-sm" id="add-payment" {
                                    "Ajouter un paiement"
                                }
                                (" ")
                                button."btn btn-primary btn-sm" id="add-refund" {
                                    "Ajouter un remboursement"
                                }
                            }
                        }
                        div."row" {
                            div."col-12" {
                                (transaction_list_html)
                            }
                        }
                        div."row" {
                            div."col-12" {
                                form method="POST" action=(payment_url) {
                                    div."d-none" id="payment-form" {
                                        (transaction_form_fields(&TheTransaction::Payment(&None), Some(&pre_calculated_payment), page_data.facture_info.facture_computed.balance))
                                        button."btn btn-primary" type="submit" {
                                            "Sauvegarder"
                                        }
                                    }
                                }

                                form method="POST" action=(refunds_url) {
                                    div."d-none" id="refund-form" {
                                        (transaction_form_fields(&TheTransaction::Refund(&None), None, page_data.facture_info.facture_computed.balance))
                                        button."btn btn-primary" type="submit" {
                                            "Sauvegarder"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div."col-12 order-1 col-lg-4 order-lg-12" {
                        (sidebar_info_box("Détails de la facture", Some(&facture_title), box_content))
                    }
                }
            }
        }
    };
    let javascript = html! {
        @for s in scripts {
            (s)
        }
    };
    TransactionPage { body, javascript }
}

fn add_product(facture_id: u64, product_types: &Vec<ProductTypeView>) -> Markup {
    let url = format!("/factures/{}/add-product", facture_id);
    html! {
        main role="main" {
            div."container-fluid" {
                div."row" {
                    div."col-12" {
                        form action=(url) method="POST" {
                            div."form-row form-group" {
                                div."col-12" {
                                    label for="type" {
                                        "Type"
                                    }
                                    select."custom-select" id="type" name="type" {
                                        @for pt in product_types {
                                            option value=(pt.name) {
                                                (pt.name)
                                            }
                                        }
                                    }
                                }
                            }
                            div."form-row form-group" {
                                div."col-12" {
                                    label for="name" {
                                        "Nom du produit"
                                    }
                                    input."form-control" id="name" type="text" required name="name";
                                }
                            }
                            div."form-row form-group" {
                                div."col-12" {
                                    (price_input("price", "Prix unitaire", &None, true))
                                }
                            }
                            div."form-row form-group" {
                                div."col-2" {
                                    button."btn btn-primary" type="submit" {
                                        "Ajouter"
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

// pages

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
        (the_items(&page_data))
        (footer())
        (generate_print_js(false))
    };
    page("Items de la facture", body)
}

pub fn page_one_facture_item(page_data: PageOneFactureItemData) -> Markup {
    let body = html! {
        (navbar(MenuConstants::Factures))
        (the_item(page_data))
        (footer())
        (item_form_scripts())
    };
    page("Item de facture", body)
}

pub fn page_new_facture_the_client(facture_type: Option<&str>, clients: Vec<ClientView>) -> Markup {
    let facture_type = facture_type.unwrap_or("Product");
    let body = html! {
        (navbar(MenuConstants::Factures))
        (new_facture_the_client(facture_type, clients))
        (footer())
        (find_clients("clients-actions", "search", "table.find-client", None))
    };
    page("Sélectionner un client", body)
}

pub fn page_new_facture_new_client(facture_type: Option<&str>, clients: Vec<ClientView>) -> Markup {
    let facture_type = facture_type.unwrap_or("Product");
    let url = format!("/factures/new/new-client?facture-type={}", facture_type);
    let client_form = new_client_form(url.as_ref(), None);
    let fuzzy = fuzzy_search_client(facture_type, clients);

    let body = html! {
        (navbar(MenuConstants::Factures))
        (new_facture_new_client(client_form.body, fuzzy.body))
        (footer())
        (client_form.javascript)
        (fuzzy.javascript)
    };
    page("Nouveau client", body)
}

pub fn page_new_facture_the_event(
    facture_id: u64,
    no_event_url: &str,
    events: Vec<EventView>,
) -> Markup {
    let body = html! {
        (navbar(MenuConstants::Factures))
        (new_facture_the_event(facture_id, no_event_url, events))
        (footer())
        (find_clients("events-actions", "search", "table.find-event", None))
    };
    page("Sélectionner un événement", body)
}

pub fn page_new_facture_new_event(facture_id: u64, event_types: Vec<String>) -> Markup {
    let url = format!("/factures/{}/new-event", facture_id);
    let event_form = new_event_form(url.as_ref(), None, &event_types);

    let body = html! {
        (navbar(MenuConstants::Factures))
        (new_facture_new_event(event_form.body))
        (footer())
        (event_form.javascript)
    };
    page("Nouvel événement", body)
}
pub fn page_select_item(facture_id: u64, products: Vec<ProductInfo>) -> Markup {
    let body = html! {
        (navbar(MenuConstants::Factures))
        (select_item(facture_id, products))
        (footer())
        (search_products())
    };
    page("Nouvel événement", body)
}

pub fn page_add_item(page_data: PageAddOneFactureItemData) -> Markup {
    let body = html! {
        (navbar(MenuConstants::Factures))
        (add_item(page_data))
        (footer())
        (item_form_scripts())
    };
    page("Item de facture", body)
}

pub fn page_transactions(page_data: PageTransactionsData) -> Markup {
    let content = list_transactions(page_data);
    let body = html! {
        (navbar(MenuConstants::Factures))
        (content.body)
        (footer())
        (content.javascript)
    };
    page("Item de facture", body)
}

pub fn page_add_product(page_data: PageAddProduct) -> Markup {
    let body = html! {
        (navbar(MenuConstants::Factures))
        (add_product(page_data.facture_info.facture.id, &page_data.product_types))
        (footer())
    };
    page("Ajouter un produit", body)
}

pub fn page_print(page_data: PagePrintData) -> Markup {
    fn print_total(facture_data: &PagePrintData) -> Markup {
        html! {
            table."table" {
                tbody {
                    tr {
                        th {
                            "Sous-total:"
                        }
                        td."text-right" {
                            (facture_data.facture_info.facture_computed.total)
                        }
                    }
                    tr {
                        th {
                            "TPS 5%:"
                        }
                        td."text-right" {
                            (facture_data.facture_info.facture_computed.tps)
                        }
                    }
                    tr {
                        th {
                            "TVQ 9.975%:"
                        }
                        td."text-right" {
                            (facture_data.facture_info.facture_computed.tvq)
                        }
                    }
                    tr {
                        th {
                            "Total:"
                        }
                        td."text-right" {
                            (facture_data.facture_info.facture_computed.tax_total)
                        }
                    }
                    tr {
                        td colspan="2" {
                            b {
                                "La Mariée à l'Honneur Inc.  # TPS: "
                            }
                            "786527291"
                            b {
                                "# TVQ: "
                            }
                            "1223489504"
                        }
                    }
                    tr {
                        @if facture_data.payments.iter().count() > 0 {
                            th {
                                "Paiements reçus:"
                            }
                            td."text-right" {
                                ul style=("list-style-position: inside;") {
                                    @for p in &facture_data.payments {
                                        li {
                                            (p.date) " " (p.payment_type) " - " (p.amount)
                                        }
                                    }
                                }
                            }
                        } @else {
                            span {
                                "Aucun paiement reçu"
                            }
                        }
                    }
                    tr {
                        @if facture_data.refunds.iter().count() > 0 {
                            th {
                                "Remboursements reçus:"
                            }
                            td."text-right" {
                                ul style=("list-style-position: inside;") {
                                    @for r in &facture_data.refunds {
                                        li {
                                            (r.date) " " (r.refund_type) " - " (r.amount) @if let Some(c) = r.cheque_number.as_ref() { "No chèque: " (c) }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    tr."border-top-lg" {
                        th {
                            "Balance à recevoir:"
                        }
                        td."text-right" {
                            (facture_data.facture_info.facture_computed.balance)
                        }
                    }
                }
            }
        }
    }
    fn display_notes(note: &str) -> Markup {
        let notes: Vec<&str> = note.split("\n").collect();
        html! {
            @for n in notes {
                li { i { (n) } }
            }
        }
    }
    fn list_the_items_row(entry: &FactureItemInfo) -> Markup {
        match &entry.item.value {
            FactureItemValue::FactureItemProduct(value) => {
                html! {
                    tr {
                        td {
                            (value.quantity)
                        }
                        td {
                            (entry.product_info.product.name)
                        }
                        td {
                            @if let Some(p) = &value.price {
                                (p)
                            }
                        }
                        td."details" {
                            ul."list-unstyled ml-0" {
                                @if let Some(percent) = &value.rebate_percent.filter(|a| *a > 0) {
                                    li { b { "Rabais de " (percent) "% appliqué sur le prix unistaire:" } (entry.item_computed.calculated_rebate) }
                                }
                                @if let Some(ex) = &value.extra_large_size {
                                    li { b { "Frais additionnels pour taille forte: " } (ex) }
                                }
                                @if entry.product_info.types.iter().any(|pt| pt.is_dress()) && !value.floor_item {
                                    li { b { "Mesures: " } (entry.item_computed.measurements) }
                                }
                                @if value.floor_item {
                                    li { "Modèle planché" }
                                }
                                @if let Some(n) = value.notes.as_ref() {
                                    (display_notes(&n))
                                }
                            }
                        }
                        td {
                            @if let Some(b) = &value.beneficiary {
                                (b)
                            }
                        }
                        td."text-right" {
                            (entry.item_computed.total)
                        }
                    }
                }
            }
            FactureItemValue::FactureItemLocation(value) => html! {
                tr {
                    td {
                        (value.quantity)
                    }
                    td."details" {
                        ul."list-unstyled ml-0" {
                            li { b { "Location de vêtements et d'accessoires" } }
                            @if let Some(ass) = &value.insurance {
                                li { b { "Frais d'assurances: " } (ass) }
                            }
                            @if let Some(oc) = &value.other_costs {
                                li { b { "Autres frais supplémentaires: " } (oc) }
                            }
                            @if let Some(n) = value.notes.as_ref() {
                                (display_notes(&n))
                            }
                        }
                    }
                    td {
                        @if let Some(b) = &value.beneficiary {
                            (b)
                        }
                    }
                    td {
                        @if let Some(p) = value.price {
                            (p)
                        }
                    }
                }
            },
            FactureItemValue::FactureItemAlteration(value) => html! {

                tr {
                    td {
                        (value.quantity)
                    }
                    td {
                        (entry.product_info.product.name)
                    }
                    td {
                        @if let Some(p) = value.price {
                            (p)
                        }
                    }
                    td."details" {
                        ul."list-unstyled ml-0" {
                            @if let Some(rebate) = &value.rebate_dollar {
                                li { b { "Rabais appliqué sur le prix unitaire: " } (rebate) }
                            }
                            @if let Some(n) = value.notes.as_ref() {
                                (display_notes(&n))
                            }
                        }
                    }
                    td."text-right" {
                        (entry.item_computed.total)
                    }
                }
            },
        }
    }
    fn list_the_items(facture_data: &PagePrintData) -> Markup {
        let default = String::from("Product");
        let facture_type = facture_data
            .facture_info
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
                            "Quantité"
                        }
                        th scope="col" {
                            "Détails"
                        }
                        th scope="col" {
                            "Bénéficiaire"
                        }
                        th scope="col" {
                            "Prix"
                        }
                    }
                }
            },
            "Alteration" => html! {
                thead {
                    tr {
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
                            "Details"
                        }
                        th scope="col" {
                            "Prix"
                        }
                    }
                }
            },
            _ => html! {
                thead {
                    tr {
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
                            "Détails"
                        }
                        th scope="col" {
                            "Bénéficiaire"
                        }
                        th scope="col" {
                            "Prix"
                        }
                    }
                }
            },
        };
        html! {
            @if facture_data.items.iter().count() > 0 {
                table."table print-items" {
                    (header)
                    tbody {
                        @for entry in &facture_data.items {
                            (list_the_items_row(entry))
                        }
                    }
                }

            } @else {
                p { "Il n'y a aucun item dans la facture." }
            }
        }
    }

    let title_page = format!("Facture #{}", page_data.facture_info.facture.id);
    html! {
        (DOCTYPE)
        html lang="fr" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no";
                title {
                    (title_page)
                }
                (bootstrap_css())
                link href="https://fonts.googleapis.com/css?family=Lobster" rel="stylesheet";
                style {
                    (PreEscaped(
                    "body {
                        font-size: 0.75rem;
                    }
                    /*https://stackoverflow.com/questions/585254/how-to-remove-the-url-from-the-printing-page*/
                    @page {
                        size: letter;
                        margin: 0; /* this affects the margin in the printer settings */
                    }
                    .container {
                        max-width: 8.5in;
                    }
                    .print {
                        margin-top: 0.5in;
                        margin-right: 0.5in;
                        margin-left: 0.5in;
                    }
                    .brand {
                        text-align: right;
                        font-family: \"Lobster\", cursive;
                        font-size: 2rem;
                    }
                    .spacer-100 {
                        height: 100px;
                    }
                    .clauses {
                        font-size: 0.5rem;
                    }
                    .total table tr.border-top-lg td,
                    .total table tr.border-top-lg th {
                        border-top: 2px solid black;
                    }
                    .table.client tbody th,
                    .table.client tbody td {
                        padding: 0;
                    }
                    .print .print-items td.details {
                        max-width: 400px;
                        word-wrap: break-word;
                    }
                    .inline-block {
                        display: inline-block;
                    }"
                    ))
                }
            }
            body {
                div."container" {
                    div."print" {
                        table style="width: 100%;" {
                            tr {
                                td style="width: 30%;" {
                                    ul."list-unstyled" {
                                        @let facture_str = format!("#{}", page_data.facture_info.facture.id);
                                        @let paper_ref_str = &page_data.facture_info.facture.paper_ref.as_ref().map(|a| format!("(Réf. ancienne: {})", a));

                                        li {
                                            b {
                                                "Facture: "
                                            }
                                            (facture_str) " " @if let Some(pr) = paper_ref_str { (pr) }
                                        }
                                        li {
                                            b {
                                                "Date de facturation: "
                                            }
                                            @if let Some(date) = page_data.facture_info.facture.date.as_ref() { (date) }
                                        }
                                    }
                                }
                                td."text-right" style="width: 70%;" {
                                    span."brand" {
                                        "La Mariée à l'Honneur"
                                    }
                                }
                            }
                            tr {
                                td style="width: 50%;" {}
                                td."text-right" style="width: 50%;" {
                                    img src="data:image/png;base64,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" alt="logo" width="100" height="100";
                                    (" ")
                                    div."text-right inline-block" {
                                        p {
                                            "2023 route Marie-Victorin"
                                            br;
                                            "Varennes, Québec, (J3X 1R3)"
                                        }
                                        p {
                                            "450-652-4771"
                                            br;
                                            "514-880-4771"
                                        }
                                    }
                                }
                            }

                            tr {
                                td colspan="2" {
                                    table."table client table-borderless" style="width: 42%;" {
                                        tbody {
                                            tr {
                                                th {
                                                    "Nom:"
                                                }
                                                td {
                                                    (page_data.facture_info.client.name())
                                                }
                                            }
                                            tr {
                                                th {
                                                    "Rue:"
                                                }
                                                td {
                                                    @if let Some(c) = &page_data.facture_info.client.street.as_ref() {
                                                        (c)
                                                    }
                                                }
                                            }
                                            tr {
                                                th {
                                                    "Ville:"
                                                }
                                                td {
                                                    @if let Some(city) = &page_data.facture_info.client.city.as_ref() {
                                                        (city)
                                                    }
                                                }
                                            }
                                            tr {
                                                th {
                                                    "Téléphone #1:"
                                                }
                                                td {
                                                    (page_data.facture_info.client.phone1)
                                                }
                                            }
                                            tr {
                                                th {
                                                    "Téléphone #2:"
                                                }
                                                td {
                                                    @if let Some(phone2) = &page_data.facture_info.client.phone2.as_ref() {
                                                        (phone2)
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            tr {
                                td colspan="2" {
                                    (list_the_items(&page_data))
                                }
                            }
                            tr."spacer-100" {}
                            tr {
                                td colspan="2" {
                                    (print_total(&page_data))
                                }
                            }
                            tr {
                                td."clauses" colspan="2" {
                                    @for c in &page_data.print_config.clauses {
                                        p { (c) }
                                    }
                                }
                            }
                            @if page_data.print_config.signatures.is_empty() {
                                tr {
                                    td."text-right" colspan="2" {
                                        b {
                                            "Signature:"
                                        }
                                        "_________________________________________"
                                    }
                                }
                            }
                            @for s in &page_data.print_config.signatures {
                                tr {
                                    td colspan="2" {
                                        span {
                                            (s)
                                        }
                                    }
                                }
                                tr."mt-3" {
                                    td."text-right" colspan="2" {
                                        b {
                                            "Signature:"
                                        }
                                        "_________________________________________"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            script type="text/javascript" {
                (PreEscaped(r#"
                    window.onload = function() {
                        if (parent && parent.iframeLoaded) {
                            parent.iframeLoaded();
                        }
                    }
                "#))
            }
        }
    }
}
