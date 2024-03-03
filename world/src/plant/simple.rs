//! Модуль описывающий простое растение.
//! Растение не должно знать свои координаты, т.е. где оно выросло. В месте с тем,
//! растение должно хранить энергию, которую оно может отдать при поедании его животным.

use crate::landscape::Energy;
use crate::plant::{PlantAction, PlantAlive};

/// Структура, описывающая растение.
pub struct Plant {
    // Энергия запасенная в растении. После полного съедения (энергия меньше или равна нулю),
    // растение умирает, но может вырасти заново.
    energy: Energy,

    // Максимальная энергия которую может иметь растение.
    max_energy: Energy,

    // Энергия которую отдают растения когда их поедают животные.
    eaten_energy: Energy,

    //
    reproduce_energy_rate: f64,

    // Параметр определяющий может ли растение размножаться или нет.
    no_repro: bool
}

impl Plant {

    /// Конструктор. Создает новое растение.
    ///
    /// # Arguments
    ///
    /// * `energy`: Текущая энергия растения.
    /// * `max_energy`: Максимально возможная энергия растения.
    /// * `eaten_energy`: Сколько энергии отдает растение за раз при его поедании.
    /// * `reproduce_energy_rate`: Критерий готовности к размножению.
    /// * `no_repro`: Запрещает размножение.
    ///
    /// returns: Box<Plant>
    pub fn new(
        energy: Energy,
        max_energy: Energy,
        eaten_energy: Energy,
        reproduce_energy_rate: f64,
        no_repro: bool
    ) -> Box<Plant> {
        Box::new(Plant {
            energy,
            max_energy,
            eaten_energy,
            reproduce_energy_rate,
            no_repro,
        })
    }
}

impl PlantAlive for Plant {
    // Методы получения состояния растения.

    /// Съедено ли растение?
    fn is_eaten(&self) -> bool {
        if self.energy <= 0 as Energy {
            return true;
        }

        false
    }

    // Метод Action

    /// Действие растения.
    fn action(&mut self) -> PlantAction {
        // Размножение животного не зависит от решения его мозга.
        if !self.no_repro
            && self.energy > (self.reproduce_energy_rate * self.max_energy as f64) as Energy {
            return PlantAction::Reproduce;
        }


        return if self.energy < self.max_energy {
            PlantAction::Grow
        } else {
            PlantAction::None
        }
    }

    // Действия, которые реализуют "желания" растения.

    /// Действие "рост растения".
    fn grow_action(&mut self, energy: Energy) {
        self.energy += energy;

        if self.energy > self.max_energy {
            self.energy = self.max_energy;
        }
    }

    /// Действие "размножение растения".
    fn reproduce_action(&mut self) -> Box<dyn PlantAlive> {
        Box::new(Plant {
            energy: 0 as Energy, // Семечко не имеет энергии и должно прорасти в растение.
            max_energy: self.max_energy,
            eaten_energy: self.eaten_energy,
            reproduce_energy_rate: self.reproduce_energy_rate,
            no_repro: false
        })
    }

    /// Действие "нет действия".
    fn inactivity_action(&mut self) {
        // Пока пусто.
    }

    // Действия, которые можно совершить с растением против его воли.

    /// Поедание растения.
    fn be_eaten(&mut self) -> Energy {
        if self.eaten_energy > self.energy {
            self.energy -= self.eaten_energy;

            self.eaten_energy
        } else {
            let rest = self.energy;
            self.energy = 0 as Energy;

            rest
        }
    }
}