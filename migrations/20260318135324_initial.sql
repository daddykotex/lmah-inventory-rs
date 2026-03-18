-- ============================================================================
-- LMAH Inventory - SQLite Schema Migration
-- ============================================================================
-- Description: Complete database schema for LMAH Inventory system
-- Source: Migrating from JSON (data/db.json) to SQLite
-- Date: 2026-03-17
-- Tables: 12 (config, clients, events, products, product_types,
--              product_product_types, product_images, factures, facture_items,
--              payments, refunds, statuts)
-- ============================================================================

-- Enable foreign key constraints (MUST be set on every connection)
PRAGMA foreign_keys = ON;
PRAGMA journal_mode=WAL;

-- ============================================================================
-- TABLE DEFINITIONS (in dependency order)
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. CONFIG - Application configuration
-- ----------------------------------------------------------------------------
-- Purpose: Store configuration values (clauses, signatures, event types, etc.)
-- Maps to: Site model (site.scala:51-58)
-- ----------------------------------------------------------------------------

CREATE TABLE config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    type TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ----------------------------------------------------------------------------
-- 2. CLIENTS - Customer information
-- ----------------------------------------------------------------------------
-- Purpose: Store customer/client data
-- Maps to: LMAHClient model (clients.scala:7-20)
-- ----------------------------------------------------------------------------

CREATE TABLE clients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    airtable_id TEXT NOT NULL UNIQUE,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    street TEXT,
    city TEXT NOT NULL,
    phone1 TEXT NOT NULL,
    phone2 TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ----------------------------------------------------------------------------
-- 3. EVENTS - Events (weddings, proms, etc.)
-- ----------------------------------------------------------------------------
-- Purpose: Store event information
-- Maps to: LMAHEvent model (events.scala:11)
-- ----------------------------------------------------------------------------

CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    airtable_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    date TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ----------------------------------------------------------------------------
-- 4. PRODUCT_TYPES - Product type catalog
-- ----------------------------------------------------------------------------
-- Purpose: Store available product types (from product_types.json)
-- Valid types: See produits.scala:94-118
-- ----------------------------------------------------------------------------

CREATE TABLE product_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ----------------------------------------------------------------------------
-- 5. PRODUCTS - Product/service catalog
-- ----------------------------------------------------------------------------
-- Purpose: Store products (dresses, alterations, accessories)
-- Maps to: LMAHProduit model (produits.scala:23-31)
-- Note: Types and images are in separate tables
-- ----------------------------------------------------------------------------

CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    airtable_id TEXT NOT NULL UNIQUE,
    product_type_id INTEGER NOT NULL REFERENCES product_types(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    price INTEGER NOT NULL,
    liquidation INTEGER,
    visible_on_site INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ----------------------------------------------------------------------------
-- 6. PRODUCT_IMAGES - Product image attachments
-- ----------------------------------------------------------------------------
-- Purpose: Store product images (front/back)
-- Maps to: LinkedAirtableRecordData[AirtableAttachment]
-- ----------------------------------------------------------------------------

CREATE TABLE product_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    filename TEXT NOT NULL,
    position TEXT NOT NULL CHECK(position IN ('front', 'back')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ----------------------------------------------------------------------------
-- 7. FACTURES - Invoices
-- ----------------------------------------------------------------------------
-- Purpose: Store invoices (factures) for products, location, alteration
-- Maps to: LMAHFacture model (factures.scala:231-250)
-- Note: Computed fields (total, balance, TVQ, TPS) are calculated in Scala
-- ----------------------------------------------------------------------------

CREATE TABLE factures (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    airtable_id TEXT NOT NULL UNIQUE,
    client_id INTEGER NOT NULL REFERENCES clients(id) ON DELETE RESTRICT,
    type TEXT CHECK(type IN ('Produits', 'Location', 'Altération')),
    date TEXT,
    event_id INTEGER REFERENCES events(id) ON DELETE SET NULL,
    fixed_total INTEGER,
    cancelled INTEGER NOT NULL DEFAULT 0,
    paper_ref TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ----------------------------------------------------------------------------
-- 8. FACTURE_ITEMS - Invoice line items (polymorphic)
-- ----------------------------------------------------------------------------
-- Purpose: Line items on factures (3 types: Produit, Location, Alteration)
-- Maps to: LMAHItemProduit, LMAHItemLocation, LMAHItemAlteration (items.scala)
-- Design: Single table with type discriminator + nullable type-specific columns
-- ----------------------------------------------------------------------------

CREATE TABLE facture_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    airtable_id TEXT NOT NULL UNIQUE,
    facture_id INTEGER NOT NULL REFERENCES factures(id) ON DELETE CASCADE,
    product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    item_type TEXT NOT NULL CHECK(item_type IN ('Produit', 'Location', 'Alteration')),

    -- Common fields (all types)
    price INTEGER,
    notes TEXT,
    quantity INTEGER DEFAULT 1,

    -- Produit-specific fields (items.scala:53-74)
    extra_large_size INTEGER,
    rebate_percent INTEGER,
    size TEXT,
    chest INTEGER,
    waist INTEGER,
    hips INTEGER,
    color TEXT,
    beneficiary TEXT,
    floor_item INTEGER DEFAULT 0,

    -- Location-specific fields (items.scala:208-222)
    insurance INTEGER,
    other_costs INTEGER,
    files TEXT,

    -- Alteration-specific fields (items.scala:298-310)
    rebate_dollar INTEGER,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ----------------------------------------------------------------------------
-- 9. PAYMENTS - Payment transactions
-- ----------------------------------------------------------------------------
-- Purpose: Store payment records for factures
-- Maps to: LMAHPayment model (transactions.scala:28-33)
-- Valid types: Mastercard, Visa, American Express, Interac, Argent comptant
-- ----------------------------------------------------------------------------

CREATE TABLE payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    airtable_id TEXT NOT NULL UNIQUE,
    facture_id INTEGER NOT NULL REFERENCES factures(id) ON DELETE CASCADE,
    amount INTEGER NOT NULL,
    date TEXT NOT NULL,
    type TEXT NOT NULL,
    cheque_number TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ----------------------------------------------------------------------------
-- 10. REFUNDS - Refund transactions
-- ----------------------------------------------------------------------------
-- Purpose: Store refund records for factures
-- Maps to: LMAHRefund model (transactions.scala:148-154)
-- Valid types: Mastercard, Visa, American Express, Interac, Argent comptant, Chèque
-- ----------------------------------------------------------------------------

CREATE TABLE refunds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    airtable_id TEXT NOT NULL UNIQUE,
    facture_id INTEGER NOT NULL REFERENCES factures(id) ON DELETE CASCADE,
    amount INTEGER NOT NULL,
    date TEXT NOT NULL,
    type TEXT NOT NULL,
    cheque_number TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ----------------------------------------------------------------------------
-- 11. STATUTS - State machine history
-- ----------------------------------------------------------------------------
-- Purpose: Track state transitions for facture items (workflow tracking)
-- Maps to: LMAHStatus model (state.scala:546-552)
-- Design: Append-only history, current state = most recent status by date
-- Valid types: See state.scala:70-544 (4 state machines)
-- ----------------------------------------------------------------------------

CREATE TABLE statuts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    airtable_id TEXT NOT NULL UNIQUE,
    facture_id INTEGER NOT NULL REFERENCES factures(id) ON DELETE CASCADE,
    facture_item_id INTEGER NOT NULL REFERENCES facture_items(id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    date TEXT NOT NULL,
    seamstress TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- INDEXES (for query performance)
-- ============================================================================

-- Clients indexes
CREATE INDEX idx_clients_name ON clients(last_name, first_name);
CREATE INDEX idx_clients_airtable_id ON clients(airtable_id);

-- Events indexes
CREATE INDEX idx_events_date ON events(date DESC);
CREATE INDEX idx_events_type ON events(type);
CREATE INDEX idx_events_airtable_id ON events(airtable_id);

-- Products indexes
CREATE INDEX idx_products_name ON products(name);
CREATE INDEX idx_products_price ON products(price);
CREATE INDEX idx_products_visible ON products(visible_on_site);
CREATE INDEX idx_products_airtable_id ON products(airtable_id);


-- Product images indexes
CREATE INDEX idx_product_images_product ON product_images(product_id);
CREATE INDEX idx_product_images_position ON product_images(product_id, position);

-- Factures indexes
CREATE INDEX idx_factures_client ON factures(client_id);
CREATE INDEX idx_factures_event ON factures(event_id);
CREATE INDEX idx_factures_date ON factures(date DESC);
CREATE INDEX idx_factures_type ON factures(type);
CREATE INDEX idx_factures_cancelled ON factures(cancelled);
CREATE INDEX idx_factures_airtable_id ON factures(airtable_id);

-- Facture items indexes
CREATE INDEX idx_facture_items_facture ON facture_items(facture_id);
CREATE INDEX idx_facture_items_product ON facture_items(product_id);
CREATE INDEX idx_facture_items_type ON facture_items(item_type);
CREATE INDEX idx_facture_items_airtable_id ON facture_items(airtable_id);

-- Payments indexes
CREATE INDEX idx_payments_facture ON payments(facture_id);
CREATE INDEX idx_payments_date ON payments(date DESC);
CREATE INDEX idx_payments_type ON payments(type);
CREATE INDEX idx_payments_airtable_id ON payments(airtable_id);

-- Refunds indexes
CREATE INDEX idx_refunds_facture ON refunds(facture_id);
CREATE INDEX idx_refunds_date ON refunds(date DESC);
CREATE INDEX idx_refunds_airtable_id ON refunds(airtable_id);

-- Statuts indexes (critical for current state queries)
CREATE INDEX idx_statuts_facture ON statuts(facture_id);
CREATE INDEX idx_statuts_item ON statuts(facture_item_id);
CREATE INDEX idx_statuts_date ON statuts(date DESC);
CREATE INDEX idx_statuts_type ON statuts(type);
CREATE INDEX idx_statuts_item_date ON statuts(facture_item_id, date DESC);
CREATE INDEX idx_statuts_airtable_id ON statuts(airtable_id);

-- ============================================================================
-- TRIGGERS (for auto-updating updated_at timestamps)
-- ============================================================================

-- Config trigger
CREATE TRIGGER update_config_timestamp
AFTER UPDATE ON config
FOR EACH ROW
BEGIN
  UPDATE config SET updated_at = datetime('now') WHERE id = OLD.id;
END;

-- Clients trigger
CREATE TRIGGER update_clients_timestamp
AFTER UPDATE ON clients
FOR EACH ROW
BEGIN
  UPDATE clients SET updated_at = datetime('now') WHERE id = OLD.id;
END;

-- Events trigger
CREATE TRIGGER update_events_timestamp
AFTER UPDATE ON events
FOR EACH ROW
BEGIN
  UPDATE events SET updated_at = datetime('now') WHERE id = OLD.id;
END;

-- Products trigger
CREATE TRIGGER update_products_timestamp
AFTER UPDATE ON products
FOR EACH ROW
BEGIN
  UPDATE products SET updated_at = datetime('now') WHERE id = OLD.id;
END;

-- Factures trigger
CREATE TRIGGER update_factures_timestamp
AFTER UPDATE ON factures
FOR EACH ROW
BEGIN
  UPDATE factures SET updated_at = datetime('now') WHERE id = OLD.id;
END;

-- Facture items trigger
CREATE TRIGGER update_facture_items_timestamp
AFTER UPDATE ON facture_items
FOR EACH ROW
BEGIN
  UPDATE facture_items SET updated_at = datetime('now') WHERE id = OLD.id;
END;

-- Payments trigger
CREATE TRIGGER update_payments_timestamp
AFTER UPDATE ON payments
FOR EACH ROW
BEGIN
  UPDATE payments SET updated_at = datetime('now') WHERE id = OLD.id;
END;

-- Refunds trigger
CREATE TRIGGER update_refunds_timestamp
AFTER UPDATE ON refunds
FOR EACH ROW
BEGIN
  UPDATE refunds SET updated_at = datetime('now') WHERE id = OLD.id;
END;

-- Statuts trigger
CREATE TRIGGER update_statuts_timestamp
AFTER UPDATE ON statuts
FOR EACH ROW
BEGIN
  UPDATE statuts SET updated_at = datetime('now') WHERE id = OLD.id;
END;

-- ============================================================================
-- VERIFICATION QUERIES (uncomment to test schema)
-- ============================================================================

-- List all tables
-- SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;

-- Check foreign keys for factures table
-- SELECT * FROM pragma_foreign_key_list('factures');

-- Count records per table (compare with JSON after import)
-- SELECT 'config' as table_name, COUNT(*) as count FROM config
-- UNION ALL SELECT 'clients', COUNT(*) FROM clients
-- UNION ALL SELECT 'events', COUNT(*) FROM events
-- UNION ALL SELECT 'products', COUNT(*) FROM products
-- UNION ALL SELECT 'product_types', COUNT(*) FROM product_types
-- UNION ALL SELECT 'product_images', COUNT(*) FROM product_images
-- UNION ALL SELECT 'factures', COUNT(*) FROM factures
-- UNION ALL SELECT 'facture_items', COUNT(*) FROM facture_items
-- UNION ALL SELECT 'payments', COUNT(*) FROM payments
-- UNION ALL SELECT 'refunds', COUNT(*) FROM refunds
-- UNION ALL SELECT 'statuts', COUNT(*) FROM statuts;

-- ============================================================================
-- SCHEMA COMPLETE
-- ============================================================================
-- To create the database:
--   nix-shell -p sqlite --run "sqlite3 lmah.db < migration.sql"
--
-- To verify the schema:
--   nix-shell -p sqlite --run "sqlite3 lmah.db .schema"
--
-- To check foreign key integrity after import:
--   PRAGMA foreign_key_check;
-- ============================================================================
