use std::cell::RefCell;

use image::RgbaImage;
use mint::Vector2;

#[derive(Debug)]
pub(crate) struct Node {
    pub(crate) children: Option<[Box<RefCell<Node>>; 2]>,
    pub(crate) topleft: Vector2<u32>,
    pub(crate) botright: Vector2<u32>,
    pub(crate) image_index: Option<usize>,
}

impl Node {
    pub fn insert(&mut self, image: &RgbaImage, index: usize) -> bool {
        match &self.children {
            Some([left, right]) => {
                if left.borrow_mut().insert(image, index) {
                    true
                } else {
                    right.borrow_mut().insert(image, index)
                }
            }

            None => {
                if self.image_index.is_some() {
                    return false;
                }

                let img_size: Vector2<u32> = [image.width(), image.height()].into();

                if self.botright.x < img_size.x || self.botright.y < img_size.y {
                    return false;
                }

                if img_size == self.botright {
                    self.image_index = Some(index);
                    return true;
                }

                let dx = (self.botright.x - self.topleft.x) - img_size.x;
                let dy = (self.botright.y - self.topleft.y) - img_size.y;

                let children = if dx > dy {
                    [
                        Node {
                            children: None,
                            topleft: self.topleft,
                            botright: [self.topleft.x + img_size.x - 1, self.botright.y].into(),
                            image_index: Some(index),
                        },
                        Node {
                            children: None,
                            topleft: [self.topleft.x + img_size.x, self.topleft.y].into(),
                            botright: self.botright,
                            image_index: None,
                        },
                    ]
                } else {
                    [
                        Node {
                            children: None,
                            topleft: self.topleft,
                            botright: [self.botright.x, self.topleft.y + img_size.y - 1].into(),
                            image_index: Some(index),
                        },
                        Node {
                            children: None,
                            topleft: [self.topleft.x, self.topleft.y + img_size.y].into(),
                            botright: self.botright,
                            image_index: None,
                        },
                    ]
                };

                self.children = Some(children.map(|a| Box::new(RefCell::new(a))));
                true
            }
        }
    }
}
