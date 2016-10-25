use sfml::graphics::{Color, CustomShape, Drawable, RenderTarget, RenderStates,
                        RenderWindow, ShapeImpl, Transformable};
use polygon::World;
use car::Car;
use view::View;

pub struct PolyshapeStyle {
    pub fill_color: Color,
    pub outline_color: Color,
    pub outline_thickness: f32,
}

impl PolyshapeStyle {
    pub fn new() -> PolyshapeStyle {
        PolyshapeStyle {
            fill_color: Color::transparent(),
            outline_color: Color::black(),
            outline_thickness: 0.2
        }
    }

    pub fn set_fill_color(&mut self, c: Color) -> &mut PolyshapeStyle {
        self.fill_color = c;
        self
    }

    pub fn set_outline_color(&mut self, c: Color) -> &mut PolyshapeStyle {
        self.outline_color = c;
        self
    }

    pub fn set_outline_thickness(&mut self, t: f32) -> &mut PolyshapeStyle {
        self.outline_thickness = t;
        self
    }
}

pub struct Polyshape<'s> {
    pub shapes: Vec<CustomShape<'s>>,
    pub view: View
}

impl<'s> Polyshape<'s> {
    pub fn new(view: View) -> Polyshape<'s> {
        Polyshape {
            shapes: Vec::new(),
            view: view
        }
    }

    pub fn add_shape(&mut self, shape_impl: Box<ShapeImpl + Send>, style: &PolyshapeStyle) {
        let mut cs = CustomShape::new(shape_impl).unwrap();
        cs.set_position(&self.view.pos());
        cs.set_scale(&self.view.scale());

        cs.set_fill_color(&style.fill_color);
        cs.set_outline_color(&style.outline_color);
        cs.set_outline_thickness(style.outline_thickness);
        self.shapes.push(cs);
    }
}

impl<'s> Drawable for Polyshape<'s> {
    fn draw<RT: RenderTarget>(&self, target: &mut RT, states: &mut RenderStates) {
        for s in self.shapes.iter() {
            s.draw(target, states);
        }
    }
}

pub trait Polyshapable {
    fn get_polyshape(&self, view: View, pss: &PolyshapeStyle) -> Polyshape;
}

impl Polyshapable for World {
    fn get_polyshape(&self, view: View, pss: &PolyshapeStyle) -> Polyshape {
        let mut ps = Polyshape::new(view);
        for p in self.walls.paths.iter() {
            ps.add_shape(Box::new(p.clone()), pss);
        }
        ps
    }
}

impl Polyshapable for Car {
    fn get_polyshape(&self, view: View, pss: &PolyshapeStyle) -> Polyshape {
        let mut ps = Polyshape::new(view);
        for p in self.path.paths.iter() {
            ps.add_shape(Box::new(p.clone()), pss);
                /*
                &PolyshapeStyle::new()
                    .set_outline_color(Color::red())
                    .set_fill_color(Color::red()));
                */
        }
        ps
    }
}