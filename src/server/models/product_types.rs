use sqlx::prelude::FromRow;

/// Database row structure for product_types table
#[derive(Debug, FromRow)]
pub struct ProductTypeRow {
    pub name: String,
}

#[derive(Debug)]
pub struct ProductTypeView {
    pub name: String,
}

impl From<ProductTypeRow> for ProductTypeView {
    fn from(value: ProductTypeRow) -> Self {
        ProductTypeView { name: value.name }
    }
}

impl ProductTypeView {
    pub fn is_dress(&self) -> bool {
        self.name.starts_with("Robe de ")
    }
    pub fn is_wedding(&self) -> bool {
        self.name == "Robe de mariée"
    }
    pub fn is_mom(&self) -> bool {
        self.name == "Robe de mère de la mariée"
    }
    pub fn is_bal(&self) -> bool {
        self.name == "Robe de bal"
    }
    pub fn is_bouq(&self) -> bool {
        self.name == "Robe de bouquetière"
    }
    pub fn is_gaine(&self) -> bool {
        self.name == "Gaine"
    }
}
