use image::{GrayImage, RgbaImage};

pub struct RemovalResult {
    pub image: RgbaImage,
    pub mask: GrayImage,
}

impl RemovalResult {
    pub fn image(&self) -> &RgbaImage {
        &self.image
    }

    pub fn mask(&self) -> &GrayImage {
        &self.mask
    }

    pub fn into_parts(self) -> (RgbaImage, GrayImage) {
        (self.image, self.mask)
    }
}
