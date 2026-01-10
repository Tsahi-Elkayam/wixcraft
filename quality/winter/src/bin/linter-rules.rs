//! Linter Rules Management CLI
//!
//! Manage linter rules from various sources (static YAML or database).
//! Generic tool that works with any linter built on the Winter engine.
//!
//! Usage:
//!   linter-rules sync              # Sync rules from configured source
//!   linter-rules export [--format] # Export rules to JSON/YAML
//!   linter-rules list [--category] # List rules
//!   linter-rules show <rule-id>    # Show rule details
//!   linter-rules create            # Create new rule (interactive)
//!   linter-rules update <rule-id>  # Update existing rule
//!   linter-rules delete <rule-id>  # Delete rule

use clap::{Parser, Subcommand};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Type alias for rule data from database sync
type SyncRuleRow = (String, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, bool);

/// Linter Rules Management CLI
#[derive(Parser)]
#[command(name = "linter-rules")]
#[command(about = "Manage linter rules from database or static files")]
struct Cli {
    /// Config file path
    #[arg(short, long, default_value = ".linterrc.yaml")]
    config: PathBuf,

    /// Database path (overrides config)
    #[arg(long)]
    database: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync rules from configured source to plugin
    Sync {
        /// Plugin to sync (default: all)
        #[arg(short, long)]
        plugin: Option<String>,
    },

    /// Export rules to file
    Export {
        /// Output format (json, yaml)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Plugin to export (default: all)
        #[arg(short, long)]
        plugin: Option<String>,
    },

    /// List rules
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Filter by severity
        #[arg(short, long)]
        severity: Option<String>,

        /// Filter by plugin
        #[arg(short, long)]
        plugin: Option<String>,

        /// Show only enabled rules
        #[arg(long)]
        enabled_only: bool,
    },

    /// Show rule details
    Show {
        /// Rule ID
        rule_id: String,
    },

    /// Import rules from JSON/YAML file to database
    Import {
        /// Input file
        file: PathBuf,

        /// Replace existing rules
        #[arg(long)]
        replace: bool,
    },

    /// Create new rule
    Create {
        /// Rule ID
        #[arg(long)]
        id: String,

        /// Category
        #[arg(long)]
        category: String,

        /// Severity (error, warning, info)
        #[arg(long, default_value = "warning")]
        severity: String,

        /// Rule name/message
        #[arg(long)]
        name: String,

        /// Condition expression
        #[arg(long)]
        condition: String,

        /// Description
        #[arg(long)]
        description: Option<String>,

        /// Target element name
        #[arg(long)]
        target: Option<String>,

        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },

    /// Update existing rule
    Update {
        /// Rule ID
        rule_id: String,

        /// New severity
        #[arg(long)]
        severity: Option<String>,

        /// New name/message
        #[arg(long)]
        name: Option<String>,

        /// New condition
        #[arg(long)]
        condition: Option<String>,

        /// Enable/disable
        #[arg(long)]
        enabled: Option<bool>,
    },

    /// Delete rule
    Delete {
        /// Rule ID
        rule_id: String,

        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },

    /// Show statistics
    Stats,
}

/// Rules configuration
#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct RulesConfig {
    #[serde(default)]
    plugins: HashMap<String, PluginRulesConfig>,

    #[serde(default)]
    database: Option<PathBuf>,
}

/// Per-plugin rules configuration
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PluginRulesConfig {
    /// Rule source: "static" or "database"
    #[serde(default = "default_source")]
    source: String,

    /// Path to static rules (for source: static)
    #[serde(default)]
    rules_path: Option<PathBuf>,

    /// Database table/category (for source: database)
    #[serde(default)]
    category: Option<String>,
}

fn default_source() -> String {
    "static".to_string()
}

/// Rule from database
#[derive(Debug, Serialize, Deserialize)]
struct DbRule {
    rule_id: String,
    category: String,
    severity: String,
    name: String,
    description: Option<String>,
    rationale: Option<String>,
    fix_suggestion: Option<String>,
    enabled: bool,
    auto_fixable: bool,
    condition: Option<String>,
    target_kind: Option<String>,
    target_name: Option<String>,
    tags: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Load config
    let config = load_config(&cli.config)?;

    // Get database path
    let db_path = cli.database
        .or(config.database.clone())
        .unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".wixcraft").join("wix.db"))
                .unwrap_or_else(|| PathBuf::from("wix.db"))
        });

    match cli.command {
        Commands::Sync { plugin } => cmd_sync(&db_path, &config, plugin.as_deref())?,
        Commands::Export { format, output, plugin } => {
            cmd_export(&db_path, &format, output.as_deref(), plugin.as_deref())?
        }
        Commands::List { category, severity, plugin, enabled_only } => {
            cmd_list(&db_path, category.as_deref(), severity.as_deref(), plugin.as_deref(), enabled_only)?
        }
        Commands::Show { rule_id } => cmd_show(&db_path, &rule_id)?,
        Commands::Import { file, replace } => cmd_import(&db_path, &file, replace)?,
        Commands::Create { id, category, severity, name, condition, description, target, tags } => {
            cmd_create(&db_path, &id, &category, &severity, &name, &condition,
                       description.as_deref(), target.as_deref(), tags.as_deref())?
        }
        Commands::Update { rule_id, severity, name, condition, enabled } => {
            cmd_update(&db_path, &rule_id, severity.as_deref(), name.as_deref(),
                       condition.as_deref(), enabled)?
        }
        Commands::Delete { rule_id, force } => cmd_delete(&db_path, &rule_id, force)?,
        Commands::Stats => cmd_stats(&db_path)?,
    }

    Ok(())
}

fn load_config(path: &PathBuf) -> anyhow::Result<RulesConfig> {
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        let config: RulesConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(RulesConfig::default())
    }
}

fn cmd_sync(db_path: &PathBuf, _config: &RulesConfig, plugin: Option<&str>) -> anyhow::Result<()> {
    let conn = Connection::open(db_path)?;

    println!("Syncing rules from database: {}", db_path.display());

    // Get rules from database
    let mut stmt = conn.prepare(
        "SELECT rule_id, category, severity, name, description, condition, target_name, tags, enabled
         FROM rules WHERE condition IS NOT NULL"
    )?;

    let rules: Vec<SyncRuleRow> =
        stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get::<_, i32>(8)? == 1,
            ))
        })?.filter_map(|r| r.ok()).collect();

    println!("Found {} rules with conditions", rules.len());

    // Group by category (plugin)
    let mut by_category: HashMap<String, Vec<_>> = HashMap::new();
    for rule in rules {
        by_category.entry(rule.1.clone()).or_default().push(rule);
    }

    for (cat, cat_rules) in &by_category {
        if plugin.is_some() && plugin != Some(cat.as_str()) {
            continue;
        }
        println!("  {}: {} rules", cat, cat_rules.len());
    }

    println!("Sync complete.");
    Ok(())
}

fn cmd_export(db_path: &PathBuf, format: &str, output: Option<&std::path::Path>, plugin: Option<&str>) -> anyhow::Result<()> {
    let conn = Connection::open(db_path)?;

    let query = if let Some(cat) = plugin {
        format!("SELECT rule_id, category, severity, name, description, rationale, fix_suggestion,
                        enabled, auto_fixable, condition, target_kind, target_name, tags
                 FROM rules WHERE category = '{}'", cat)
    } else {
        "SELECT rule_id, category, severity, name, description, rationale, fix_suggestion,
                enabled, auto_fixable, condition, target_kind, target_name, tags
         FROM rules".to_string()
    };

    let mut stmt = conn.prepare(&query)?;
    let rules: Vec<DbRule> = stmt.query_map([], |row| {
        Ok(DbRule {
            rule_id: row.get(0)?,
            category: row.get(1)?,
            severity: row.get(2)?,
            name: row.get(3)?,
            description: row.get(4)?,
            rationale: row.get(5)?,
            fix_suggestion: row.get(6)?,
            enabled: row.get::<_, i32>(7)? == 1,
            auto_fixable: row.get::<_, i32>(8)? == 1,
            condition: row.get(9)?,
            target_kind: row.get(10)?,
            target_name: row.get(11)?,
            tags: row.get(12)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    let content = match format {
        "yaml" | "yml" => serde_yaml::to_string(&rules)?,
        _ => serde_json::to_string_pretty(&rules)?,
    };

    if let Some(out_path) = output {
        std::fs::write(out_path, &content)?;
        println!("Exported {} rules to {}", rules.len(), out_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn cmd_list(db_path: &PathBuf, category: Option<&str>, severity: Option<&str>,
            _plugin: Option<&str>, enabled_only: bool) -> anyhow::Result<()> {
    let conn = Connection::open(db_path)?;

    let mut query = "SELECT rule_id, category, severity, name, enabled FROM rules WHERE 1=1".to_string();

    if let Some(cat) = category {
        query.push_str(&format!(" AND category = '{}'", cat));
    }
    if let Some(sev) = severity {
        query.push_str(&format!(" AND severity = '{}'", sev));
    }
    if enabled_only {
        query.push_str(" AND enabled = 1");
    }

    query.push_str(" ORDER BY category, rule_id");

    let mut stmt = conn.prepare(&query)?;
    let rules: Vec<(String, String, String, String, bool)> = stmt.query_map([], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get::<_, i32>(4)? == 1,
        ))
    })?.filter_map(|r| r.ok()).collect();

    println!("{:<25} {:<15} {:<10} {:<6} NAME", "RULE_ID", "CATEGORY", "SEVERITY", "ON");
    println!("{}", "-".repeat(80));

    for (rule_id, cat, sev, name, enabled) in rules {
        let name_short = if name.len() > 35 { format!("{}...", &name[..32]) } else { name };
        println!("{:<25} {:<15} {:<10} {:<6} {}",
                 rule_id, cat, sev, if enabled { "yes" } else { "no" }, name_short);
    }

    Ok(())
}

fn cmd_show(db_path: &PathBuf, rule_id: &str) -> anyhow::Result<()> {
    let conn = Connection::open(db_path)?;

    let rule: DbRule = conn.query_row(
        "SELECT rule_id, category, severity, name, description, rationale, fix_suggestion,
                enabled, auto_fixable, condition, target_kind, target_name, tags
         FROM rules WHERE rule_id = ?",
        params![rule_id],
        |row| {
            Ok(DbRule {
                rule_id: row.get(0)?,
                category: row.get(1)?,
                severity: row.get(2)?,
                name: row.get(3)?,
                description: row.get(4)?,
                rationale: row.get(5)?,
                fix_suggestion: row.get(6)?,
                enabled: row.get::<_, i32>(7)? == 1,
                auto_fixable: row.get::<_, i32>(8)? == 1,
                condition: row.get(9)?,
                target_kind: row.get(10)?,
                target_name: row.get(11)?,
                tags: row.get(12)?,
            })
        }
    )?;

    println!("Rule: {}", rule.rule_id);
    println!("Category: {}", rule.category);
    println!("Severity: {}", rule.severity);
    println!("Enabled: {}", rule.enabled);
    println!("Name: {}", rule.name);
    if let Some(desc) = &rule.description {
        println!("Description: {}", desc);
    }
    if let Some(cond) = &rule.condition {
        println!("Condition: {}", cond);
    }
    if let Some(target) = &rule.target_name {
        println!("Target: {} ({})", target, rule.target_kind.as_deref().unwrap_or("element"));
    }
    if let Some(tags) = &rule.tags {
        println!("Tags: {}", tags);
    }
    if let Some(fix) = &rule.fix_suggestion {
        println!("Fix: {}", fix);
    }

    Ok(())
}

fn cmd_import(db_path: &PathBuf, file: &PathBuf, replace: bool) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(file)?;

    let rules: Vec<DbRule> = if file.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
        serde_yaml::from_str(&content)?
    } else {
        serde_json::from_str(&content)?
    };

    let conn = Connection::open(db_path)?;

    let mut inserted = 0;
    let mut updated = 0;

    for rule in &rules {
        let exists: bool = conn.query_row(
            "SELECT 1 FROM rules WHERE rule_id = ?",
            params![rule.rule_id],
            |_| Ok(true)
        ).unwrap_or(false);

        if exists && !replace {
            continue;
        }

        conn.execute(
            "INSERT OR REPLACE INTO rules
             (rule_id, category, severity, name, description, rationale, fix_suggestion,
              enabled, auto_fixable, condition, target_kind, target_name, tags)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                rule.rule_id,
                rule.category,
                rule.severity,
                rule.name,
                rule.description,
                rule.rationale,
                rule.fix_suggestion,
                if rule.enabled { 1 } else { 0 },
                if rule.auto_fixable { 1 } else { 0 },
                rule.condition,
                rule.target_kind,
                rule.target_name,
                rule.tags,
            ],
        )?;

        if exists { updated += 1; } else { inserted += 1; }
    }

    println!("Imported {} rules: {} new, {} updated", rules.len(), inserted, updated);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_create(db_path: &PathBuf, id: &str, category: &str, severity: &str, name: &str,
              condition: &str, description: Option<&str>, target: Option<&str>,
              tags: Option<&str>) -> anyhow::Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT INTO rules (rule_id, category, severity, name, description, condition, target_name, tags, enabled)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1)",
        params![id, category, severity, name, description, condition, target, tags],
    )?;

    println!("Created rule: {}", id);
    Ok(())
}

fn cmd_update(db_path: &PathBuf, rule_id: &str, severity: Option<&str>, name: Option<&str>,
              condition: Option<&str>, enabled: Option<bool>) -> anyhow::Result<()> {
    let conn = Connection::open(db_path)?;

    let mut updates = Vec::new();
    if let Some(s) = severity { updates.push(format!("severity = '{}'", s)); }
    if let Some(n) = name { updates.push(format!("name = '{}'", n)); }
    if let Some(c) = condition { updates.push(format!("condition = '{}'", c)); }
    if let Some(e) = enabled { updates.push(format!("enabled = {}", if e { 1 } else { 0 })); }

    if updates.is_empty() {
        println!("Nothing to update");
        return Ok(());
    }

    let query = format!("UPDATE rules SET {} WHERE rule_id = ?", updates.join(", "));
    let affected = conn.execute(&query, params![rule_id])?;

    if affected > 0 {
        println!("Updated rule: {}", rule_id);
    } else {
        println!("Rule not found: {}", rule_id);
    }

    Ok(())
}

fn cmd_delete(db_path: &PathBuf, rule_id: &str, force: bool) -> anyhow::Result<()> {
    if !force {
        println!("Delete rule '{}'? Use --force to confirm.", rule_id);
        return Ok(());
    }

    let conn = Connection::open(db_path)?;
    let affected = conn.execute("DELETE FROM rules WHERE rule_id = ?", params![rule_id])?;

    if affected > 0 {
        println!("Deleted rule: {}", rule_id);
    } else {
        println!("Rule not found: {}", rule_id);
    }

    Ok(())
}

fn cmd_stats(db_path: &PathBuf) -> anyhow::Result<()> {
    let conn = Connection::open(db_path)?;

    println!("Database: {}", db_path.display());
    println!();

    // Total rules
    let total: i32 = conn.query_row("SELECT COUNT(*) FROM rules", [], |r| r.get(0))?;
    let with_condition: i32 = conn.query_row(
        "SELECT COUNT(*) FROM rules WHERE condition IS NOT NULL", [], |r| r.get(0)
    )?;

    println!("Total rules: {}", total);
    println!("With conditions (Winter): {}", with_condition);
    println!();

    // By category
    println!("By category:");
    let mut stmt = conn.prepare("SELECT category, COUNT(*) FROM rules GROUP BY category ORDER BY COUNT(*) DESC")?;
    let cats: Vec<(String, i32)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok()).collect();
    for (cat, count) in cats {
        println!("  {:<20} {}", cat, count);
    }
    println!();

    // By severity
    println!("By severity:");
    let mut stmt = conn.prepare("SELECT severity, COUNT(*) FROM rules GROUP BY severity")?;
    let sevs: Vec<(String, i32)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .filter_map(|r| r.ok()).collect();
    for (sev, count) in sevs {
        println!("  {:<20} {}", sev, count);
    }

    Ok(())
}
