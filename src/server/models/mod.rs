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
    pub facture: factures::FactureView,
    pub client: clients::ClientView,
    pub state_per_item: Vec<(u64, statuts::StateView)>,
}

impl FactureDashboardData {
    pub fn seamstresses(&self) -> Vec<String> {
        self.state_per_item
            .iter()
            .filter_map(|(_, state)| match &state.current_state {
                statuts::State::GivenToSeamstress(_, seamstress) => Some(seamstress.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn smallest_state(&self) -> Option<statuts::StateView> {
        self.state_per_item
            .iter()
            .min_by_key(|&a| a.1.current_state.value())
            .map(|(_, state)| state.clone())
    }
}

pub struct FactureItemsData {
    pub facture_info: FactureInfo,
    pub items: Vec<FactureItemEntry<facture_items::FactureItemView>>,
}

pub struct PagePrintData {
    pub facture_info: FactureInfo,
    pub items: Vec<FactureItemInfo>,
    pub payments: Vec<payments::PaymentView>,
    pub refunds: Vec<refunds::RefundView>,
    pub print_config: PrintConfig,
}

pub struct PrintConfig {
    pub signatures: Vec<String>,
    pub clauses: Vec<String>,
}

pub struct FactureItemInfo {
    pub item: facture_items::FactureItemView,
    pub item_computed: facture_items::FactureItemComputed,
    pub product_info: products::ProductInfo,
}

pub struct FactureItemEntry<Item> {
    pub item: Item,
    pub product: products::ProductView,
    pub state: statuts::StateView,
}

pub struct PageFactureItemsData {
    pub facture_data: FactureItemsData,
    pub alteration_product: products::ProductView,
    pub location_product: products::ProductView,
}

pub struct FactureItemFormConfig {
    pub note_templates: Vec<config::NoteTemplate>,
    pub extra_large_amount: config::ExtraLargeAmounts,
    pub seamstresses: Vec<String>,
}

pub struct PageAddOneFactureItemData {
    pub facture_info: FactureInfo,
    pub item: FactureItemEntry<facture_items::FactureItemType>,
    pub product_type: product_types::ProductTypeView,
    pub form_config: FactureItemFormConfig,
}

pub struct PageAddProduct {
    pub facture_info: FactureInfo,
    pub product_types: Vec<product_types::ProductTypeView>,
}

pub struct PageOneFactureItemData {
    pub facture: factures::FactureView,
    pub client: clients::ClientView,
    pub item: FactureItemEntry<facture_items::FactureItemView>,
    pub product_type: product_types::ProductTypeView,
    pub form_config: FactureItemFormConfig,
}
pub struct PageTransactionsData {
    pub facture_info: FactureInfo,
    pub payments: Vec<payments::PaymentView>,
    pub refunds: Vec<refunds::RefundView>,
}

pub struct FactureInfo {
    pub facture: factures::FactureView,
    pub facture_computed: facture_items::FactureComputed,
    pub event: Option<events::EventView>,
    pub client: clients::ClientView,
}

pub struct FactureAndClient {
    pub facture: factures::FactureView,
    pub client: clients::ClientView,
}

pub struct PageOneEvent {
    pub event: events::EventView,
    pub event_types: Vec<String>,
    pub related_factures: Vec<FactureAndClient>,
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

pub type Transaction<'a> = TheTransaction<'a, payments::PaymentView, refunds::RefundView>;
pub type MaybeTransaction<'a> =
    TheTransaction<'a, Option<&'a payments::PaymentView>, Option<&'a refunds::RefundView>>;
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
    pub fn id(&self) -> u64 {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.id,
            TheTransaction::Refund(refund_view) => refund_view.id,
        }
    }
    pub fn facture_id(&self) -> u64 {
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
    pub fn id(&self) -> Option<u64> {
        match self {
            TheTransaction::Payment(payment_view) => payment_view.map(|a| a.id),
            TheTransaction::Refund(refund_view) => refund_view.map(|a| a.id),
        }
    }
    pub fn facture_id(&self) -> Option<u64> {
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

pub fn initial_payment_amount(f: &FactureInfo) -> payments::PreCalculatedPayment {
    match f.facture.facture_type.as_ref().map(|a| a.as_str()) {
        Some("Altération") => payments::PreCalculatedPayment {
            is_alteration: true,
            amount_ratio: 50,
            tax_total: f.facture_computed.tax_total,
            balance: f.facture_computed.balance,
        },
        _ => payments::PreCalculatedPayment {
            is_alteration: false,
            amount_ratio: 60,
            tax_total: f.facture_computed.tax_total,
            balance: f.facture_computed.balance,
        },
    }
}
