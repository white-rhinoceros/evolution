//! Программа моделирование эволюции "Эволюция".

use crate::animal::brains::simple::Brain as AnimalBrain;
use crate::animal::species::simple::Animal;
use crate::plant::simple::Plant;

// Настройки
use crate::config::*;

use std::sync::mpsc::channel;
use std::thread::{JoinHandle, spawn};
use std::time::Duration;

use crate::animal::{AnimalDirection, AnimaType};
use crate::landscape::Landscape;

use display::{launch_screen, Map};

mod animal;
mod plant;
mod config;
mod landscape;
mod errors;
mod init;

fn main() {
    println!("Программа \"Эволюция\"");

    // Создаем мир.
    let mut world = Landscape::new(
        GRID_WIDTH,
        GRID_HEIGHT,
        MAX_HERBIVORE,
        MAX_CARNIVORE,
        MAX_PLANTS,
        MAX_PLANT_GROW_ENERGY
    ).expect("Ошибка создания мира!");

    // // Заселение мира растениями и животными.
    // let mut plant = Plant::new(
    //     MAX_PLANT_ENERGY,
    //     MAX_PLANT_ENERGY,
    //     PLANT_EATEN_ENERGY,
    //     PLANT_REPRODUCE_ENERGY_RATE,
    //     PLANT_NO_REPRO
    // );
    // world.add_plant(0, 0, plant).expect("Ячейка занята!");
    //
    // let mut herb = Animal::<AnimalBrain>::new(
    //     AnimaType::Herbivore,
    //     ANIMAL_BIRTH_ENERGY,
    //     MAX_ANIMAL_ENERGY,
    //     ANIMAL_LIVE_ENERGY,
    //     ANIMAL_EATEN_ENERGY_RATE,
    //     ANIMAL_REPRODUCE_ENERGY_RATE,
    //     ANIMAL_NO_REPRO,
    //     AnimalDirection::North,
    //     0,
    // );
    // world.add_animal(0, 1, herb).expect("Ячейка занята!");

    let mut carn = Animal::<AnimalBrain>::new(
        AnimaType::Carnivore,
        ANIMAL_BIRTH_ENERGY,
        MAX_ANIMAL_ENERGY,
        ANIMAL_LIVE_ENERGY,
        ANIMAL_EATEN_ENERGY_RATE,
        ANIMAL_REPRODUCE_ENERGY_RATE,
        ANIMAL_NO_REPRO,
        AnimalDirection::North,
        0,
    );
    world.add_animal(5, 5, carn).expect("Ячейка занята!");

    if HEADLESS_MODE == false {
        // Канал для пересылки сообщений о состоянии мира.
        let (sender, receiver) = channel::<Map>();

        // Запуск отображения мира в отдельном потоке.
        let handler = spawn(|| {
            launch_screen(
                SCREEN_TYPE,
                GRID_WIDTH,
                GRID_HEIGHT,
                receiver,
                "D:/Projects/RustroverProjects/evolution",
                "Программа эволюция"
            ).expect("Ошибка создания экрана!");
        });

        // Итерации мира.
        for _ in 0..MAX_STEPS {
            // Одна итерация
            world.tick();

            // Собираем карту состояния мира для отображения.
            sender.send(world.get_view_state()).expect("Не удалось отправить данные для отображения в канал");

            use std::thread;
            //thread::sleep(Duration::from_millis(1000));
        }

        // Если итерации мира закончились, ждем явного выхода из окна отображения мира.
        handler.join().unwrap();
    } else {
        use chrono::Utc;
        use round::round;

        let start = Utc::now().timestamp() as f64;

        // Итерации мира.
        for _ in 0..MAX_STEPS {
            // Одна итерация
            world.tick();
        }

        let end = Utc::now().timestamp() as f64;

        println!("Программа проработала {} минут(ы)", round((end - start)/60.0, 4));
    }
}
