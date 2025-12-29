use std::collections::HashSet;
use std::sync::OnceLock;

pub(crate) fn get_image_media_types() -> &'static HashSet<&'static str> {
    static IMAGE_MEDIA_TYPES: OnceLock<HashSet<&'static str>> = OnceLock::new();
    IMAGE_MEDIA_TYPES.get_or_init(|| {
        let mut set = HashSet::new();
        set.insert("image/jpeg");
        set.insert("image/png");
        set.insert("image/gif");
        set.insert("image/webp");
        set
    })
}
