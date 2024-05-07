// gtk4::Dialog is deprecated, but the replacement adw::ToolbarView is not suitable for a async flow
#![allow(deprecated)]

// Imports
use crate::canvas::RnCanvas;
use crate::{config, RnAppWindow};
use adw::prelude::*;
use gettextrs::gettext;
use gtk4::{
    gio, glib, glib::clone, Builder, Dialog, FileDialog, FileFilter, Label, ResponseType,
    ToggleButton,
};
use num_traits::ToPrimitive;
use rnote_engine::engine::import::{PdfImportPageSpacing, PdfImportPagesType};

/// Opens a new rnote save file in a new tab
pub(crate) async fn filedialog_open_doc(appwindow: &RnAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/rnote");
    filter.add_suffix("rnote");
    filter.set_name(Some(&gettext(".rnote")));

    let filter_list = gio::ListStore::new::<FileFilter>();
    filter_list.append(&filter);

    let filedialog = FileDialog::builder()
        .title(gettext("Open File"))
        .modal(true)
        .accept_label(gettext("Open"))
        .filters(&filter_list)
        .default_filter(&filter)
        .build();

    if let Some(current_workspace_dir) = appwindow.sidebar().workspacebrowser().dir_list_dir() {
        filedialog.set_initial_folder(Some(&gio::File::for_path(current_workspace_dir)));
    }

    match filedialog.open_future(Some(appwindow)).await {
        Ok(selected_file) => {
            appwindow
                .open_file_w_dialogs(selected_file, None, true)
                .await;
        }
        Err(e) => {
            tracing::debug!(
                "Did not open document (Error or dialog dismissed by user), Err: {e:?}"
            );
        }
    }
}

pub(crate) async fn filedialog_import_file(appwindow: &RnAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/x-xopp");
    filter.add_mime_type("application/pdf");
    filter.add_mime_type("image/svg+xml");
    filter.add_mime_type("image/png");
    filter.add_mime_type("image/jpeg");
    filter.add_mime_type("text/plain");
    filter.add_suffix("xopp");
    filter.add_suffix("pdf");
    filter.add_suffix("svg");
    filter.add_suffix("png");
    filter.add_suffix("jpg");
    filter.add_suffix("jpeg");
    filter.add_suffix("txt");
    filter.set_name(Some(&gettext("Jpg, Pdf, Png, Svg, Xopp, Txt")));

    let filter_list = gio::ListStore::new::<FileFilter>();
    filter_list.append(&filter);

    let dialog = FileDialog::builder()
        .title(gettext("Import File"))
        .modal(true)
        .accept_label(gettext("Import"))
        .filters(&filter_list)
        .default_filter(&filter)
        .build();

    if let Some(current_workspace_dir) = appwindow.sidebar().workspacebrowser().dir_list_dir() {
        dialog.set_initial_folder(Some(&gio::File::for_path(current_workspace_dir)));
    }

    match dialog.open_future(Some(appwindow)).await {
        Ok(selected_file) => {
            appwindow
                .open_file_w_dialogs(selected_file, None, true)
                .await;
        }
        Err(e) => {
            tracing::debug!("Did not import file (Error or dialog dismissed by user), Err: {e:?}");
        }
    }
}

/// Imports the file as Pdf with an import dialog.
///
/// Returns true when the file was imported, else false.
pub(crate) async fn dialog_import_pdf_w_prefs(
    appwindow: &RnAppWindow,
    canvas: &RnCanvas,
    input_file: gio::File,
    target_pos: Option<na::Vector2<f64>>,
) -> anyhow::Result<bool> {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/import.ui").as_str(),
    );
    let dialog: Dialog = builder.object("dialog_import_pdf_w_prefs").unwrap();
    let pdf_page_start_row: adw::SpinRow = builder.object("pdf_page_start_row").unwrap();
    let pdf_page_end_row: adw::SpinRow = builder.object("pdf_page_end_row").unwrap();
    let pdf_info_label: Label = builder.object("pdf_info_label").unwrap();
    let pdf_import_width_row: adw::SpinRow = builder.object("pdf_import_width_row").unwrap();
    let pdf_import_page_spacing_row: adw::ComboRow =
        builder.object("pdf_import_page_spacing_row").unwrap();
    let pdf_import_as_bitmap_toggle: ToggleButton =
        builder.object("pdf_import_as_bitmap_toggle").unwrap();
    let pdf_import_as_vector_toggle: ToggleButton =
        builder.object("pdf_import_as_vector_toggle").unwrap();
    let pdf_import_bitmap_scalefactor_row: adw::SpinRow =
        builder.object("pdf_import_bitmap_scalefactor_row").unwrap();
    let pdf_import_page_borders_row: adw::SwitchRow =
        builder.object("pdf_import_page_borders_row").unwrap();
    let pdf_import_adjust_document_row: adw::SwitchRow =
        builder.object("pdf_import_adjust_document_row").unwrap();

    dialog.set_transient_for(Some(appwindow));

    pdf_import_adjust_document_row
        .bind_property("active", &pdf_import_width_row, "sensitive")
        .invert_boolean()
        .sync_create()
        .build();
    pdf_import_adjust_document_row
        .bind_property("active", &pdf_import_page_spacing_row, "sensitive")
        .invert_boolean()
        .sync_create()
        .build();

    let pdf_import_prefs = canvas.engine_ref().import_prefs.pdf_import_prefs;

    // Set the widget state from the pdf import prefs
    pdf_import_width_row.set_value(pdf_import_prefs.page_width_perc);
    match pdf_import_prefs.pages_type {
        PdfImportPagesType::Bitmap => {
            pdf_import_as_bitmap_toggle.set_active(true);
            pdf_import_bitmap_scalefactor_row.set_sensitive(true);
        }
        PdfImportPagesType::Vector => {
            pdf_import_as_vector_toggle.set_active(true);
            pdf_import_bitmap_scalefactor_row.set_sensitive(false);
        }
    }
    pdf_import_page_spacing_row.set_selected(pdf_import_prefs.page_spacing.to_u32().unwrap());
    pdf_import_bitmap_scalefactor_row.set_value(pdf_import_prefs.bitmap_scalefactor);
    pdf_import_page_borders_row.set_active(pdf_import_prefs.page_borders);
    pdf_import_adjust_document_row.set_active(pdf_import_prefs.adjust_document);

    pdf_page_start_row
        .bind_property("value", &pdf_page_end_row.adjustment(), "lower")
        .sync_create()
        .build();
    pdf_page_end_row
        .bind_property("value", &pdf_page_start_row.adjustment(), "upper")
        .sync_create()
        .build();

    // Update preferences
    pdf_import_as_vector_toggle.connect_toggled(
        clone!(@weak pdf_import_bitmap_scalefactor_row, @weak canvas, @weak appwindow => move |toggle| {
            if toggle.is_active() {
                canvas.engine_mut().import_prefs.pdf_import_prefs.pages_type = PdfImportPagesType::Vector;
                pdf_import_bitmap_scalefactor_row.set_sensitive(false);
            }
        }),
    );

    pdf_import_as_bitmap_toggle.connect_toggled(
        clone!(@weak pdf_import_bitmap_scalefactor_row, @weak canvas, @weak appwindow => move |toggle| {
            if toggle.is_active() {
                canvas.engine_mut().import_prefs.pdf_import_prefs.pages_type = PdfImportPagesType::Bitmap;
                pdf_import_bitmap_scalefactor_row.set_sensitive(true);
            }
        }),
    );

    pdf_import_bitmap_scalefactor_row.connect_changed(
        clone!(@weak canvas, @weak appwindow => move |row| {
            canvas.engine_mut().import_prefs.pdf_import_prefs.bitmap_scalefactor = row.value();
        }),
    );

    pdf_import_page_spacing_row.connect_selected_notify(
        clone!(@weak canvas, @weak appwindow => move |row| {
            let page_spacing = PdfImportPageSpacing::try_from(row.selected()).unwrap();

            canvas.engine_mut().import_prefs.pdf_import_prefs.page_spacing = page_spacing;
        }),
    );

    pdf_import_width_row.connect_changed(clone!(@weak canvas, @weak appwindow => move |row| {
            canvas.engine_mut().import_prefs.pdf_import_prefs.page_width_perc = row.value();
    }));

    pdf_import_page_borders_row.connect_active_notify(
        clone!(@weak canvas, @weak appwindow => move |row| {
            canvas.engine_mut().import_prefs.pdf_import_prefs.page_borders = row.is_active();
        }),
    );

    pdf_import_adjust_document_row.connect_active_notify(
        clone!(@weak canvas, @weak appwindow => move |row| {
            canvas.engine_mut().import_prefs.pdf_import_prefs.adjust_document = row.is_active();
        }),
    );

    if let Ok(poppler_doc) =
        poppler::Document::from_gfile(&input_file, None, None::<&gio::Cancellable>)
    {
        let file_name = input_file.basename().map_or_else(
            || gettext("- no file name -"),
            |s| s.to_string_lossy().to_string(),
        );
        let title = poppler_doc
            .title()
            .map_or_else(|| gettext("- no title -"), |s| s.to_string());
        let author = poppler_doc
            .author()
            .map_or_else(|| gettext("- no author -"), |s| s.to_string());
        let mod_date = poppler_doc
            .mod_datetime()
            .and_then(|dt| dt.format("%F").ok())
            .map_or_else(|| gettext("- no date -"), |s| s.to_string());
        let n_pages = poppler_doc.n_pages();

        // pdf info
        pdf_info_label.set_label(
            (String::from("")
                + "<b>"
                + &gettext("File name:")
                + "  </b>"
                + &format!("{file_name}\n")
                + "<b>"
                + &gettext("Title:")
                + "  </b>"
                + &format!("{title}\n")
                + "<b>"
                + &gettext("Author:")
                + "  </b>"
                + &format!("{author}\n")
                + "<b>"
                + &gettext("Modification date:")
                + "  </b>"
                + &format!("{mod_date}\n")
                + "<b>"
                + &gettext("Pages:")
                + "  </b>"
                + &format!("{n_pages}\n"))
                .as_str(),
        );

        // Configure pages spinners
        pdf_page_start_row.set_range(1.into(), n_pages.into());
        pdf_page_start_row.set_value(1.into());

        pdf_page_end_row.set_range(1.into(), n_pages.into());
        pdf_page_end_row.set_value(n_pages.into());
    }

    let response = dialog.run_future().await;
    dialog.close();
    match response {
        ResponseType::Apply => {
            let page_range =
                (pdf_page_start_row.value() as u32 - 1)..pdf_page_end_row.value() as u32;
            let (bytes, _) = input_file.load_bytes_future().await?;
            canvas
                .load_in_pdf_bytes(bytes.to_vec(), target_pos, Some(page_range))
                .await?;
            Ok(true)
        }
        _ => {
            // Cancel
            Ok(false)
        }
    }
}

/// Imports the file as Xopp with an import dialog.
///
/// Returns true when the file was imported, else false.
pub(crate) async fn dialog_import_xopp_w_prefs(
    appwindow: &RnAppWindow,
    canvas: &RnCanvas,
    input_file: gio::File,
) -> anyhow::Result<bool> {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/import.ui").as_str(),
    );
    let dialog: Dialog = builder.object("dialog_import_xopp_w_prefs").unwrap();
    let dpi_row: adw::SpinRow = builder.object("xopp_import_dpi_row").unwrap();
    let xopp_import_prefs = canvas.engine_ref().import_prefs.xopp_import_prefs;

    dialog.set_transient_for(Some(appwindow));

    // Set initial widget state for preference
    dpi_row.set_value(xopp_import_prefs.dpi);

    // Update preferences
    dpi_row.connect_changed(clone!(@weak canvas, @weak appwindow => move |row| {
        canvas.engine_mut().import_prefs.xopp_import_prefs.dpi = row.value();
    }));

    let response = dialog.run_future().await;
    dialog.close();
    match response {
        ResponseType::Apply => {
            let (bytes, _) = input_file.load_bytes_future().await?;
            canvas.load_in_xopp_bytes(bytes.to_vec()).await?;
            Ok(true)
        }
        _ => {
            // Cancel
            Ok(false)
        }
    }
}
