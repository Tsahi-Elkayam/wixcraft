//! UI panels for the MSI Explorer

use eframe::egui::{self, RichText, Rounding, Stroke};
use msi_explorer::TableCategory;

use crate::app::MsiExplorerApp;
use crate::theme::{category_icon, Theme};
use crate::schema::{self, ColumnInfo, DetectedValue, ValueType};

/// Welcome panel shown when no file is open
pub fn welcome_panel(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(60.0);

        // Icon
        ui.label(RichText::new("◈")
            .size(72.0)
            .color(Theme::ACCENT));

        ui.add_space(16.0);

        ui.label(RichText::new("MSI Explorer")
            .size(36.0)
            .color(Theme::TEXT_PRIMARY)
            .strong());

        ui.add_space(8.0);

        ui.label(RichText::new("Smart MSI database viewer with IntelliSense")
            .size(15.0)
            .color(Theme::TEXT_SECONDARY));

        ui.add_space(48.0);

        // Drop zone hint
        egui::Frame::none()
            .fill(Theme::BG_MEDIUM)
            .rounding(Rounding::same(12.0))
            .stroke(Stroke::new(2.0, Theme::BORDER_LIGHT))
            .inner_margin(egui::Margin::symmetric(48.0, 32.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Drop an MSI file here")
                        .size(16.0)
                        .color(Theme::TEXT_SECONDARY));
                    ui.add_space(8.0);
                    ui.label(RichText::new("or click Open in the toolbar")
                        .size(13.0)
                        .color(Theme::TEXT_MUTED));
                });
            });

        ui.add_space(40.0);

        // Smart features
        ui.label(RichText::new("SMART FEATURES")
            .size(11.0)
            .color(Theme::TEXT_MUTED));

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            let width = ui.available_width();
            ui.add_space((width - 500.0).max(0.0) / 2.0);
            feature_card(ui, "◇", "Hover Info", "Column & value descriptions");
            feature_card(ui, "◈", "FK Navigation", "Click to jump to references");
            feature_card(ui, "◎", "Value Detection", "GUIDs, paths, properties");
        });
    });
}

fn feature_card(ui: &mut egui::Ui, icon: &str, title: &str, desc: &str) {
    egui::Frame::none()
        .fill(Theme::BG_MEDIUM)
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon).color(Theme::ACCENT).size(16.0));
                    ui.label(RichText::new(title).color(Theme::TEXT_PRIMARY).size(13.0).strong());
                });
                ui.label(RichText::new(desc).color(Theme::TEXT_MUTED).size(11.0));
            });
        });
}

/// Left panel showing table list
pub fn table_list_panel(ui: &mut egui::Ui, app: &mut MsiExplorerApp) {
    // Search section
    egui::Frame::none()
        .fill(Theme::BG_MEDIUM)
        .inner_margin(egui::Margin::symmetric(12.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⌕").color(Theme::TEXT_MUTED).size(14.0));

                let search_field = egui::TextEdit::singleline(&mut app.search_query)
                    .hint_text("Search all tables...")
                    .desired_width(ui.available_width() - 30.0)
                    .frame(false);

                let response = ui.add(search_field);
                if response.changed() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    app.do_search();
                }

                if !app.search_query.is_empty() {
                    if ui.small_button("×").clicked() {
                        app.search_query.clear();
                        app.search_results.clear();
                    }
                }
            });
        });

    if app.tables.is_empty() {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No file open")
                .color(Theme::TEXT_MUTED)
                .size(13.0));
        });
        return;
    }

    // Tables section header
    egui::Frame::none()
        .fill(Theme::BG_DARK)
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.label(RichText::new("TABLES")
                .color(Theme::TEXT_MUTED)
                .size(11.0));
        });

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            if app.show_categories {
                let categories = [
                    TableCategory::Core,
                    TableCategory::File,
                    TableCategory::Registry,
                    TableCategory::UI,
                    TableCategory::CustomAction,
                    TableCategory::Service,
                    TableCategory::Sequence,
                    TableCategory::Validation,
                    TableCategory::Other,
                ];

                let tables_by_category = app.tables_by_category.clone();

                for category in categories {
                    if let Some(tables) = tables_by_category.get(&category) {
                        if !tables.is_empty() {
                            let cat_name = category.display_name();
                            let icon = category_icon(cat_name);

                            egui::CollapsingHeader::new(
                                RichText::new(format!("{} {} ({})", icon, cat_name, tables.len()))
                                    .color(Theme::TEXT_SECONDARY)
                                    .size(12.0)
                            )
                            .default_open(category == TableCategory::Core)
                            .show(ui, |ui| {
                                for table in tables {
                                    let selected = app.selected_table.as_ref() == Some(table);
                                    table_item(ui, table, selected, || app.select_table(table));
                                }
                            });
                        }
                    }
                }
            } else {
                for table in &app.tables.clone() {
                    let selected = app.selected_table.as_ref() == Some(table);
                    table_item(ui, table, selected, || app.select_table(table));
                }
            }
        });
}

fn table_item(ui: &mut egui::Ui, name: &str, selected: bool, on_click: impl FnOnce()) {
    let bg = if selected { Theme::ACCENT_MUTED } else { Theme::BG_DARK };

    let response = egui::Frame::none()
        .fill(bg)
        .inner_margin(egui::Margin::symmetric(16.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let text_color = if selected { Theme::ACCENT } else { Theme::TEXT_PRIMARY };
                ui.label(RichText::new(name).color(text_color).size(13.0));
            });
        })
        .response;

    if response.interact(egui::Sense::click()).clicked() {
        on_click();
    }

    // Hover effect
    if response.hovered() && !selected {
        ui.painter().rect_filled(
            response.rect,
            Rounding::ZERO,
            Theme::BG_LIGHT,
        );
    }
}

/// Summary panel showing MSI info
pub fn summary_panel(ui: &mut egui::Ui, app: &MsiExplorerApp) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("◆").color(Theme::ACCENT).size(20.0));
        ui.label(RichText::new("Package Summary")
            .size(20.0)
            .color(Theme::TEXT_PRIMARY)
            .strong());
    });

    ui.add_space(16.0);

    if let Some(ref summary) = app.summary {
        // Info card
        egui::Frame::none()
            .fill(Theme::BG_MEDIUM)
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                egui::Grid::new("summary_grid")
                    .num_columns(2)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        if let Some(ref title) = summary.title {
                            info_row(ui, "Title", title);
                        }
                        if let Some(ref author) = summary.author {
                            info_row(ui, "Author", author);
                        }
                        if let Some(ref subject) = summary.subject {
                            info_row(ui, "Subject", subject);
                        }
                        if let Some(platform) = summary.platform() {
                            info_row(ui, "Platform", platform);
                        }
                        if let Some(ref uuid) = summary.uuid {
                            info_row(ui, "Package Code", uuid);
                        }
                    });
            });
    }

    ui.add_space(16.0);

    if let Some(ref stats) = app.stats {
        ui.horizontal(|ui| {
            ui.label(RichText::new("◎").color(Theme::ACCENT).size(16.0));
            ui.label(RichText::new("Statistics")
                .size(16.0)
                .color(Theme::TEXT_PRIMARY)
                .strong());
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            stat_card(ui, "File Size", &format_size(stats.file_size));
            stat_card(ui, "Tables", &stats.table_count.to_string());
            stat_card(ui, "Total Rows", &stats.total_rows.to_string());
        });

        ui.add_space(8.0);

        egui::Frame::none()
            .fill(Theme::BG_MEDIUM)
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Largest table:")
                        .color(Theme::TEXT_SECONDARY)
                        .size(13.0));
                    ui.label(RichText::new(&stats.largest_table)
                        .color(Theme::TEXT_PRIMARY)
                        .size(13.0));
                    ui.label(RichText::new(format!("({} rows)", stats.largest_table_rows))
                        .color(Theme::TEXT_MUTED)
                        .size(13.0));
                });
            });
    }
}

fn info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(RichText::new(label).color(Theme::TEXT_MUTED).size(13.0));
    ui.label(RichText::new(value).color(Theme::TEXT_PRIMARY).size(13.0));
    ui.end_row();
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str) {
    egui::Frame::none()
        .fill(Theme::BG_MEDIUM)
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(value)
                    .size(20.0)
                    .color(Theme::TEXT_PRIMARY)
                    .strong());
                ui.label(RichText::new(label)
                    .size(12.0)
                    .color(Theme::TEXT_MUTED));
            });
        });
}

/// Smart table view panel with tooltips and navigation
pub fn table_view_panel(ui: &mut egui::Ui, app: &mut MsiExplorerApp) {
    let schema = schema::get_schema();

    if let Some(ref table) = app.current_table.clone() {
        let table_schema = schema.get(table.name.as_str());

        // Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("◫").color(Theme::ACCENT).size(18.0));
            ui.label(RichText::new(&table.name)
                .size(18.0)
                .color(Theme::TEXT_PRIMARY)
                .strong());

            ui.add_space(8.0);

            // Badges
            egui::Frame::none()
                .fill(Theme::BG_LIGHT)
                .rounding(Rounding::same(4.0))
                .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("{} rows", table.rows.len()))
                        .color(Theme::TEXT_SECONDARY)
                        .size(12.0));
                });

            egui::Frame::none()
                .fill(Theme::BG_LIGHT)
                .rounding(Rounding::same(4.0))
                .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("{} columns", table.columns.len()))
                        .color(Theme::TEXT_SECONDARY)
                        .size(12.0));
                });

            // Smart indicator
            if table_schema.is_some() {
                ui.add_space(4.0);
                egui::Frame::none()
                    .fill(Theme::ACCENT_MUTED)
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("◇ IntelliSense")
                            .color(Theme::ACCENT)
                            .size(11.0));
                    });
            }
        });

        ui.add_space(12.0);

        // Table with smart features
        egui::Frame::none()
            .fill(Theme::BG_MEDIUM)
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(1.0))
            .show(ui, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .columns(egui_extras::Column::auto().at_least(100.0).resizable(true), table.columns.len())
                        .header(32.0, |mut header| {
                            for col in &table.columns {
                                header.col(|ui| {
                                    let col_info = table_schema.and_then(|s| s.get(col.name.as_str()));

                                    let response = ui.horizontal(|ui| {
                                        // Column name
                                        ui.label(RichText::new(&col.name)
                                            .color(Theme::TEXT_PRIMARY)
                                            .size(12.0)
                                            .strong());

                                        // Badges
                                        if col.primary_key {
                                            ui.label(RichText::new("PK")
                                                .color(Theme::ACCENT)
                                                .size(9.0));
                                        }

                                        if let Some(info) = col_info {
                                            if info.foreign_key.is_some() {
                                                ui.label(RichText::new("FK")
                                                    .color(Theme::WARNING)
                                                    .size(9.0));
                                            }
                                        }
                                    }).response;

                                    // Smart tooltip on hover
                                    if let Some(info) = col_info {
                                        response.on_hover_ui(|ui| {
                                            column_tooltip(ui, &col.name, info);
                                        });
                                    }
                                });
                            }
                        })
                        .body(|body| {
                            // Calculate total rows including pending adds
                            let pending_adds_for_table: Vec<_> = app.pending_adds.iter()
                                .filter(|a| a.table == table.name)
                                .cloned()
                                .collect();
                            let total_rows = table.rows.len() + pending_adds_for_table.len();

                            body.rows(26.0, total_rows, |mut row| {
                                let row_idx = row.index();

                                // Check if this is an existing row or a pending add
                                if row_idx < table.rows.len() {
                                    // Existing row
                                    let is_deleted = app.is_row_deleted(&table.name, row_idx);

                                    if let Some(data_row) = table.rows.get(row_idx) {
                                        for (col_idx, val) in data_row.values.iter().enumerate() {
                                            row.col(|ui| {
                                                if is_deleted {
                                                    // Show deleted row with strikethrough
                                                    let text = val.display();
                                                    ui.label(RichText::new(&text)
                                                        .color(Theme::TEXT_MUTED)
                                                        .strikethrough()
                                                        .size(12.0));
                                                } else {
                                                    let original_text = val.display();
                                                    let col_name = table.columns.get(col_idx)
                                                        .map(|c| c.name.as_str())
                                                        .unwrap_or("");
                                                    let is_pk = table.columns.get(col_idx)
                                                        .map(|c| c.primary_key)
                                                        .unwrap_or(false);
                                                    let col_info = table_schema
                                                        .and_then(|_| table.columns.get(col_idx))
                                                        .and_then(|col| table_schema.and_then(|s| s.get(col.name.as_str())));

                                                    // Get display value (considering pending edits)
                                                    let text = app.get_cell_value(&table.name, row_idx, col_idx, &original_text);
                                                    let is_modified = app.is_cell_modified(&table.name, row_idx, col_idx);

                                                    // Check if we're editing this cell
                                                    let is_editing = app.editing_cell.as_ref()
                                                        .map(|e| e.table == table.name && e.row_idx == row_idx && e.col_idx == col_idx)
                                                        .unwrap_or(false);

                                                    if is_editing {
                                                        // Show inline editor
                                                        if let Some(ref mut edit) = app.editing_cell {
                                                            let response = ui.add(
                                                                egui::TextEdit::singleline(&mut edit.text)
                                                                    .desired_width(ui.available_width() - 8.0)
                                                                    .font(egui::TextStyle::Monospace)
                                                            );
                                                            response.request_focus();
                                                        }
                                                    } else {
                                                        // Show cell with edit capability
                                                        smart_cell_editable(ui, &text, col_info, &table.name, col_name, row_idx, col_idx, is_pk, is_modified, app);
                                                    }
                                                }
                                            });
                                        }

                                        // Show delete/undelete button in edit mode (after last column)
                                        if app.edit_mode && data_row.values.len() == table.columns.len() {
                                            // This would need an extra column, so we handle it via right-click context menu
                                        }
                                    }
                                } else {
                                    // Pending add row
                                    let add_idx = row_idx - table.rows.len();
                                    if let Some(pending_add) = pending_adds_for_table.get(add_idx) {
                                        for (_col_idx, value) in pending_add.values.iter().enumerate() {
                                            row.col(|ui| {
                                                // Show pending add with green color
                                                ui.label(RichText::new(value)
                                                    .color(Theme::SUCCESS)
                                                    .size(12.0));
                                            });
                                        }
                                    }
                                }
                            });
                        });
                });
            });

        // Handle keyboard input for editing
        if app.editing_cell.is_some() {
            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));

            if enter_pressed {
                app.commit_edit();
            } else if escape_pressed {
                app.cancel_edit();
            }
        }
    }
}

/// Render a smart cell with value detection and FK navigation (used in non-editable contexts)
#[allow(dead_code)]
fn smart_cell(ui: &mut egui::Ui, value: &str, col_info: Option<&ColumnInfo>, current_table: &str, current_col: &str, app: &mut MsiExplorerApp) {
    // Format number values according to current format
    let display_value = if let Some(info) = col_info {
        if info.value_type == ValueType::Integer {
            if let Ok(num) = value.parse::<i32>() {
                app.number_format.format_i32(num)
            } else {
                value.to_string()
            }
        } else {
            value.to_string()
        }
    } else {
        value.to_string()
    };

    let truncated = if display_value.len() > 50 {
        format!("{}...", &display_value[..47])
    } else {
        display_value.clone()
    };

    // Detect value type
    let detected = schema::detect_value_type(value);

    // Get attribute decoding if applicable
    let attr_decode = if let Some(info) = col_info {
        if info.value_type == ValueType::Integer {
            if let Ok(num) = value.parse::<i32>() {
                schema::decode_attributes(current_table, current_col, num)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Check if this is a foreign key we can navigate to
    let fk_target = col_info.and_then(|info| info.foreign_key);

    // Render based on value type
    let response = if let Some((target_table, _)) = fk_target {
        // Clickable FK link
        let link = ui.add(
            egui::Label::new(RichText::new(&truncated)
                .color(Theme::ACCENT)
                .size(12.0)
                .underline())
            .sense(egui::Sense::click())
        );

        if link.clicked() {
            app.select_table(target_table);
        }

        link
    } else {
        // Regular cell with type-based coloring
        let color = match detected {
            Some(DetectedValue::Guid) => Theme::TEXT_SECONDARY,
            Some(DetectedValue::PropertyRef) => egui::Color32::from_rgb(180, 140, 255), // Purple
            Some(DetectedValue::Path) => egui::Color32::from_rgb(140, 200, 140), // Green
            Some(DetectedValue::Version) => egui::Color32::from_rgb(255, 200, 100), // Orange
            None => Theme::TEXT_SECONDARY,
        };

        // Add icon for detected types
        if let Some(det) = detected {
            ui.label(RichText::new(det.icon()).color(color).size(10.0));
        }

        ui.label(RichText::new(&truncated).color(color).size(12.0))
    };

    // Smart tooltip
    response.on_hover_ui(|ui| {
        value_tooltip(ui, value, current_col, col_info, detected, attr_decode.as_deref());
    });
}

/// Render a smart cell with edit capability (double-click to edit in edit mode)
#[allow(clippy::too_many_arguments)]
fn smart_cell_editable(
    ui: &mut egui::Ui,
    value: &str,
    col_info: Option<&ColumnInfo>,
    table_name: &str,
    col_name: &str,
    row_idx: usize,
    col_idx: usize,
    is_pk: bool,
    is_modified: bool,
    app: &mut MsiExplorerApp,
) {
    // Format number values according to current format
    let display_value = if let Some(info) = col_info {
        if info.value_type == ValueType::Integer {
            if let Ok(num) = value.parse::<i32>() {
                app.number_format.format_i32(num)
            } else {
                value.to_string()
            }
        } else {
            value.to_string()
        }
    } else {
        value.to_string()
    };

    let truncated = if display_value.len() > 50 {
        format!("{}...", &display_value[..47])
    } else {
        display_value.clone()
    };

    // Detect value type
    let detected = schema::detect_value_type(value);

    // Get attribute decoding if applicable
    let attr_decode = if let Some(info) = col_info {
        if info.value_type == ValueType::Integer {
            if let Ok(num) = value.parse::<i32>() {
                schema::decode_attributes(table_name, col_name, num)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Check if this is a foreign key we can navigate to
    let fk_target = col_info.and_then(|info| info.foreign_key);

    // Determine color based on state
    let base_color = if is_modified {
        Theme::WARNING // Yellow for modified cells
    } else {
        match detected {
            Some(DetectedValue::Guid) => Theme::TEXT_SECONDARY,
            Some(DetectedValue::PropertyRef) => egui::Color32::from_rgb(180, 140, 255),
            Some(DetectedValue::Path) => egui::Color32::from_rgb(140, 200, 140),
            Some(DetectedValue::Version) => egui::Color32::from_rgb(255, 200, 100),
            None => Theme::TEXT_SECONDARY,
        }
    };

    // Render cell
    let response = if let Some((target_table, _)) = fk_target {
        // Clickable FK link
        let mut text = RichText::new(&truncated).color(Theme::ACCENT).size(12.0).underline();
        if is_modified {
            text = text.color(Theme::WARNING);
        }
        let link = ui.add(egui::Label::new(text).sense(egui::Sense::click()));

        if link.clicked() && !app.edit_mode {
            app.select_table(target_table);
        }

        link
    } else {
        // Add icon for detected types
        if let Some(det) = detected {
            ui.label(RichText::new(det.icon()).color(base_color).size(10.0));
        }

        // Modified indicator
        if is_modified {
            ui.label(RichText::new("*").color(Theme::WARNING).size(10.0));
        }

        ui.add(egui::Label::new(RichText::new(&truncated).color(base_color).size(12.0))
            .sense(egui::Sense::click()))
    };

    // Double-click to edit in edit mode
    if app.edit_mode && response.double_clicked() {
        app.start_edit(table_name, row_idx, col_idx, col_name, value, is_pk);
    }

    // Right-click context menu in edit mode
    if app.edit_mode {
        response.context_menu(|ui| {
            let is_deleted = app.is_row_deleted(table_name, row_idx);
            if is_deleted {
                if ui.button("↩ Restore Row").clicked() {
                    app.undelete_row(table_name, row_idx);
                    ui.close_menu();
                }
            } else {
                if ui.button("✕ Delete Row").clicked() {
                    app.delete_row(table_name, row_idx);
                    ui.close_menu();
                }
            }
            ui.separator();
            if ui.button("✎ Edit Cell").clicked() {
                app.start_edit(table_name, row_idx, col_idx, col_name, value, is_pk);
                ui.close_menu();
            }
        });
    }

    // Smart tooltip (show edit hint in edit mode)
    response.on_hover_ui(|ui| {
        if app.edit_mode {
            ui.label(RichText::new("Double-click to edit")
                .color(Theme::ACCENT)
                .size(11.0)
                .italics());
            if is_pk {
                ui.label(RichText::new("Primary key - will show cascade rename dialog")
                    .color(Theme::WARNING)
                    .size(10.0));
            }
            ui.add_space(8.0);
        }
        value_tooltip(ui, value, col_name, col_info, detected, attr_decode.as_deref());
    });
}

/// Column header tooltip
fn column_tooltip(ui: &mut egui::Ui, name: &str, info: &ColumnInfo) {
    ui.set_max_width(300.0);

    egui::Frame::none()
        .fill(Theme::BG_DARK)
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.label(RichText::new(name)
                .color(Theme::TEXT_PRIMARY)
                .size(14.0)
                .strong());

            ui.add_space(8.0);

            ui.label(RichText::new(info.description)
                .color(Theme::TEXT_SECONDARY)
                .size(12.0));

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Type:")
                    .color(Theme::TEXT_MUTED)
                    .size(11.0));
                ui.label(RichText::new(info.value_type.description())
                    .color(Theme::TEXT_SECONDARY)
                    .size(11.0));
            });

            if let Some((table, col)) = info.foreign_key {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("References:")
                        .color(Theme::TEXT_MUTED)
                        .size(11.0));
                    ui.label(RichText::new(format!("{}.{}", table, col))
                        .color(Theme::ACCENT)
                        .size(11.0));
                });
                ui.label(RichText::new("Click value to navigate")
                    .color(Theme::TEXT_MUTED)
                    .size(10.0)
                    .italics());
            }
        });
}

/// Cell value tooltip
fn value_tooltip(ui: &mut egui::Ui, value: &str, col_name: &str, col_info: Option<&ColumnInfo>, detected: Option<DetectedValue>, attr_decode: Option<&str>) {
    ui.set_max_width(400.0);

    egui::Frame::none()
        .fill(Theme::BG_DARK)
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            // Column name and description first
            if let Some(info) = col_info {
                ui.label(RichText::new(col_name)
                    .color(Theme::ACCENT)
                    .size(13.0)
                    .strong());
                ui.add_space(4.0);
                ui.label(RichText::new(info.description)
                    .color(Theme::TEXT_SECONDARY)
                    .size(12.0));
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
            }

            // Full value (in case it was truncated)
            ui.label(RichText::new("Value")
                .color(Theme::TEXT_MUTED)
                .size(10.0));

            ui.add_space(2.0);

            // Show full value in a selectable text area
            let display_value = if value.len() > 200 {
                format!("{}...", &value[..200])
            } else {
                value.to_string()
            };

            egui::Frame::none()
                .fill(Theme::BG_LIGHT)
                .rounding(Rounding::same(4.0))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(&display_value)
                        .color(Theme::TEXT_PRIMARY)
                        .size(11.0)
                        .monospace());
                });

            // Attribute decoding (if available)
            if let Some(decoded) = attr_decode {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("▤").color(Theme::WARNING).size(12.0));
                    ui.label(RichText::new("Decoded flags:")
                        .color(Theme::TEXT_MUTED)
                        .size(10.0));
                });
                egui::Frame::none()
                    .fill(Theme::BG_LIGHT)
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(decoded)
                            .color(Theme::WARNING)
                            .size(11.0));
                    });
            }

            // Detected type info
            if let Some(det) = detected {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(det.icon()).color(Theme::ACCENT).size(12.0));
                    ui.label(RichText::new(det.description())
                        .color(Theme::TEXT_SECONDARY)
                        .size(11.0));
                });
            }

            // Property reference expansion
            if let Some(DetectedValue::PropertyRef) = detected {
                ui.add_space(4.0);
                // Extract property names
                let props: Vec<&str> = value.split('[')
                    .filter_map(|s| s.split(']').next())
                    .filter(|s| !s.is_empty())
                    .collect();

                for prop in props {
                    if let Some(desc) = schema::get_property_description(prop) {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("[{}]", prop))
                                .color(egui::Color32::from_rgb(180, 140, 255))
                                .size(11.0));
                            ui.label(RichText::new("→")
                                .color(Theme::TEXT_MUTED)
                                .size(11.0));
                            ui.label(RichText::new(desc)
                                .color(Theme::TEXT_SECONDARY)
                                .size(11.0));
                        });
                    }
                }
            }

            // Directory description
            if let Some(desc) = schema::get_directory_description(value) {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("◨").color(Theme::ACCENT).size(12.0));
                    ui.label(RichText::new("Standard Directory")
                        .color(Theme::TEXT_MUTED)
                        .size(11.0));
                });
                ui.label(RichText::new(desc)
                    .color(Theme::TEXT_SECONDARY)
                    .size(11.0));
            }

            // Special handling for known column types
            if let Some(info) = col_info {
                match info.value_type {
                    ValueType::Condition if !value.is_empty() => {
                        ui.add_space(8.0);
                        ui.label(RichText::new("Condition Expression")
                            .color(Theme::TEXT_MUTED)
                            .size(10.0));
                        ui.label(RichText::new("Evaluated at install time. True = action runs")
                            .color(Theme::TEXT_SECONDARY)
                            .size(11.0));
                    }
                    _ => {}
                }
            }

            // FK navigation hint
            if let Some(info) = col_info {
                if info.foreign_key.is_some() {
                    ui.add_space(8.0);
                    ui.label(RichText::new("Click to navigate to referenced table")
                        .color(Theme::ACCENT)
                        .size(10.0)
                        .italics());
                }
            }
        });
}

/// Search results panel
pub fn search_results_panel(ui: &mut egui::Ui, app: &mut MsiExplorerApp) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("⌕").color(Theme::ACCENT).size(18.0));
        ui.label(RichText::new("Search Results")
            .size(18.0)
            .color(Theme::TEXT_PRIMARY)
            .strong());

        ui.add_space(8.0);

        egui::Frame::none()
            .fill(Theme::BG_LIGHT)
            .rounding(Rounding::same(4.0))
            .inner_margin(egui::Margin::symmetric(8.0, 2.0))
            .show(ui, |ui| {
                ui.label(RichText::new(format!("{} matches", app.search_results.len()))
                    .color(Theme::TEXT_SECONDARY)
                    .size(12.0));
            });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(RichText::new("Clear").color(Theme::TEXT_SECONDARY).size(12.0)).clicked() {
                app.search_query.clear();
                app.search_results.clear();
            }
        });
    });

    ui.add_space(12.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for result in &app.search_results.clone() {
            egui::Frame::none()
                .fill(Theme::BG_MEDIUM)
                .rounding(Rounding::same(6.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.link(RichText::new(format!("{}.{}", result.table, result.column))
                            .color(Theme::ACCENT)
                            .size(13.0)).clicked() {
                            app.select_table(&result.table);
                        }

                        ui.label(RichText::new(format!("[{}]", result.primary_key))
                            .color(Theme::TEXT_MUTED)
                            .size(12.0));
                    });

                    ui.add_space(4.0);

                    ui.label(RichText::new(result.highlighted("»", "«"))
                        .color(Theme::TEXT_SECONDARY)
                        .size(12.0));
                });

            ui.add_space(4.0);
        }
    });
}

/// Format file size for display
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_size_i64(bytes: i64) -> String {
    format_size(bytes as u64)
}

/// Tree view panel showing Feature → Component → File hierarchy
pub fn tree_view_panel(ui: &mut egui::Ui, app: &MsiExplorerApp) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("◬").color(Theme::ACCENT).size(18.0));
        ui.label(RichText::new("Feature / Component / File Tree")
            .size(18.0)
            .color(Theme::TEXT_PRIMARY)
            .strong());
    });

    ui.add_space(8.0);

    if app.tree_data.is_empty() {
        ui.label(RichText::new("No tree data available. The MSI may be missing Feature, Component, or File tables.")
            .color(Theme::TEXT_MUTED));
        return;
    }

    // Stats
    let feature_count = app.tree_data.len();
    let comp_count: usize = app.tree_data.iter().map(|f| f.children.len()).sum();
    let file_count: usize = app.tree_data.iter()
        .flat_map(|f| f.children.iter())
        .map(|c| c.children.len())
        .sum();

    ui.horizontal(|ui| {
        badge(ui, &format!("{} features", feature_count));
        badge(ui, &format!("{} components", comp_count));
        badge(ui, &format!("{} files", file_count));
    });

    ui.add_space(12.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for feature in &app.tree_data {
            render_tree_node(ui, feature, 0);
        }
    });
}

fn render_tree_node(ui: &mut egui::Ui, node: &crate::app::TreeNode, depth: usize) {
    use crate::app::TreeNodeType;

    let indent = depth as f32 * 20.0;

    match &node.node_type {
        TreeNodeType::Feature { title } => {
            let header_text = if let Some(t) = title {
                format!("◆ {} - {}", node.name, t)
            } else {
                format!("◆ {}", node.name)
            };

            egui::CollapsingHeader::new(
                RichText::new(header_text)
                    .color(Theme::ACCENT)
                    .size(13.0)
                    .strong()
            )
            .default_open(true)
            .show(ui, |ui| {
                for child in &node.children {
                    render_tree_node(ui, child, depth + 1);
                }
            });
        }
        TreeNodeType::Component { directory } => {
            let header_text = format!("◎ {}", node.name);

            egui::CollapsingHeader::new(
                RichText::new(header_text)
                    .color(Theme::TEXT_PRIMARY)
                    .size(12.0)
            )
            .default_open(false)
            .show(ui, |ui| {
                if let Some(dir) = directory {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Directory:").color(Theme::TEXT_MUTED).size(11.0));
                        ui.label(RichText::new(dir).color(Theme::TEXT_SECONDARY).size(11.0));
                    });
                }
                for child in &node.children {
                    render_tree_node(ui, child, depth + 1);
                }
            });
        }
        TreeNodeType::File { size, version } => {
            ui.horizontal(|ui| {
                ui.add_space(indent);
                ui.label(RichText::new("◨").color(Theme::TEXT_MUTED).size(11.0));
                ui.label(RichText::new(&node.name).color(Theme::TEXT_SECONDARY).size(12.0));

                if let Some(s) = size {
                    ui.label(RichText::new(format_size_i64(*s))
                        .color(Theme::TEXT_MUTED)
                        .size(10.0));
                }
                if let Some(v) = version {
                    if !v.is_empty() {
                        ui.label(RichText::new(format!("v{}", v))
                            .color(egui::Color32::from_rgb(255, 200, 100))
                            .size(10.0));
                    }
                }
            });
        }
        TreeNodeType::Directory { path } => {
            ui.horizontal(|ui| {
                ui.add_space(indent);
                ui.label(RichText::new("◫").color(Theme::TEXT_MUTED).size(11.0));
                ui.label(RichText::new(path).color(Theme::TEXT_SECONDARY).size(12.0));
            });
        }
    }
}

fn badge(ui: &mut egui::Ui, text: &str) {
    egui::Frame::none()
        .fill(Theme::BG_LIGHT)
        .rounding(Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(8.0, 2.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text)
                .color(Theme::TEXT_SECONDARY)
                .size(11.0));
        });
}

/// Files panel showing all files with extraction info
pub fn files_panel(ui: &mut egui::Ui, app: &MsiExplorerApp) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("◨").color(Theme::ACCENT).size(18.0));
        ui.label(RichText::new("Files")
            .size(18.0)
            .color(Theme::TEXT_PRIMARY)
            .strong());

        ui.add_space(8.0);

        badge(ui, &format!("{} files", app.files_list.len()));

        let total_size: i64 = app.files_list.iter().map(|f| f.size).sum();
        badge(ui, &format!("{} total", format_size_i64(total_size)));
    });

    ui.add_space(12.0);

    if app.files_list.is_empty() {
        ui.label(RichText::new("No files found in this MSI.")
            .color(Theme::TEXT_MUTED));
        return;
    }

    // Table view of files
    egui::Frame::none()
        .fill(Theme::BG_MEDIUM)
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::same(1.0))
        .show(ui, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(egui_extras::Column::auto().at_least(200.0).resizable(true)) // Filename
                    .column(egui_extras::Column::auto().at_least(150.0).resizable(true)) // Directory
                    .column(egui_extras::Column::auto().at_least(80.0))  // Size
                    .column(egui_extras::Column::auto().at_least(80.0))  // Version
                    .column(egui_extras::Column::auto().at_least(100.0)) // Cabinet
                    .header(28.0, |mut header| {
                        header.col(|ui| {
                            ui.label(RichText::new("Filename").color(Theme::TEXT_PRIMARY).size(12.0).strong());
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("Directory").color(Theme::TEXT_PRIMARY).size(12.0).strong());
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("Size").color(Theme::TEXT_PRIMARY).size(12.0).strong());
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("Version").color(Theme::TEXT_PRIMARY).size(12.0).strong());
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("Cabinet").color(Theme::TEXT_PRIMARY).size(12.0).strong());
                        });
                    })
                    .body(|body| {
                        body.rows(24.0, app.files_list.len(), |mut row| {
                            let idx = row.index();
                            if let Some(file) = app.files_list.get(idx) {
                                row.col(|ui| {
                                    ui.label(RichText::new(&file.file_name)
                                        .color(Theme::TEXT_PRIMARY)
                                        .size(12.0));
                                });
                                row.col(|ui| {
                                    let response = ui.label(RichText::new(&file.directory)
                                        .color(Theme::ACCENT)
                                        .size(12.0));
                                    if let Some(desc) = schema::get_directory_description(&file.directory) {
                                        response.on_hover_text(desc);
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(RichText::new(format_size_i64(file.size))
                                        .color(Theme::TEXT_SECONDARY)
                                        .size(12.0));
                                });
                                row.col(|ui| {
                                    if let Some(ref v) = file.version {
                                        ui.label(RichText::new(v)
                                            .color(egui::Color32::from_rgb(255, 200, 100))
                                            .size(12.0));
                                    }
                                });
                                row.col(|ui| {
                                    if let Some(ref cab) = file.cab_name {
                                        let is_embedded = cab.starts_with('#');
                                        let display = if is_embedded {
                                            format!("{} (embedded)", &cab[1..])
                                        } else {
                                            cab.clone()
                                        };
                                        ui.label(RichText::new(display)
                                            .color(Theme::TEXT_MUTED)
                                            .size(12.0));
                                    }
                                });
                            }
                        });
                    });
            });
        });
}

/// Diff panel showing differences between two MSIs
pub fn diff_panel(ui: &mut egui::Ui, app: &mut MsiExplorerApp) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("◇").color(Theme::ACCENT).size(18.0));
        ui.label(RichText::new("MSI Comparison")
            .size(18.0)
            .color(Theme::TEXT_PRIMARY)
            .strong());
    });

    ui.add_space(8.0);

    // Show file names
    ui.horizontal(|ui| {
        if let Some(ref path1) = app.current_file {
            egui::Frame::none()
                .fill(Theme::BG_MEDIUM)
                .rounding(Rounding::same(4.0))
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Base:")
                        .color(Theme::TEXT_MUTED)
                        .size(11.0));
                    ui.label(RichText::new(path1.file_name().unwrap_or_default().to_string_lossy())
                        .color(Theme::TEXT_PRIMARY)
                        .size(12.0));
                });
        }

        ui.label(RichText::new("vs").color(Theme::TEXT_MUTED));

        if let Some(ref path2) = app.diff_file {
            egui::Frame::none()
                .fill(Theme::BG_MEDIUM)
                .rounding(Rounding::same(4.0))
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Compare:")
                        .color(Theme::TEXT_MUTED)
                        .size(11.0));
                    ui.label(RichText::new(path2.file_name().unwrap_or_default().to_string_lossy())
                        .color(Theme::TEXT_PRIMARY)
                        .size(12.0));
                });
        }
    });

    ui.add_space(16.0);

    // Perform diff
    if let (Some(ref mut msi1), Some(ref mut msi2)) = (&mut app.msi, &mut app.diff_msi) {
        match msi_explorer::diff::compare(msi1, msi2) {
            Ok(result) => {
                if !result.has_differences() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.label(RichText::new("✓").color(Theme::SUCCESS).size(48.0));
                        ui.label(RichText::new("No differences found")
                            .color(Theme::SUCCESS)
                            .size(16.0));
                    });
                    return;
                }

                // Summary
                ui.horizontal(|ui| {
                    badge(ui, &format!("{} changes", result.change_count()));
                    if !result.tables_only_in_first.is_empty() {
                        diff_badge(ui, &format!("-{} tables", result.tables_only_in_first.len()), Theme::ERROR);
                    }
                    if !result.tables_only_in_second.is_empty() {
                        diff_badge(ui, &format!("+{} tables", result.tables_only_in_second.len()), Theme::SUCCESS);
                    }
                });

                ui.add_space(12.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Tables only in first
                    if !result.tables_only_in_first.is_empty() {
                        egui::CollapsingHeader::new(
                            RichText::new(format!("Tables removed ({})", result.tables_only_in_first.len()))
                                .color(Theme::ERROR)
                                .size(13.0)
                        )
                        .default_open(true)
                        .show(ui, |ui| {
                            for table in &result.tables_only_in_first {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("-").color(Theme::ERROR));
                                    ui.label(RichText::new(table).color(Theme::TEXT_SECONDARY));
                                });
                            }
                        });
                    }

                    // Tables only in second
                    if !result.tables_only_in_second.is_empty() {
                        egui::CollapsingHeader::new(
                            RichText::new(format!("Tables added ({})", result.tables_only_in_second.len()))
                                .color(Theme::SUCCESS)
                                .size(13.0)
                        )
                        .default_open(true)
                        .show(ui, |ui| {
                            for table in &result.tables_only_in_second {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("+").color(Theme::SUCCESS));
                                    ui.label(RichText::new(table).color(Theme::TEXT_SECONDARY));
                                });
                            }
                        });
                    }

                    // Table diffs
                    for table_diff in &result.table_diffs {
                        if table_diff.change_count() == 0 {
                            continue;
                        }

                        egui::CollapsingHeader::new(
                            RichText::new(format!("{} ({} changes)",
                                table_diff.table_name,
                                table_diff.change_count()))
                                .color(Theme::TEXT_PRIMARY)
                                .size(13.0)
                        )
                        .default_open(false)
                        .show(ui, |ui| {
                            // Rows added
                            for row in &table_diff.rows_added {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("+").color(Theme::SUCCESS).size(12.0));
                                    ui.label(RichText::new(format!("[{}]", row.primary_key))
                                        .color(Theme::SUCCESS)
                                        .size(12.0));
                                });
                            }

                            // Rows removed
                            for row in &table_diff.rows_removed {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("-").color(Theme::ERROR).size(12.0));
                                    ui.label(RichText::new(format!("[{}]", row.primary_key))
                                        .color(Theme::ERROR)
                                        .size(12.0));
                                });
                            }

                            // Rows modified
                            for row_mod in &table_diff.rows_modified {
                                egui::CollapsingHeader::new(
                                    RichText::new(format!("~ [{}]", row_mod.primary_key))
                                        .color(Theme::WARNING)
                                        .size(12.0)
                                )
                                .default_open(false)
                                .show(ui, |ui| {
                                    for cell in &row_mod.cell_changes {
                                        egui::Frame::none()
                                            .fill(Theme::BG_LIGHT)
                                            .rounding(Rounding::same(4.0))
                                            .inner_margin(egui::Margin::same(8.0))
                                            .show(ui, |ui| {
                                                ui.label(RichText::new(&cell.column)
                                                    .color(Theme::TEXT_MUTED)
                                                    .size(11.0));
                                                ui.horizontal(|ui| {
                                                    ui.label(RichText::new(&cell.old_value)
                                                        .color(Theme::ERROR)
                                                        .size(11.0));
                                                    ui.label(RichText::new("→")
                                                        .color(Theme::TEXT_MUTED));
                                                    ui.label(RichText::new(&cell.new_value)
                                                        .color(Theme::SUCCESS)
                                                        .size(11.0));
                                                });
                                            });
                                    }
                                });
                            }
                        });
                    }

                    // Property diffs
                    if !result.property_diffs.is_empty() {
                        egui::CollapsingHeader::new(
                            RichText::new(format!("Properties ({} changes)", result.property_diffs.len()))
                                .color(Theme::TEXT_PRIMARY)
                                .size(13.0)
                        )
                        .default_open(true)
                        .show(ui, |ui| {
                            for prop in &result.property_diffs {
                                ui.horizontal(|ui| {
                                    match (&prop.old_value, &prop.new_value) {
                                        (None, Some(v)) => {
                                            ui.label(RichText::new("+").color(Theme::SUCCESS));
                                            ui.label(RichText::new(&prop.name).color(Theme::TEXT_PRIMARY));
                                            ui.label(RichText::new("=").color(Theme::TEXT_MUTED));
                                            ui.label(RichText::new(v).color(Theme::SUCCESS));
                                        }
                                        (Some(v), None) => {
                                            ui.label(RichText::new("-").color(Theme::ERROR));
                                            ui.label(RichText::new(&prop.name).color(Theme::TEXT_PRIMARY));
                                            ui.label(RichText::new("=").color(Theme::TEXT_MUTED));
                                            ui.label(RichText::new(v).color(Theme::ERROR));
                                        }
                                        (Some(old), Some(new)) => {
                                            ui.label(RichText::new("~").color(Theme::WARNING));
                                            ui.label(RichText::new(&prop.name).color(Theme::TEXT_PRIMARY));
                                            ui.label(RichText::new(old).color(Theme::ERROR).size(11.0));
                                            ui.label(RichText::new("→").color(Theme::TEXT_MUTED));
                                            ui.label(RichText::new(new).color(Theme::SUCCESS).size(11.0));
                                        }
                                        _ => {}
                                    }
                                });
                            }
                        });
                    }
                });
            }
            Err(e) => {
                ui.label(RichText::new(format!("Diff failed: {}", e))
                    .color(Theme::ERROR));
            }
        }
    } else {
        ui.label(RichText::new("Select a second MSI file to compare")
            .color(Theme::TEXT_MUTED));
    }
}

fn diff_badge(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    egui::Frame::none()
        .fill(color.gamma_multiply(0.2))
        .rounding(Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(8.0, 2.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text)
                .color(color)
                .size(11.0));
        });
}
