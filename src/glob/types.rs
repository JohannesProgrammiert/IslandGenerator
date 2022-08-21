use crate::glob::*;
use num::traits::Zero;
use std::ops::AddAssign;
use std::ops::SubAssign;
use std::ops::{Add, Div, Mul, Sub};
use std::fmt::Debug;
#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    NoDirection,
    North,
    South,
    West,
    East,
    NorthWest,
    SouthWest,
    NorthEast,
    SouthEast,
}

pub trait Coordinate<T>: Sized + Sub<Output = Self> + Add<Output = Self> + SubAssign + AddAssign
where T: num::traits::Num + Copy + Clone
{
    fn x(&self) -> T;
    fn y(&self) -> T;
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Coord<T>
    where T: num::Num
{
    x: T,
    y: T,
}

impl<T> Coord<T>
    where T: num::Num
    {
    pub fn new(x: T, y: T) -> Self {
        Coord { x, y }
    }
}

impl<T> Add for Coord<T>
    where T: num::Num
{
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> Sub for Coord<T>
    where T: num::Num
{
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Mul<f32> for Coord<f32>
{
    type Output = Self;
    fn mul(self, other: f32) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<f32> for Coord<f32>
{
    type Output = Self;
    fn div(self, other: f32) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
impl Mul<isize> for Coord<isize>
{
    type Output = Self;
    fn mul(self, other: isize) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<isize> for Coord<isize>
{
    type Output = Self;
    fn div(self, other: isize) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
impl Mul<usize> for Coord<usize>
{
    type Output = Self;
    fn mul(self, other: usize) -> Self::Output {
        Self {
            x: self.x* other,
            y: self.y* other,
        }
    }
}

impl Div<usize> for Coord<usize>
{
    type Output = Self;
    fn div(self, other: usize) -> Self::Output {
        Self {
            x: self.x/ other,
            y: self.y/ other,
        }
    }
}

impl<T> AddAssign for Coord<T>
    where T: num::Num + Copy
{
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> SubAssign for Coord<T>
    where T: num::Num + Copy
{
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T> Coordinate<T> for Coord<T>
    where T: num::Num + Copy
{
    fn x(&self) -> T {
        return self.x;
    }
    fn y(&self) -> T {
        return self.y;
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct WorldCoordinate {
    x: f32,
    y: f32,
}

impl WorldCoordinate {
    pub const fn new(x: f32, y: f32) -> Self {
        WorldCoordinate { x, y }
    }
    pub fn from_screen(pixel: ScreenCoordinate, camera_pos: WorldCoordinate, scale: f32) -> Self {
        let relative_pos: WorldCoordinate = WorldCoordinate::new(
            0.5 * (pixel.x() / TILE_QUARTER.x() + pixel.y() / TILE_QUARTER.y()) / scale,
            0.5 * (pixel.y() / TILE_QUARTER.y() - pixel.x() / TILE_QUARTER.x()) / scale,
        );
        WorldCoordinate {
            x: relative_pos.x + camera_pos.x,
            y: relative_pos.y + camera_pos.y,
        }
    }
}

impl Coordinate<f32> for WorldCoordinate {
    fn x(&self) -> f32 {
        return self.x;
    }
    fn y(&self) -> f32 {
        return self.y;
    }
}
impl Add for WorldCoordinate {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl AddAssign for WorldCoordinate {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for WorldCoordinate {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for WorldCoordinate {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for WorldCoordinate {
    type Output = Self;
    fn mul(self, other: f32) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<f32> for WorldCoordinate {
    type Output = Self;
    fn div(self, other: f32) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ScreenCoordinate {
    x: f32,
    y: f32,
}

impl ScreenCoordinate {
    pub const fn new(x: f32, y: f32) -> Self {
        ScreenCoordinate { x, y }
    }
    pub fn from_world(world_pos: WorldCoordinate, camera_pos: WorldCoordinate, scale: f32) -> Self {
        let relative_pos: WorldCoordinate = world_pos - camera_pos;
        ScreenCoordinate {
            x: (relative_pos.x() - relative_pos.y()) * scale * TILE_QUARTER.x(),
            y: (relative_pos.x() + relative_pos.y()) * scale * TILE_QUARTER.y(),
        }
    }
}

impl Coordinate<f32> for ScreenCoordinate {
    fn x(&self) -> f32 {
        return self.x;
    }
    fn y(&self) -> f32 {
        return self.y;
    }
}

impl Add for ScreenCoordinate {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for ScreenCoordinate {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for ScreenCoordinate {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for ScreenCoordinate {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for ScreenCoordinate {
    type Output = Self;
    fn mul(self, other: f32) -> Self::Output {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<f32> for ScreenCoordinate {
    type Output = Self;
    fn div(self, other: f32) -> Self::Output {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

pub trait Rectangle<T: Coordinate<F>, F: Zero + Sub<Output = F> + PartialOrd + Copy>
    where F: num::Num
{
    fn new(upper_left: T, lower_right: T) -> Self;
    fn upper_left(&self) -> T;
    fn lower_right(&self) -> T;
    fn lower_left(&self) -> T;
    fn upper_right(&self) -> T;
    fn intersects(&self, other: &Self) -> bool {
        return self.upper_left().x() < other.upper_right().x()
            && self.upper_right().x() > other.upper_left().x()
            && self.upper_left().y() < other.lower_left().y()
            && self.lower_left().y() > other.upper_left().y();
    }
    fn width(&self) -> F {
        let w = self.upper_right().x() - self.upper_left().x();
        if w > F::zero() {
            return w;
        } else {
            return F::zero() - w;
        }
    }
    fn height(&self) -> F {
        let h = self.lower_left().y() - self.upper_left().y();
        if h > F::zero() {
            return h;
        } else {
            return F::zero() - h;
        }
    }
    fn shift(&mut self, offset: T);
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct WorldRect {
    upperleft: WorldCoordinate,
    upperright: WorldCoordinate,
    lowerleft: WorldCoordinate,
    lowerright: WorldCoordinate,
}

impl WorldRect {
    pub fn from_screen(rect: ScreenRect, scale: f32, screen_pos: WorldCoordinate) -> Self {
        WorldRect {
            upperleft: WorldCoordinate::from_screen(
                ScreenCoordinate::new(0.0, 0.0) - rect.lower_right() / 2.0,
                screen_pos,
                scale,
            ),
            upperright: WorldCoordinate::from_screen(
                ScreenCoordinate::new(rect.lower_right().x(), -rect.lower_right().y()) / 2.0,
                screen_pos,
                scale,
            ),
            lowerleft: WorldCoordinate::from_screen(
                ScreenCoordinate::new(-rect.lower_right().x(), rect.lower_right().y()),
                screen_pos,
                scale,
            ),
            lowerright: WorldCoordinate::from_screen(rect.lower_right() / 2.0, screen_pos, scale),
        }
    }
}

impl Default for WorldRect {
    fn default() -> Self {
        let zero = WorldCoordinate::new(0.0, 0.0);
        WorldRect {
            upperleft: zero,
            lowerleft: zero,
            upperright: zero,
            lowerright: zero,
        }
    }
}

impl Rectangle<WorldCoordinate, f32> for WorldRect {
    fn new(upper_left: WorldCoordinate, lower_right: WorldCoordinate) -> Self {
        WorldRect {
            upperleft: upper_left,
            lowerleft: WorldCoordinate::new(upper_left.x(), lower_right.y()),
            lowerright: lower_right,
            upperright: WorldCoordinate::new(lower_right.x(), upper_left.y()),
        }
    }
    fn upper_left(&self) -> WorldCoordinate {
        self.upperleft
    }
    fn lower_right(&self) -> WorldCoordinate {
        self.lowerright
    }
    fn lower_left(&self) -> WorldCoordinate {
        self.lowerleft
    }
    fn upper_right(&self) -> WorldCoordinate {
        self.upperright
    }
    fn shift(&mut self, offset: WorldCoordinate) {
        self.upperright += offset;
        self.lowerleft += offset;
        self.upperright += offset;
        self.lowerright += offset;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ScreenRect {
    upperleft: ScreenCoordinate,
    upperright: ScreenCoordinate,
    lowerleft: ScreenCoordinate,
    lowerright: ScreenCoordinate,
}

impl Rectangle<ScreenCoordinate, f32> for ScreenRect {
    fn new(upper_left: ScreenCoordinate, lower_right: ScreenCoordinate) -> Self {
        ScreenRect {
            upperleft: upper_left,
            lowerleft: ScreenCoordinate::new(upper_left.x(), lower_right.y()),
            lowerright: lower_right,
            upperright: ScreenCoordinate::new(lower_right.x(), upper_left.y()),
        }
    }
    fn upper_left(&self) -> ScreenCoordinate {
        self.upperleft
    }
    fn lower_right(&self) -> ScreenCoordinate {
        self.lowerright
    }
    fn lower_left(&self) -> ScreenCoordinate {
        self.lowerleft
    }
    fn upper_right(&self) -> ScreenCoordinate {
        self.upperright
    }
    fn shift(&mut self, offset: ScreenCoordinate) {
        self.upperright += offset;
        self.lowerleft += offset;
        self.upperright += offset;
        self.lowerright += offset;
    }
}

impl Default for ScreenRect {
    fn default() -> Self {
        let zero = ScreenCoordinate::new(0.0, 0.0);
        ScreenRect {
            upperleft: zero,
            lowerleft: zero,
            upperright: zero,
            lowerright: zero,
        }
    }
}

#[derive(Debug)]
pub struct Rect<T>
    where T: num::Num
{
    upperleft: Coord<T>,
    upperright: Coord<T>,
    lowerleft: Coord<T>,
    lowerright: Coord<T>,
}

impl<T> Rectangle<Coord<T>, T> for Rect<T>
    where T: num::Num + std::cmp::PartialOrd + Copy
{
    fn new(upper_left: Coord<T>, lower_right: Coord<T>) -> Self {
        Rect {
            upperleft: upper_left,
            lowerleft: Coord::new(upper_left.x(), lower_right.y()),
            lowerright: lower_right,
            upperright: Coord::new(lower_right.x(), upper_left.y()),
        }
    }
    fn upper_left(&self) -> Coord<T> {
        self.upperleft
    }
    fn lower_right(&self) -> Coord<T> {
        self.lowerright
    }
    fn lower_left(&self) -> Coord<T> {
        self.lowerleft
    }
    fn upper_right(&self) -> Coord<T> {
        self.upperright
    }
    fn shift(&mut self, offset: Coord<T>) {
        self.upperright += offset;
        self.lowerleft += offset;
        self.upperright += offset;
        self.lowerright += offset;
    }

}
