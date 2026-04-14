use crate::server::models::{
    clients::ClientView,
    config::{ExtraLargeAmounts, NoteTemplate},
    events::EventView,
    facture_items::{FactureComputed, FactureItemComputed, FactureItemType, FactureItemView},
    factures::FactureView,
    payments::{PaymentView, PreCalculatedPayment},
    product_types::ProductTypeView,
    products::ProductView,
    refunds::RefundView,
    statuts::{State, StateView},
};

pub mod clients;
pub mod config;
pub mod events;
pub mod facture_items;
pub mod factures;
pub mod payments;
pub mod product_types;
pub mod products;
pub mod refunds;
pub mod statuts;

pub struct FactureDashboardData {
    pub facture: FactureView,
    pub client: ClientView,
    pub state_per_item: Vec<(i64, StateView)>,
}

impl FactureDashboardData {
    pub fn seamstresses(&self) -> Vec<String> {
        self.state_per_item
            .iter()
            .filter_map(|(_, state)| match &state.current_state {
                State::GivenToSeamstress(_, seamstress) => Some(seamstress.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn smallest_state(&self) -> Option<StateView> {
        self.state_per_item
            .iter()
            .min_by_key(|&a| a.1.current_state.value())
            .map(|(_, state)| state.clone())
    }
}

pub struct FactureItemsData {
    pub facture_info: FactureInfo,
    pub items: Vec<FactureItemEntry<FactureItemView>>,
    pub items_computed: Vec<FactureItemComputed>,
}

pub struct FactureItemEntry<Item> {
    pub item: Item,
    pub product: ProductView,
    pub state: StateView,
}

pub struct PageFactureItemsData {
    pub facture_data: FactureItemsData,
    pub alteration_product: ProductView,
    pub location_product: ProductView,
}

pub struct FactureItemFormConfig {
    pub note_templates: Vec<NoteTemplate>,
    pub extra_large_amount: ExtraLargeAmounts,
    pub seamstresses: Vec<String>,
}

pub struct PageAddOneFactureItemData {
    pub facture_info: FactureInfo,
    pub item: FactureItemEntry<FactureItemType>,
    pub product_type: ProductTypeView,
    pub form_config: FactureItemFormConfig,
}

pub struct PageAddProduct {
    pub facture_info: FactureInfo,
    pub product_types: Vec<ProductTypeView>,
}

pub struct PageOneFactureItemData {
    pub facture: FactureView,
    pub client: ClientView,
    pub item: FactureItemEntry<FactureItemView>,
    pub product_type: ProductTypeView,
    pub form_config: FactureItemFormConfig,
}
pub struct PageTransactionsData {
    pub facture_info: FactureInfo,
    pub payments: Vec<PaymentView>,
    pub refunds: Vec<RefundView>,
}

pub struct FactureInfo {
    pub facture: FactureView,
    pub facture_computed: FactureComputed,
    pub event: Option<EventView>,
    pub client: ClientView,
}

pub const PAYMENT_TYPES: [&str; 5] = [
    "Mastercard",
    "Visa",
    "American Express",
    "Interac",
    "Argent comptant",
];
pub const REFUND_TYPES: [&str; 6] = [
    "Mastercard",
    "Visa",
    "American Express",
    "Interac",
    "Argent comptant",
    "Chèque",
];

pub type Transaction<'a> = TheTransaction<'a, PaymentView, RefundView>;
pub type MaybeTransaction<'a> = TheTransaction<'a, Option<&'a PaymentView>, Option<&'a RefundView>>;
pub enum TheTransaction<'a, P, R> {
    Payment(&'a P),
    Refund(&'a R),
}

impl Transaction<'_> {
    pub fn is_refund(&self) -> bool {
        match self {
            TheTransaction::Payment(_) => false,
            TheTransaction::Refund(_) => true,
        }
    }
    pub fn is_payment(&self) -> bool {
        match self {
            TheTransaction::Payment(_) => true,
            TheTransaction::Refund(_) => false,
        }
    }
    pub fn id(&self) -> i64 {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.id,
            TheTransaction::Refund(refund_view) => refund_view.id,
        }
    }
    pub fn facture_id(&self) -> i64 {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.facture_id,
            TheTransaction::Refund(refund_view) => refund_view.facture_id,
        }
    }
    pub fn amount(&self) -> i64 {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.amount,
            TheTransaction::Refund(refund_view) => refund_view.amount,
        }
    }
    pub fn date(&self) -> &str {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.date.as_str(),
            TheTransaction::Refund(refund_view) => refund_view.date.as_str(),
        }
    }
    pub fn t_type(&self) -> &str {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.payment_type.as_str(),
            TheTransaction::Refund(refund_view) => refund_view.refund_type.as_str(),
        }
    }
    pub fn cheque_number(&self) -> Option<&str> {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.cheque_number.as_deref(),
            TheTransaction::Refund(refund_view) => refund_view.cheque_number.as_deref(),
        }
    }
    pub fn created_at(&self) -> &str {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.created_at.as_str(),
            TheTransaction::Refund(refund_view) => refund_view.created_at.as_str(),
        }
    }
    pub fn updated_at(&self) -> &str {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.updated_at.as_str(),
            TheTransaction::Refund(refund_view) => refund_view.updated_at.as_str(),
        }
    }
}

impl MaybeTransaction<'_> {
    pub fn is_none(&self) -> bool {
        match &self {
            TheTransaction::Payment(None) | TheTransaction::Refund(None) => true,
            _ => false,
        }
    }
    pub fn is_refund(&self) -> bool {
        match self {
            TheTransaction::Payment(_) => false,
            TheTransaction::Refund(_) => true,
        }
    }
    pub fn is_payment(&self) -> bool {
        match self {
            TheTransaction::Payment(_) => true,
            TheTransaction::Refund(_) => false,
        }
    }
    pub fn id(&self) -> Option<i64> {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.map(|a| a.id),
            TheTransaction::Refund(refund_view) => refund_view.map(|a| a.id),
        }
    }
    pub fn facture_id(&self) -> Option<i64> {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.map(|a| a.facture_id),
            TheTransaction::Refund(refund_view) => refund_view.map(|a| a.facture_id),
        }
    }
    pub fn amount(&self) -> Option<i64> {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.map(|a| a.amount),
            TheTransaction::Refund(refund_view) => refund_view.map(|a| a.amount),
        }
    }
    pub fn date(&self) -> Option<&str> {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.as_ref().map(|a| a.date.as_str()),
            TheTransaction::Refund(refund_view) => refund_view.as_ref().map(|a| a.date.as_str()),
        }
    }
    pub fn t_type(&self) -> Option<&str> {
        match self {
            TheTransaction::Payment(payment_view) => {
                payment_view.as_ref().map(|a| a.payment_type.as_str())
            }
            TheTransaction::Refund(refund_view) => {
                refund_view.as_ref().map(|a| a.refund_type.as_str())
            }
        }
    }
    pub fn cheque_number(&self) -> Option<&str> {
        match self {
            TheTransaction::Payment(payment_view) => payment_view
                .as_ref()
                .and_then(|a| a.cheque_number.as_deref()),
            TheTransaction::Refund(refund_view) => refund_view
                .as_ref()
                .and_then(|a| a.cheque_number.as_deref()),
        }
    }
    pub fn created_at(&self) -> Option<&str> {
        match self {
            TheTransaction::Payment(payment_view) => {
                payment_view.as_ref().map(|a| a.created_at.as_str())
            }
            TheTransaction::Refund(refund_view) => {
                refund_view.as_ref().map(|a| a.created_at.as_str())
            }
        }
    }
    pub fn updated_at(&self) -> Option<&str> {
        match self {
            TheTransaction::Payment(payment_view) => {
                payment_view.as_ref().map(|a| a.updated_at.as_str())
            }
            TheTransaction::Refund(refund_view) => {
                refund_view.as_ref().map(|a| a.updated_at.as_str())
            }
        }
    }
}

pub fn initial_payment_amount(f: &FactureInfo) -> PreCalculatedPayment {
    match f.facture.facture_type.as_ref().map(|a| a.as_str()) {
        Some("Altération") => PreCalculatedPayment {
            is_alteration: true,
            amount_ratio: 50,
            tax_total: f.facture_computed.tax_total,
            balance: f.facture_computed.balance,
        },
        _ => PreCalculatedPayment {
            is_alteration: false,
            amount_ratio: 60,
            tax_total: f.facture_computed.tax_total,
            balance: f.facture_computed.balance,
        },
    }
}
