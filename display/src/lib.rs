
use std::sync::mpsc::Receiver;
use crate::tetra::Window;

mod tetra;

/// Перечисление определяет как образом можно отобразить ячейку.
#[derive(Copy, Clone)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum CellStuff {
    KilledAnimal,
    DeadAnimal,
    HerbLeft,
    HerbRight,
    HerbFront,
    HerbBack,
    CarnLeft,
    CarnRight,
    CarnFront,
    CarnBack,
    Plant,
    None,
}

// Синонимы типов
pub type Point = (usize, usize, CellStuff);

pub type Map = Vec<Point>;

/// Перечисление с типами драйверов.
pub enum ScreenType {
    Tetra,
}

pub fn launch_screen(
    screen_type: ScreenType,
    width: usize,
    height: usize,
    receiver: Receiver<Map>,
    base_path: &str,
    title: &str,
) -> Result<(), String> {
    match screen_type {
        ScreenType::Tetra => {
            Window::new(
                width,
                height,
                receiver,
                base_path,
                title
            )?;

            Ok(())
        }
    }
}