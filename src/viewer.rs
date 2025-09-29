use iced::{
    Color, Element, Length, Size,
    advanced::{
        Layout, Widget,
        graphics::{self, geometry}, layout, renderer, widget,
    },
    border, mouse,
};
use std::sync::{Arc, Mutex};
use libreda_db::prelude::*;

pub struct Viewer {
    filename: Option<String>,
    parsed_chip: Arc<Mutex<Option<Chip>>>,
    parse_error: Arc<Mutex<Option<String>>>,
}

impl Viewer {
    pub fn new(
        filename: Option<String>,
        parsed_chip: Arc<Mutex<Option<Chip>>>,
        parse_error: Arc<Mutex<Option<String>>>,
    ) -> Self {
        Self { filename, parsed_chip, parse_error }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Viewer
where
    Renderer: geometry::Renderer,
{
    fn size(&self) -> iced::Size<iced::Length> {
        // Use shrink so layout() decides the pixel size.
        Size { width: Length::Shrink, height: Length::Shrink }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(400.0, 400.0))
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        // Draw a simple background and a filename text if present.
        _renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: border::rounded(8.0),
                ..renderer::Quad::default()
            },
            Color::from_rgb(0.95, 0.95, 0.95),
        );

        // If a filename was provided, indicate parse status by reading the shared state.
        if self.filename.is_some() {
            let pc = self.parsed_chip.lock().unwrap();
            let pe = self.parse_error.lock().unwrap();

            if pc.is_some() {
                // parsed successfully: slightly green tint
                _renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds(),
                        ..renderer::Quad::default()
                    },
                    Color::from_rgb(0.90, 0.98, 0.90),
                );
            } else if pe.is_some() {
                // parse error: light red tint
                _renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds(),
                        ..renderer::Quad::default()
                    },
                    Color::from_rgb(0.98, 0.90, 0.90),
                );
            }
            // drop locks
            drop(pc);
            drop(pe);
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Viewer> for Element<'a, Message, Theme, Renderer>
where
    Renderer: geometry::Renderer,
{
    fn from(viewer: Viewer) -> Self {
        Self::new(viewer)
    }
}
