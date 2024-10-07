use std::collections::HashMap;

use bevy::math::Vec2;
use rand::Rng;

pub fn generate_random_vec2(x_bounds: (f32, f32), y_bounds: (f32, f32)) -> Vec2 {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(x_bounds.0..=x_bounds.1);
    let y = rng.gen_range(y_bounds.0..=y_bounds.1);
    Vec2::new(x, y)
}

pub fn f32_to_vec2(f: (f32, f32)) -> Vec2 {
    Vec2::new(f.0, f.1)
}

pub fn vec2_to_i32(vec: Vec2) -> (i32, i32) {
    (vec.x as i32, vec.y as i32)
}

pub struct Grid2d<T: Default + Clone + PartialEq> {
    pub width: usize,
    pub height: usize,
    pub data: Vec<T>,
}

impl<T: Default + Clone + PartialEq> Grid2d<T> {
    pub fn new(width: usize, height: usize) -> Self {
        Grid2d {
            width,
            height,
            data: vec![T::default(); width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        let item = self.data.get(y * self.width + x);
        item.filter(|&item| *item != T::default())
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.data.get_mut(y * self.width + x)
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        if let Some(cell) = self.get_mut(x, y) {
            *cell = value;
        }
    }
}
