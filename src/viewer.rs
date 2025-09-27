use std::fs::File;

use iced::{
    Color, Element, Size,
    advanced::{
        Layout, Widget,
        graphics::{self, geometry},
        layout, renderer, widget,
    },
    border, mouse,
};
// use libreda_db::{self as db, chip::Chip};
use libreda_db::prelude::*;
use libreda_oasis::{
    LayoutStreamReader,
    OASISStreamReader,
};

pub struct Viewer {}

impl Viewer {
    pub fn new(filename: &str) -> Self {
        let mut f = File::open(filename).unwrap();
        let mut layout = Chip::new();
        let mut reader = OASISStreamReader::new();
        let result = reader.read_layout(&mut f, &mut layout);
        // layout.bounding_box(cell)
        // layout.shape_geometry(shape_id)
        Self {}
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Viewer
where
    Renderer: geometry::Renderer,
{
    fn size(&self) -> iced::Size<iced::Length> {
        todo!()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(400.0, 400.0))
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: border::rounded(200.0),
                ..renderer::Quad::default()
            },
            Color::BLACK,
        );
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
