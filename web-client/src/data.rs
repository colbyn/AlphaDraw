//! Basic data types

use std::collections::LinkedList;
#[derive(Debug, Clone, Copy, Default)]
pub struct Point([f64; 2]);

impl Point {
    pub fn new<T: Into<f64>>(x: T, y: T) -> Self {
        let x: f64 = x.into();
        let y: f64 = y.into();
        Point([x, y])
    }
    pub fn into(self) -> [f64; 2] {
        self.0
    }
    pub fn x(self) -> f64 {self.0[0]}
    pub fn y(self) -> f64 {self.0[1]}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SegmentPoint {
    pub point: Point,
}

impl SegmentPoint {
    pub fn new(point: Point) -> Self {
        SegmentPoint {
            point,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Segment {
    pub points: LinkedList<SegmentPoint>,
}





#[derive(Debug, Clone, Default)]
pub struct Segments(pub LinkedList<Segment>);

impl Segments {
    pub fn begin_new_segment(&mut self, point: Point) {
        let mut new_segment = Segment::default();
        new_segment.points.push_back(SegmentPoint::new(point));
        self.0.push_back(new_segment);
    }
    /// Add the given point to the current segment.
    pub fn add_point(&mut self, point: Point) {
        if self.0.len() == 0 {
            self.begin_new_segment(point);
            return
        }
        let segment = self.0.back_mut().unwrap();
        segment.points.push_back(SegmentPoint::new(point));
    }
}




