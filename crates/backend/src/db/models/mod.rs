//! 数据模型定义

pub mod command;
pub mod customer;
pub mod dialogue;
pub mod garden;
pub mod memory;
pub mod panda;
pub mod recipe;
pub mod shop;
pub mod travel;
pub mod weather;

// 重新导出常用类型
pub use command::Command;
pub use customer::CustomerRecord;
pub use dialogue::DialogueMessage;
pub use garden::GardenPlot;
pub use memory::MemoryFragment;
pub use panda::PandaState;
pub use recipe::Recipe;
pub use shop::ShopState;
pub use travel::Travel;
pub use weather::Weather;
