mod circle {
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Widget};
    use iced::border;
    use iced::mouse;
    use iced::{Color, Element, Length, Rectangle, Size};

    pub struct Circle {
        radius: f32,
    }

    impl Circle {
        pub fn new(radius: f32) -> Self {
            Self { radius }
        }
    }

    pub fn circle(radius: f32) -> Circle {
        Circle::new(radius)
    }

    impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Circle
    where
        Renderer: renderer::Renderer,
    {
        fn size(&self) -> Size<Length> {
            Size {
                width: Length::Shrink,
                height: Length::Shrink,
            }
        }

        fn layout(
            &self,
            _tree: &mut widget::Tree,
            _renderer: &Renderer,
            _limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(Size::new(self.radius * 2.0, self.radius * 2.0))
        }

        fn draw(
            &self,
            _state: &widget::Tree,
            renderer: &mut Renderer,
            _theme: &Theme,
            _style: &renderer::Style,
            layout: Layout<'_>,
            _cursor: mouse::Cursor,
            _viewport: &Rectangle,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: border::rounded(self.radius),
                    ..renderer::Quad::default()
                },
                Color::BLACK,
            );
        }
    }

    impl<'a, Message, Theme, Renderer> From<Circle>
        for Element<'a, Message, Theme, Renderer>
    where
        Renderer: renderer::Renderer,
    {
        fn from(circle: Circle) -> Self {
            Self::new(circle)
        }
    }
}

mod viewer;
use viewer::Viewer;
use iced::widget::{button, row};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use libreda_db::prelude::*;

use circle::circle;
use iced::widget::{center, column, slider, text};
use iced::{Center, Element};

pub fn main() -> iced::Result {
    iced::run("Custom Widget - Iced", App::update, App::view)
}

struct App {
    radius: f32,
    opened: Option<PathBuf>,
    // Shared parsed chip and parse error set by background thread
    parsed_chip: Arc<Mutex<Option<libreda_db::prelude::Chip>>>,
    parse_error: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone)]
enum Message {
    RadiusChanged(f32),
    OpenFile,
    FileChosen(Option<PathBuf>),
}

impl App {
    fn new() -> Self {
        Self {
            radius: 50.0,
            opened: None,
            parsed_chip: Arc::new(Mutex::new(None)),
            parse_error: Arc::new(Mutex::new(None)),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::RadiusChanged(radius) => {
                self.radius = radius;
            }
            Message::OpenFile => {
                // On native builds open a native dialog. On web builds, we can't use rfd;
                // for now set an explanatory parse_error so the UI can show guidance.
                #[cfg(feature = "native")]
                let file = rfd::FileDialog::new()
                    .add_filter("GDS or OASIS", &["gds", "oas"])
                    .pick_file();

                #[cfg(not(feature = "native"))]
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

                #[cfg(not(feature = "native"))]
                {
                    let mut pe = self.parse_error.lock().unwrap();
                    *pe = Some("Use browser file input to open OASIS/GDS on web build".to_string());
                }

                // spawn background thread to parse the chosen file (native only)
                #[cfg(feature = "native")]
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
            }
            Message::FileChosen(opt) => {
                self.opened = opt;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let open_button = button("Open GDS/OASIS").on_press(Message::OpenFile);

        let filename_text = match &self.opened {
            Some(p) => text!("Opened: {}", p.to_string_lossy()),
            None => text!("No file opened"),
        };

        let content = column![
            circle(self.radius),//.width(Length::fill).height(Length::fill),
            row![open_button, filename_text].spacing(10),
            // pass optional filename as &str
            Viewer::new(
                self.opened.as_ref().map(|p| p.to_string_lossy().into_owned()),
                Arc::clone(&self.parsed_chip),
                Arc::clone(&self.parse_error),
            ),
            text!("Radius: {:.2}", self.radius),
            slider(1.0..=100.0, self.radius, Message::RadiusChanged).step(0.01),
        ]
        .padding(20)
        .spacing(20)
        .max_width(500)
        .align_x(Center);

        center(content).into()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}