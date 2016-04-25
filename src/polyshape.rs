use sfml::graphics::{Color, CustomShape, Drawable, RenderTarget, RenderStates,
                        RenderWindow, ShapeImpl, Transformable};
use polygon::World;
use car::Car;
use view::View;

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
    
    pub fn add_shape(&mut self, shape_impl: Box<ShapeImpl + Send>) {
        let mut cs = CustomShape::new(shape_impl).unwrap();
        cs.set_position(&self.view.pos());
        cs.set_scale(&self.view.scale());

        cs.set_fill_color(&Color::transparent());
        cs.set_outline_color(&Color::black());
        cs.set_outline_thickness(0.2);
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
    fn get_polyshape(&self, view: View) -> Polyshape;
}

impl Polyshapable for World {
    fn get_polyshape(&self, view: View) -> Polyshape {
        let mut ps = Polyshape::new(view);
        for p in self.walls.paths.iter() {
            ps.add_shape(Box::new(p.clone()));
        }
        ps
    }
}

impl Polyshapable for Car {
    fn get_polyshape(&self, view: View) -> Polyshape {
        let mut ps = Polyshape::new(view);
        for p in self.path.paths.iter() {
            ps.add_shape(Box::new(p.clone()));
        }
        ps
    }    
}