use quicksilver::geom::Vector;

pub struct LineT {
    a: Vector,
    b: Vector,
    t: f32,
}

impl LineT {
    pub fn new(start: Vector, end: Vector) -> LineT {
        Self {
            a: start,
            b: end,
            t: 1.0,
        }
    }

    ///Create a line with a changed thickness
    pub fn with_thickness(self, thickness: f32) -> LineT {
        LineT {
            t: thickness,
            ..self
        }
    }

    pub fn draw(self) -> Vec<Vector> {
        // adapted from https://stackoverflow.com/a/8189511
        // Right hand normal with x positive to the right and y positive down
        let normal_raw = Vector::new(-(self.b.y - self.a.y), self.b.x - self.a.x);
        // Normalize normal
        let normal = normal_raw.normalize();

        let p1 = Vector::new(
            self.a.x - self.t / 2.0 * normal.x,
            self.a.y - self.t / 2.0 * normal.y,
        );
        let p2 = Vector::new(
            self.b.x - self.t / 2.0 * normal.x,
            self.b.y - self.t / 2.0 * normal.y,
        );
        let p3 = Vector::new(
            self.b.x + self.t / 2.0 * normal.x,
            self.b.y + self.t / 2.0 * normal.y,
        );
        let p4 = Vector::new(
            self.a.x + self.t / 2.0 * normal.x,
            self.a.y + self.t / 2.0 * normal.y,
        );
        let path = vec![p1, p2, p3, p4];

        path
    }
}
