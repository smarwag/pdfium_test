use std::sync::Mutex;

use pdfium_render::prelude::{PdfDocument, Pdfium};
use tauri::{Manager, State};

#[tauri::command]
fn greet(path: String, state: State<'_, Mutex<AppData>>) -> String {
    let mut state = state.lock().unwrap();
    state.pdf = ActivePdf::load_document(path, &state.pdfium);
    "".to_string()
}

pub struct PdfManager {
    pdfium: Pdfium,
}
pub struct ActivePdf<'a> {
    document: Option<PdfDocument<'a>>,
}
struct AppData<'a> {
    pdfium: PdfManager,
    pdf: ActivePdf<'a>,
}

impl<'a> ActivePdf<'a> {
    pub fn load_document(path: String, lib: &'a PdfManager) -> Self {
        ActivePdf {
            document: Some(lib.pdfium.load_pdf_from_file(&path, None).unwrap()),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(Mutex::new(AppData {
                pdfium: PdfManager {
                    pdfium: Pdfium::new(
                        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("."))
                            .unwrap(),
                    ),
                },
                pdf: ActivePdf {
                    document: Option::None,
                },
            }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
