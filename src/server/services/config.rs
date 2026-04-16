use anyhow::Result;
use toasty::Db;

use crate::server::models::config::{Config, ConfigView, ExtraLargeAmounts, NoteTemplate};

pub async fn load_note_templates(db: &mut Db) -> Result<Vec<NoteTemplate>> {
    let configs = Config::all().exec(db).await?;
    let config_views: Vec<ConfigView> = configs.into_iter().map(ConfigView::from).collect();

    let templates = config_views
        .into_iter()
        .filter(|config| config.config_type.starts_with("formule-type"))
        .map(|config| NoteTemplate {
            note_type: config.config_type,
            key: config.key,
            value: config.value,
        })
        .collect();

    Ok(templates)
}

pub async fn load_extra_large_amount(db: &mut Db) -> Result<ExtraLargeAmounts> {
    let configs = Config::all().exec(db).await?;
    let config_views: Vec<ConfigView> = configs.into_iter().map(ConfigView::from).collect();

    let mut wedding = None;
    let mut others = None;

    for config in config_views {
        if config.config_type == "extra-taille-forte" {
            if config.key == "robes-de-mariées" {
                wedding = Some(config.value.parse::<i64>()?);
            } else if config.key == "autres-robes" {
                others = Some(config.value.parse::<i64>()?);
            }
        }
    }

    Ok(ExtraLargeAmounts {
        wedding: wedding.ok_or_else(|| anyhow::anyhow!("Missing wedding extra large amount"))?,
        others: others.ok_or_else(|| anyhow::anyhow!("Missing others extra large amount"))?,
    })
}

pub async fn load_seamstresses(db: &mut Db) -> Result<Vec<String>> {
    let configs = Config::all().exec(db).await?;
    let config_views: Vec<ConfigView> = configs.into_iter().map(ConfigView::from).collect();

    let seamstresses = config_views
        .into_iter()
        .filter(|config| config.config_type == "couturiere")
        .map(|config| config.value)
        .collect();

    Ok(seamstresses)
}

pub async fn load_clauses(db: &mut Db) -> Result<Vec<String>> {
    let configs = Config::all().exec(db).await?;
    let config_views: Vec<ConfigView> = configs.into_iter().map(ConfigView::from).collect();

    let clauses = config_views
        .into_iter()
        .filter(|config| config.config_type == "clause-facture")
        .map(|config| config.value)
        .collect();

    Ok(clauses)
}

pub async fn load_signatures(db: &mut Db) -> Result<Vec<String>> {
    let configs = Config::all().exec(db).await?;
    let config_views: Vec<ConfigView> = configs.into_iter().map(ConfigView::from).collect();

    let signatures = config_views
        .into_iter()
        .filter(|config| config.config_type == "signature-facture")
        .map(|config| config.value)
        .collect();

    Ok(signatures)
}

pub async fn load_event_types(db: &mut Db) -> Result<Vec<String>> {
    let configs = Config::all().exec(db).await?;
    let config_views: Vec<ConfigView> = configs.into_iter().map(ConfigView::from).collect();

    let event_types = config_views
        .into_iter()
        .filter(|config| config.config_type == "event-type")
        .map(|config| config.value)
        .collect();

    Ok(event_types)
}
