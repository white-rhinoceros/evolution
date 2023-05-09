//! Простое животное.

use crate::animal::brains::AnimalBrain;
use crate::animal::{AnimalAction, AnimalAlive, AnimalDirection, AnimalInputSignal, AnimaType};
use crate::landscape::Energy;

const TURN_ACTION_ENERGY_RATE: f64 = 1.0;

const MOVE_ACTION_ENERGY_RATE: f64 = 1.0;

const EAT_ACTION_ENERGY_RATE: f64 = 1.0;

const REPRODUCE_ACTION_ENERGY_RATE: f64 = 1.0;

const NONE_ACTION_ENERGY_RATE: f64 = 1.0;

/// Структура, описывающая состояние агента.
pub struct Animal<B: AnimalBrain> {
    // Параметры животного
    animal_type: AnimaType,      // Тип животного.

    energy: Energy,              // Энергия животного.
    max_energy: Energy,          // Максимальная энергия которую может иметь животное.
    live_energy: Energy,         // Базовая энергия гомеостаза.
    birth_energy: Energy,        // Энергия которую получит новое животное при размножении.
    eaten_energy_rate: f64,      // Доля собственной энергия животного, которую получает
                                 // животное съевшее текущее животное.

    reproduce_energy_rate: f64,  // Критерий готовности к размножению.
    no_repro: bool,              // Запрет на размножение.

    direction: AnimalDirection,  // Текущее направление движения животного (север,
                                 // юг, восток, запад).

    // Статистика
    age: usize,                  // Возраст животного в "прожитых" итерациях.
    generation: usize,           // Поколение животного (количество его предков).
    is_eaten: bool,              // Признак того, что животное съели.
    processed: bool,             // Животное совершило "свой ход" на текущей итерации.

    // Мозг
    brain: B,
}

impl<B: AnimalBrain + 'static> Animal<B> {
    /// Конструктор. Создает новое животное.
    /// На параметром типа  наложено ограничение: трейт AnimalBrain плюс статическое
    /// временя жизни. Это означает, что тип(!) должен существовать во время компиляции,
    /// иначе получим ошибку: the parameter type `B` may not live long enough.
    ///
    /// # Arguments
    ///
    /// * `animal_type`: Тип животного (травоядное, хищник).
    /// * `energy`: Начальная энергия животного.
    /// * `max_energy`: Максимальная энергия, которую мождет иметь животное.
    /// * `live_energy`: Энергия, которую животное теряет на каждой итерации не
    /// зависимо от типа его действия (энергия гомеостаза). На основе этой величины
    /// вычисляются потери энергии для других действий (движение, поворот, и т.д.).
    /// * `eaten_energy_rate`: Доля энергии которую получит хищник, когда съест
    /// текущееживотное.
    /// * `reproduce_energy_rate`: Критерий готовности к размножению.
    /// * `no_repro`: Запретить размножение животного.
    /// * `direction`: Текущее направление движения.
    /// * `generation`: Поколение. Для животных созданных в начали мира должно
    /// равняться нулю.
    ///
    /// returns: `Box<dyn(AnimalAlive)>`
    pub fn new(
        animal_type: AnimaType,
        energy: Energy,
        max_energy: Energy,
        live_energy: Energy,
        eaten_energy_rate: f64,
        reproduce_energy_rate: f64,
        no_repro: bool,
        direction: AnimalDirection,
        generation: usize,
    ) -> Box<dyn(AnimalAlive)> {
        let brain = B::new();

        // Рождение, это уже "действие" животного, по этому processed = true.
        // В противном случае - некоторые животные совершили бы еще один ход
        // на текущей итерации, а некоторые нет.
        Box::new(Animal {
            animal_type,
            energy,
            max_energy,
            live_energy,
            birth_energy: energy,
            eaten_energy_rate,
            reproduce_energy_rate,
            no_repro,
            direction,
            age: 0,
            generation,
            is_eaten: false,
            processed: true,
            brain,
        })
    }
}

impl<B: AnimalBrain + 'static> AnimalAlive for Animal<B> {
    // Методы получения состояния животного.

    fn is_dead(&self) -> bool {
        if self.energy <= 0 {
            return true;
        }

        false
    }

    fn is_eaten(&self) -> bool {
        self.is_eaten
    }

    fn is_processed(&self) -> bool {
        self.processed
    }

    fn get_type(&self) -> AnimaType {
        self.animal_type
    }

    fn get_direction(&self) -> AnimalDirection {
        self.direction
    }

    fn get_age(&self) -> usize {
        self.age
    }

    fn get_generation(&self) -> usize {
        self.generation
    }

    fn clear(&mut self) {
        self.processed = false;
    }

    // Метод Action

    fn action(&mut self, inputs: &AnimalInputSignal) -> AnimalAction {
        // Животное прожило еще одну итерацию.
        self.age += 1;
        // Животное совершило "свой ход".
        self.processed = true;

        // Размножение животного не зависит от решения его мозга.
        if !self.no_repro
            && self.energy > (self.reproduce_energy_rate * self.max_energy as f64) as Energy {
            return AnimalAction::Reproduce;
        }

        // Передаем вектор входных сигналов в мозг животного.
        self.brain.action(inputs)
    }

    // Действия, которые реализуют "желания" животного.

    /// Изменяет направление движения в соответствии с поворотом животного.
    ///
    /// # Arguments
    ///
    /// * `turn_left`: Признак, показывающий направление
    /// поворота: `true` - поворот налево, `false` - поворот направо.
    fn turn_action(&mut self, turn_left: bool) {
        // Любое действие животного сопровождается потреблением энергии.
        self.energy -= (TURN_ACTION_ENERGY_RATE * self.live_energy as f64) as Energy;

        match self.direction {
            AnimalDirection::North => {
                if turn_left {
                    self.direction = AnimalDirection::West;
                } else {
                    self.direction = AnimalDirection::East;
                }
            }
            AnimalDirection::South => {
                if turn_left {
                    self.direction = AnimalDirection::East;
                } else {
                    self.direction = AnimalDirection::West;
                }
            }
            AnimalDirection::East => {
                if turn_left {
                    self.direction = AnimalDirection::North;
                } else {
                    self.direction = AnimalDirection::South;
                }
            }
            AnimalDirection::West => {
                if turn_left {
                    self.direction = AnimalDirection::South;
                } else {
                    self.direction = AnimalDirection::North;
                }
            }
        }
    }

    /// Движение животного в перед. Мир должен вызвать это действие - тем самым разрешив его.
    fn move_action(&mut self, _realized: bool) {
        self.energy -= (MOVE_ACTION_ENERGY_RATE * self.live_energy as f64) as Energy;
    }

    fn eat_action(&mut self, energy: Energy) {
        self.energy -= (EAT_ACTION_ENERGY_RATE * self.live_energy as f64) as Energy;
        self.energy += energy;

        if self.energy > self.max_energy {
            self.energy = self.max_energy;
        }
    }

    fn reproduce_action(&mut self) -> Box<dyn AnimalAlive> {
        self.energy -= (REPRODUCE_ACTION_ENERGY_RATE * self.live_energy as f64) as Energy;
        // Часть своей энергии передает потомку.
        self.energy -= self.birth_energy;

        let brain = self.brain.clone_with_mutation();

        Box::new(Animal {
            animal_type: self.animal_type,
            energy: self.birth_energy,
            max_energy: self.max_energy,
            live_energy: self.live_energy,
            birth_energy: self.birth_energy,
            eaten_energy_rate: self.eaten_energy_rate,
            reproduce_energy_rate: self.reproduce_energy_rate,
            no_repro: false, // Если текущее размножилось, то потомки тоже могут.
            direction: self.direction,
            age: 0,
            generation: self.generation + 1,
            is_eaten: false,
            processed: false,
            brain,
        })
    }

    fn inactivity_action(&mut self) {
        self.energy -= (NONE_ACTION_ENERGY_RATE * self.live_energy as f64) as Energy;
    }

    // Действия, которые можно совершить с животным против его воли.

    fn be_eaten(&mut self) -> Energy {
        // TODO: Пока мы просто съедаем травоядное, в последующих реализациях
        // TODO: можно съедать только убитое животное. Получается хищник сможет
        // TODO: съедать другого хищника, травоядное сможет реализовывать
        // TODO: стратегии с атакой и убийством хищников (для обороны). Тем
        // TODO: не менее, травоядное не может съесть хищника, но эти правила
        // TODO: закладываются не в этом методе, а в общих правилах мира и мозга,
        // TODO: тем более мы эти правила можем и изменить введя в рассмотрение
        // TODO: всеядных животных.

        if self.animal_type == AnimaType::Herbivore {
            // Частично съесть травоядное нельзя. Найдем энергию которую получит хищник.
            let energy =  (self.eaten_energy_rate * self.energy as f64) as Energy;

            // Обнуляем энергию (травоядное погибло).
            self.energy = 0;

            // Показываем от чего именно умерло животное.
            self.is_eaten = true;

            // Съеденное животное теряет возможность совершать действия, т.к. мертво.
            self.processed = true;

            energy
        } else {
            // Хищника вообще съесть нельзя.
            0
        }
    }
}