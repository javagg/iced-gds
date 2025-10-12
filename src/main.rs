mod viewer;
use iced::widget::{button, row};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use viewer::Viewer;
// no direct import; refer to libreda_db::prelude::Chip by full path where needed
use libreda_oasis::LayoutStreamReader;

use iced::widget::{center, column, text};
use iced::{Center, Element, Length};
// rfd is only used inside non-wasm branches below; import it locally to avoid
// unused import warnings when building for wasm.

pub fn main() -> iced::Result {
    iced::application("Custom Widget - Iced", App::update, App::view)
        .window_size((1200.0, 800.0))
        .run()
}

struct App {
    opened: Option<PathBuf>,
    // Shared parsed chip and parse error set by background thread
    parsed_chip: Arc<Mutex<Option<libreda_db::prelude::Chip>>>,
    parse_error: Arc<Mutex<Option<String>>>,
    // no fallback input; native file dialog is used when built with --features native
    // fallback path input for non-native builds
}

#[derive(Debug, Clone)]
enum Message {
    OpenFile,
}

impl App {
    fn new() -> Self {
        Self {
            opened: None,
            parsed_chip: Arc::new(Mutex::new(None)),
            parse_error: Arc::new(Mutex::new(None)),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::OpenFile => {
                // On native builds open a native dialog. On web builds, we can't use rfd;
                // for now set an explanatory parse_error so the UI can show guidance.
                #[cfg(not(target_arch = "wasm32"))]
                let file = rfd::FileDialog::new()
                    .add_filter("GDS or OASIS", &["gds", "oas"])
                    .pick_file();

                #[cfg(target_arch = "wasm32")]
                let file: Option<std::path::PathBuf> = None;

                // send chosen file back into update via message
                let chosen = file.map(|p| p.into());
                // directly assign since we are already in update
                self.opened = chosen.clone();

                // reset previous parse state
                {
                    let mut pc = self.parsed_chip.lock().unwrap();
                    *pc = None;
                }
                {
                    let mut pe = self.parse_error.lock().unwrap();
                    *pe = None;
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let mut pe = self.parse_error.lock().unwrap();
                    *pe = Some("Use browser file input to open OASIS/GDS on web build".to_string());
                }

                // spawn background thread to parse the chosen file (native only)
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = chosen {
                        let parsed_chip = Arc::clone(&self.parsed_chip);
                        let parse_error = Arc::clone(&self.parse_error);
                        std::thread::spawn(move || {
                            // parse in background
                            match std::fs::File::open(&path) {
                                Ok(mut f) => {
                                    let mut layout = libreda_db::prelude::Chip::new();
                                    let reader = libreda_oasis::OASISStreamReader::new();
                                //    let reader = libreda_db::GDSStreamReader::new();
                                    match reader.read_layout(&mut f, &mut layout) {
                                        Ok(_) => {
                                            let mut pc = parsed_chip.lock().unwrap();
                                            *pc = Some(layout);
                                        }
                                        Err(e) => {
                                            let mut pe = parse_error.lock().unwrap();
                                            *pe = Some(format!("parse error: {:?}", e));
                                        }
                                    }
                                }
                                Err(e) => {
                                    let mut pe = parse_error.lock().unwrap();
                                    *pe = Some(format!("open error: {}", e));
                                }
                            }
                        });
                    }
                }
                // For non-native builds, chosen is None. Do nothing here.
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let open_button = button("Open GDS/OASIS").on_press(Message::OpenFile);

        let filename_text = match &self.opened {
            Some(p) => text!("Opened: {}", p.to_string_lossy()),
            None => text!("No file opened"),
        };

        let viewer = Viewer::new(
            self.opened
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            Arc::clone(&self.parsed_chip),
            Arc::clone(&self.parse_error),
        );
        let content = column![row![open_button, filename_text].spacing(10),]
            .extend([row![center(viewer)].into()])
            .padding(10)
            .spacing(10)
            .align_x(Center);

        center(content).into()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
