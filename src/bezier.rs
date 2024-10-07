use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

fn de_casteljau(points: Vec<Vec2>, t: f32) -> Vec<Vec2> {
    let mut new_points = Vec::with_capacity(points.len() - 1);
    for i in 0..points.len() - 1 {
        let point = points[i].lerp(points[i + 1], t); // Linear interpolation
        new_points.push(point);
    }
    new_points
}

fn flatten_to_cubic(points: Vec<Vec2>, _tolerance: f32) -> Vec<(Vec2, Vec2, Vec2, Vec2)> {
    // If the points already form a cubic Bezier, return the single segment
    if points.len() == 4 {
        return vec![(points[0], points[1], points[2], points[3])];
    }

    // Subdivide the Bezier using de Casteljau's algorithm
    let mut left = points.clone();
    let mut right = points.clone();

    for i in 1..left.len() {
        left = de_casteljau(left, 0.5);
        right = de_casteljau(right, 0.5);
    }

    let left_segment = vec![points[0], left[0], left[1], left[2]];
    let right_segment = vec![right[2], right[1], right[0], points[points.len() - 1]];

    // Recursively flatten the left and right segments
    let mut result = flatten_to_cubic(left_segment, _tolerance);
    result.append(&mut flatten_to_cubic(right_segment, _tolerance));

    result
}
