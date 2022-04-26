use crate::pb::Spec;
use image::ImageOutputFormat;

mod photon;
pub use photon::Photon;

pub trait Engine {
    // 对 engine 按照 specs 进行一系列的处理
    fn apply(&mut self, specs: &[Spec]);
    // 从 engine 中生成目标图片，使用 self 而非 self 的引用
    fn generate(self, format: ImageOutputFormat) -> Vec<u8>;
}

// SpecTransform: 如果未来增加了更多的 spec，只需要实现它就可以了
pub trait SpecTransform<T> {
    fn transform(&mut self, op: T);
}
