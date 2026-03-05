//! 数据模型定义

pub mod save;
pub mod panpan;
pub mod shop;
pub mod garden;
pub mod travel;
pub mod memory;
pub mod recipe;
pub mod customer;
pub mod command;
pub mod dialogue;

// 重新导出常用类型
pub use save::Save;
pub use panpan::PanpanState;
pub use shop::ShopState;
pub use garden::GardenPlot;
pub use travel::Travel;
pub use memory::MemoryFragment;
pub use recipe::Recipe;
pub use customer::CustomerRecord;
pub use command::Command;
pub use dialogue::DialogueMessage;
