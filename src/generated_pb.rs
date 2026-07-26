pub mod pb_app {
    include!(concat!(env!("OUT_DIR"), "/pb_app.rs"));
}
pub mod pb_desktop {
    include!(concat!(env!("OUT_DIR"), "/pb_desktop.rs"));
}
pub mod pb_gpio {
    include!(concat!(env!("OUT_DIR"), "/pb_gpio.rs"));
}
pub mod pb_gui {
    include!(concat!(env!("OUT_DIR"), "/pb_gui.rs"));
}
pub mod pb_property {
    include!(concat!(env!("OUT_DIR"), "/pb_property.rs"));
}
pub mod pb_storage {
    include!(concat!(env!("OUT_DIR"), "/pb_storage.rs"));
}
pub mod pb_system {
    include!(concat!(env!("OUT_DIR"), "/pb_system.rs"));
}
pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/pb.rs"));
}
