use iced::{
    Color, Element, Size,
    advanced::{
        Layout, Widget,
        graphics::{self, geometry},
        layout, renderer, widget,
    },
    border, mouse,
};

pub struct Viewer {}

impl Viewer {
    pub fn new() -> Self {
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
