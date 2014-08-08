
use graphics::{
    BackEnd,
};

use Texture;

/// The graphics back-end.
pub struct Gfx2d;

impl Gfx2d {
    /// Creates a new Gfx2d object.
    pub fn new() -> Gfx2d {
        Gfx2d
    }
}

impl BackEnd<Texture> for Gfx2d {

}
