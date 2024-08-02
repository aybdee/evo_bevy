use bevy::math::Vec2;
use rand::Rng;

pub fn generate_random_vec2(x_bounds: (f32, f32), y_bounds: (f32, f32)) -> Vec2 {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(x_bounds.0..=x_bounds.1);
    let y = rng.gen_range(y_bounds.0..=y_bounds.1);
    Vec2::new(x, y)
}

pub struct Grid2d<T: Default + Clone> {
    pub width: usize,
    pub height: usize,
    data: Vec<T>,
}

impl<T: Default + Clone> Grid2d<T> {
    pub fn new(width: usize, height: usize) -> Self {
        Grid2d {
            width,
            height,
            data: vec![T::default(); width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.data.get(y * self.width + x)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.data.get_mut(y * self.width + x)
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        if let Some(cell) = self.get_mut(x, y) {
            *cell = value;
        }
    }

    // pub fn find_unique(&self, value: T) -> Option<(usize, usize)> {
    //     for y in 0..self.height {
    //         for x in 0..self.width {
    //             // if self.get(x, y).unwrap() == Some(&value) {
    //             //     return Some((x, y));
    //             // }
    //         }
    //     }
    //     None
    // }
}
