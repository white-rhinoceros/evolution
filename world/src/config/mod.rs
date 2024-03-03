use display::ScreenType;
use crate::landscape::Energy;

pub mod init;

// Настройки программы.

/// Рабочая директория
//pub const WORKING_DIR: &str = "D:/Projects/RustProjects/evolution";

/// Драйвер отображения: console, window, none.
pub const SCREEN_TYPE: ScreenType = ScreenType::Tetra;

/// Не отображать мир на экране. Должно быть true для реальных расчетов.
pub const HEADLESS_MODE: bool = false;


// Настройки среды

/// Максимальное количество итераций мира.
pub const MAX_STEPS: usize = 1000; // 1000000

/// Пошаговый режим
//pub const STEP: bool = false;

/// Размеры сетки мира.
/// (96, 54) максимальный размер мира в текущей реализации, соответствует разрешению 1920x1080.
pub const GRID_WIDTH: usize = 96;
pub const GRID_HEIGHT: usize = 54;

/// Максимальное количество растений.
/// 0 - не ограничено.
pub const MAX_PLANTS: usize = 35;

/// Максимальное количество травоядных.
/// 0 - не ограничено.
pub const MAX_HERBIVORE: usize = 18;

/// Максимальное количество хищников.
/// 0 - не ограничено.
pub const MAX_CARNIVORE: usize = 18;

/// Максимальная энергия которую может получить растение на каждой итерации.
pub const MAX_PLANT_GROW_ENERGY: Energy = 5.;



// Настройки растений

/// Максимальная энергия которую может иметь растение.
pub const MAX_PLANT_ENERGY: Energy = 15.;

/// Максимальная энергия, которую может получить животное при поедании растения.
pub const PLANT_EATEN_ENERGY: Energy = 15.;

/// Константа определяет благоприятные условия для размножения животного.
pub const PLANT_REPRODUCE_ENERGY_RATE: f64 = 0.5;

/// Запрещает размножение растений.
pub const PLANT_NO_REPRO: bool = true;


// Настройки животных

// Максимальная энергия которую может иметь животное.
pub const MAX_ANIMAL_ENERGY: Energy = 60.;

// Энергия, которую получает животное при рождении (и теряет размножающееся животное).
pub const ANIMAL_BIRTH_ENERGY: Energy = 25.;

// Энергия, которую теряет животное, что-бы жить.
pub const ANIMAL_LIVE_ENERGY: Energy = 0.005;

// Какую часть от энергии съеденного животного получит хищник.
pub const ANIMAL_EATEN_ENERGY_RATE: f64 = 0.3;

// Константа определяет благоприятные условия для размножения животного. Т.е. как только,
// энергия животного достигнет величины, равной этой доли от максимальной энергии животного,
// животное размножится.
pub const ANIMAL_REPRODUCE_ENERGY_RATE: f64 = 0.9;

// No reproduction
pub const ANIMAL_NO_REPRO: bool = false;
