//! 数据模型定义

pub mod command;
pub mod customer;
pub mod dialogue;
pub mod garden;
pub mod memory;
pub mod panpan;
pub mod recipe;
pub mod save;
pub mod shop;
pub mod travel;

// 重新导出常用类型
pub use command::Command;
pub use customer::CustomerRecord;
pub use dialogue::DialogueMessage;
pub use garden::GardenPlot;
pub use memory::MemoryFragment;
pub use panpan::PanpanState;
pub use recipe::Recipe;
pub use save::Save;
pub use shop::ShopState;
pub use travel::Travel;
