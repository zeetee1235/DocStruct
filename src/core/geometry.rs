use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl BBox {
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self { x0, y0, x1, y1 }
    }

    pub fn width(&self) -> f32 {
        (self.x1 - self.x0).max(0.0)
    }

    pub fn height(&self) -> f32 {
        (self.y1 - self.y0).max(0.0)
    }

    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    pub fn center(&self) -> (f32, f32) {
        ((self.x0 + self.x1) * 0.5, (self.y0 + self.y1) * 0.5)
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }

    pub fn iou(&self, other: &Self) -> f32 {
        let x0 = self.x0.max(other.x0);
        let y0 = self.y0.max(other.y0);
        let x1 = self.x1.min(other.x1);
        let y1 = self.y1.min(other.y1);

        let inter = BBox::new(x0, y0, x1, y1);
        let inter_area = inter.area();
        let union = self.area() + other.area() - inter_area;
        if union <= 0.0 {
            0.0
        } else {
            inter_area / union
        }
    }

    pub fn center_distance(&self, other: &Self) -> f32 {
        let (cx1, cy1) = self.center();
        let (cx2, cy2) = other.center();
        ((cx1 - cx2).powi(2) + (cy1 - cy2).powi(2)).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn computes_iou() {
        let a = BBox::new(0.0, 0.0, 10.0, 10.0);
        let b = BBox::new(5.0, 5.0, 15.0, 15.0);
        let iou = a.iou(&b);
        assert_eq!(iou, 25.0 / 175.0);
    }
}
