use iced::{
    Color, Element, Length, Size,
    advanced::{
        Layout, Widget,
        graphics::geometry, layout, renderer, widget,
    },
    border, mouse,
};
use std::sync::{Arc, Mutex};
use libreda_db::prelude::*;
use iced::Rectangle;

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
        limits: &layout::Limits,

    ) -> layout::Node {
        layout::Node::new(limits.max())
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

        // If a filename was provided, indicate parse status by reading the shared state
        // and render simple shapes (Rect, Polygon approximated) from the parsed chip.
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

                // Render shapes from the chip (Rect filled, Polygon approximated by bbox)
                if let Some(chip) = pc.as_ref() {
                    // Compute overall bounding box across cells/layers so we can scale to widget
                    let mut minx: Option<f32> = None;
                    let mut miny: Option<f32> = None;
                    let mut maxx: Option<f32> = None;
                    let mut maxy: Option<f32> = None;

                    for cell in chip.each_cell() {
                        for layer in chip.each_layer() {
                                if let Some(bbox) = chip.bounding_box_per_layer(&cell, &layer) {
                                // bbox.lower_left() / upper_right() return Point-like tuples
                                let ll = bbox.lower_left();
                                let ur = bbox.upper_right();
                                let lx = ll.x as f32;
                                let ly = ll.y as f32;
                                let rx = ur.x as f32;
                                let ry = ur.y as f32;

                                minx = Some(minx.map_or(lx, |v| v.min(lx)));
                                miny = Some(miny.map_or(ly, |v| v.min(ly)));
                                maxx = Some(maxx.map_or(rx, |v| v.max(rx)));
                                maxy = Some(maxy.map_or(ry, |v| v.max(ry)));
                            }
                        }
                    }

                    if let (Some(minx), Some(miny), Some(maxx), Some(maxy)) = (minx, miny, maxx, maxy) {
                        let world_w = (maxx - minx).max(1.0);
                        let world_h = (maxy - miny).max(1.0);
                        let bounds = layout.bounds();
                        // leave a small margin
                        let margin = 8.0;
                        let avail_w = (bounds.width - 2.0 * margin).max(1.0);
                        let avail_h = (bounds.height - 2.0 * margin).max(1.0);
                        let scale = (avail_w / world_w).min(avail_h / world_h);

                        // helper to map world coord -> pixel
                        let map_x = |x: f32| bounds.x + margin + (x - minx) * scale;
                        // invert y so that larger world y goes up on screen coordinates
                        let map_y = |y: f32| bounds.y + bounds.height - margin - (y - miny) * scale;

                        // Iterate shapes and draw rects and polygons (approx with bbox)
                        for cell in chip.each_cell() {
                            for layer in chip.each_layer() {
                                chip.for_each_shape(&cell, &layer, |_, g| {
                                    match g {
                                        Geometry::Rect(r) => {
                                            let ll = r.lower_left();
                                            let ur = r.upper_right();
                                            let x0 = map_x(ll.x as f32);
                                            let y1 = map_y(ll.y as f32);
                                            let x1 = map_x(ur.x as f32);
                                            let y0 = map_y(ur.y as f32);
                                            let rect = Rectangle { x: x0, y: y0, width: (x1 - x0).abs(), height: (y1 - y0).abs() };
                                            _renderer.fill_quad(
                                                renderer::Quad { bounds: rect, ..renderer::Quad::default() },
                                                Color::from_rgb(0.2, 0.4, 0.9),
                                            );
                                        }
                                        Geometry::Polygon(p) => {
                                            // Use a canvas Frame to build precise polygon geometry and draw it.
                                            let ext = p.exterior.points();
                                            if ext.len() >= 3 {
                                                // Build a Path in widget coordinates
                                                let path = geometry::Path::new(|b| {
                                                    let v0 = ext[0];
                                                    b.move_to(iced::Point::new(map_x(v0.x as f32), map_y(v0.y as f32)));
                                                    for v in ext.iter().skip(1) {
                                                        b.line_to(iced::Point::new(map_x(v.x as f32), map_y(v.y as f32)));
                                                    }
                                                    b.close();
                                                });

                                                // Create a canvas frame, draw into it, then convert to geometry
                                                // Create a geometry frame via the geometry module and draw it
                                                // using the renderer's draw_geometry API.
                                                let mut frame = geometry::Frame::new(_renderer, layout.bounds().size());
                                                frame.fill(&path, Color::from_rgba(0.2, 0.9, 0.4, 0.9));
                                                let geom = frame.into_geometry();

                                                // Draw the geometry using the renderer's draw_geometry method.
                                                _renderer.draw_geometry(geom);
                                            }
                                        }
                                        _ => {}
                                    }
                                });
                            }
                        }
                    }
                }
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
