//! Среда.

use std::cmp::Ordering;
use std::fmt;
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;

use crate::errors::{RecoverableError, AddAgentError};
use crate::animal::{AnimalAction, AnimalAlive, AnimalDirection, AnimalInputSignal, AnimaType};
use crate::plant::{PlantAction, PlantAlive};

use display::{CellStuff, Map};


/// Тип представляющий энергию живого существа
pub type Energy = f32;

/// Тип агента.
#[derive(Copy, Clone)]
pub enum AgentType {
    Plant,
    Herbivore,
    Carnivore,
}

// Константы смещений, в зависимости от "взгляда" животного. Каждая константа хранят
// массив кортежей смещения точек. Проходя по всем смещениям относительно текущего
// положения агента, мы обходим ту или иную область вокруг агента. Кортеж представляет
// две точки: "x" и "y".
//
// Положительное направление оси "y" в низ. У оси "x" положительное направление
// слева на право.
//
// Пример областей, в случае, если животное смотрит на север. Случай, когда
// животное смотрит на юг, определяется отражением всех координат.
// F F F F F
// L P P P R
// L P X P R
//
// Пример областей, в случае, если животное смотрит на запад (на лево).
// Случай, когда животное смотрит на восток, определяется отражением всех координат.
// F R R
// F P P
// F P X
// F P P
// F L L

/// Константы определяющие смещения по сетке при определенном "взгляде"
/// животного (прямо, слева, и т.д.) в зависимости от разворота животного.

// Grid offsets for Front/Left/Right/Proximity (North facing).
const NORTH_FRONT: [(i8, i8); 5] = [(-2, -2), (-1, -2), (0, -2), (1, -2), (2, -2)];
const NORTH_LEFT: [(i8, i8); 2] = [(-2, 0), (-2, -1)];
const NORTH_RIGHT: [(i8, i8); 2] = [(2, 0), (2, -1)];
const NORTH_PROXIMITY: [(i8, i8); 5] = [(-1, 0), (-1, -1), (0, -1), (1, -1), (1, 0)];

// Grid offsets for Front/Left/Right/Proximity (South facing).
const SOUTH_FRONT: [(i8, i8); 5] = [(2, 2), (1, 2), (0, 2), (-1, 2), (-2, 2)];
const SOUTH_LEFT: [(i8, i8); 2] = [(2, 0), (2, 1)];
const SOUTH_RIGHT: [(i8, i8); 2] = [(-2, 0), (-2, 1)];
const SOUTH_PROXIMITY: [(i8, i8); 5] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)];

// Grid offsets for Front/Left/Right/Proximity (West facing).
const WEST_FRONT: [(i8, i8); 5] = [(-2, 2), (-2, 1), (-2, 0), (-2, -1), (-2, -2)];
const WEST_LEFT: [(i8, i8); 2] = [(0, 2), (-1, 2)];
const WEST_RIGHT: [(i8, i8); 2] = [(0, -2), (-1, -2)];
const WEST_PROXIMITY: [(i8, i8); 5] = [(0, 1), (-1, 1), (-1, 0), (-1, 1), (0, 1)];

// Grid offsets for Front/Left/Right/Proximity (East facing).
const EAST_FRONT: [(i8, i8); 5] = [(-2, 2), (-2, 1), (-2, 0), (-2, -1), (-2, -2)];
const EAST_LEFT: [(i8, i8); 2] = [(0, 2), (-1, 2)];
const EAST_RIGHT: [(i8, i8); 2] = [(0, -2), (-1, -2)];
const EAST_PROXIMITY: [(i8, i8); 5] = [(0, 1), (-1, 1), (-1, 0), (-1, 1), (0, 1)];

/// Создает матрицу среды ячейками которой являются значения C типа.
///
/// # Arguments
///
/// * `width`: Ширина среды.
/// * `height`: Высота среды.
///
/// returns: Vec<Vec<C, Global>, Global>
fn create_landscape_matrix<C>(width: usize, height: usize) -> Vec<Vec<C>>
    where
        C: Default
{
    let mut width_container: Vec<Vec<C>> = Vec::new();

    for _ in 0..width {
        let mut height_container: Vec<C> = Vec::new();

        for _ in 0..height {
            let cell: C = Default::default();
            height_container.push(cell);
        }

        width_container.push(height_container);
    }

    width_container
}

/// Сортирует вектор координат (кортеж (i8, i8)) случайным образом.
///
/// # Arguments
///
/// * `array`: Вектор кортежей координат точек.
///
/// returns: Vec<(i8, i8)>
fn randomize_coord_vector(mut array: Vec<(i8, i8)>) -> Vec<(i8, i8)> {
    array.sort_unstable_by(|_, _| {
        let num = thread_rng().gen_range(0..2);
        if num == 1 {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });

    array
}

/// Растение в ячейке. Растения не умираю, а теряют энергию до нуля.
/// На последующих итерациях растение может вырасти снова.
#[derive(Copy, Clone)]
enum PlantInCell {
    Plant(*mut dyn PlantAlive),
    None,
}
impl Default for PlantInCell {
    fn default() -> Self {
        PlantInCell::None
    }
}

/// Животное в ячейке. Животное может погибнуть и может передвигаться.
/// Каждое такое действие сопровождается освобождением занимаемой ячейки.
/// В случае смерти животного можно было бы освобождать ячейку, но для
/// статистики и пока это не вызывает проблем с производительностью будем
/// переносить указатель на умершее животное в специальный массив.
#[derive(Copy, Clone)]
enum AnimalInCell {
    Animal(*mut dyn AnimalAlive),
    None,
}
impl Default for AnimalInCell {
    fn default() -> Self {
        AnimalInCell::None
    }
}

/// Ячейка среды. В ячейке хранятся указатели на агенты.
#[derive(Default)]
struct Cell {
    // Текущее растение в точке.
    plant: PlantInCell,
    // Текущее животное в точке.
    animal: AnimalInCell,
}

/// Структурой, объединяющей все вместе является среда - двухмерная структура, на
/// пересечении координат которой находится ячейка. Среда имеет два массива: растения
/// и животные. Напрямую с этим массивом мы не работаем, они лишь контейнеры. Перед
/// переносом в эти контейнеры мы получаем изменяемый *указатель* на сущность и
/// храним их в ячейке в каждой точке.
pub struct Landscape {
    // Агенты.

    // Массив животных.
    animals: Vec<Box<dyn AnimalAlive>>,
    // Массив растений.
    plants: Vec<Box<dyn PlantAlive>>,
    // Умершие животные. Растение погибнуть не может - оно может вырасти заново.
    // TODO: Возможно стоит рассмотреть варианты с погибшими растениями, восстановление
    // TODO: популяции которых происходит только при размножении.
    dead_animals: Vec<*mut dyn AnimalAlive>,

    // Среда. Точки среды - ячейки.
    landscape: Vec<Vec<Cell>>,

    // Вспомогательный массив, содержит элементы позволяющие отобразить текущую ячейку.
    view_state: Map,

    // Вспомогательные массивы для случайного размещения агентов в мире.
    shuffle_width: Vec<usize>,
    shuffle_height: Vec<usize>,

    // Настройки мира.

    // Ширина мира.
    width: usize,
    // Высота мира.
    height: usize,
    // Максимальное количество растений.
    max_plants: usize,
    // Максимальное количество травоядных.
    max_herbivore: usize,
    // Максимальное количество хищных животных.
    max_carnivore: usize,
    // Энергия, которую получает растение на каждой итерации.
    // В дальнейшим можно создавать карту энергии.
    plant_grow_energy: Energy,

    // Статистика мира.
    // В случае кортежа: первый элемент - травоядное, второй хищное.

    // Общее количество растений (не съеденных) в мире.
    plant_count: usize,
    // Количество живых животных в мире.
    animal_count: (usize, usize),
    // Текущие, живые долгожители (имеющие максимальный срок жизни в итерациях).
    best_animal: (AnimalInCell, AnimalInCell),
    // Указатель на лучшее умершее животное (прожившее дольше всех в итерациях).
    best_death_animal: (AnimalInCell, AnimalInCell),
    // Количество размножений животных.
    animal_reproductions: (usize, usize),
    // Количество смертей животных.
    animal_deaths: (usize, usize),
    // Максимальное достигнутое поколение животных.
    animal_max_generation: (usize, usize),
}

impl Landscape {
    /// Конструктор. Условно считаем, что мир простирается слева на право и с верху вниз:
    /// (0, 0) - левый, верхний угол; (width, height) - правый, нижний угол. Значение
    /// ширины и высоты не может превышать половину максимального значения типа isize
    /// для платформы для которой производится сборка программы.
    ///
    /// # Arguments
    ///
    /// * `width`: "Ширина" среды (мира).
    /// * `height`: "Высота" среды (мира).
    /// * `max_plants`: Максимальное количество растений.
    /// * `max_herbivore`: Максимальное количество травоядных.
    /// * `max_carnivore`: Максимальное количество хищников.
    /// * `plant_grow_energy`: Энергия которую среда будет передавать растению на каждой итерации.
    /// Этим самым мы как-бы эмулируем солнечный свет.
    ///
    /// TODO: Сделать сезонность на основе параметра plant_grow_energy, а так-же неоднородность по среде.
    /// TODO: Это позволит эмулировать "изменение климата", "времена года" и разные климатические зоны.
    /// TODO: В идеале это должно привести к тому, что разные области будут населять разные животные.
    ///
    /// returns: Result<World, CreatingWorldError>
    pub fn new(
        width: usize,
        height: usize,
        max_plants: usize,
        max_herbivore: usize,
        max_carnivore: usize,
        plant_grow_energy: Energy
    ) -> Result<Landscape, RecoverableError> {
        if width > isize::MAX.try_into().unwrap() ||  height > isize::MAX.try_into().unwrap() {
            return Err(RecoverableError::new(
                fmt::format(format_args!(
                    "Размеры мира ({}, {}) по каждой координате не должны превышать максимального значения типа isize для вашей платформы",
                    width,
                    height
                ))
            ));
        }

        // Массивы значений координат ячеек представленных в случайном порядке
        // для поиска случайных не занятых мест.
        let mut shuffle_width: Vec<usize> = (0..width).collect();
        let mut shuffle_height: Vec<usize> = (0..height).collect();
        shuffle_width.shuffle(&mut thread_rng());
        shuffle_height.shuffle(&mut thread_rng());

        Ok(Landscape {
            // Агенты.
            animals: vec![],
            plants: vec![],
            dead_animals: vec![],

            // Среда.
            landscape: create_landscape_matrix(width, height),
            view_state: Vec::with_capacity(max_plants * max_herbivore * max_carnivore),
            shuffle_width,
            shuffle_height,

            // Параметры мира.
            width,
            height,
            max_plants,
            max_herbivore,
            max_carnivore,
            plant_grow_energy,

            // Статистика.
            plant_count: 0,
            animal_count: (0, 0),
            best_animal: (AnimalInCell::None, AnimalInCell::None),
            best_death_animal: (AnimalInCell::None, AnimalInCell::None),
            animal_reproductions: (0, 0),
            animal_deaths: (0, 0),
            animal_max_generation: (0, 0),
        })
    }

    /// Обрезает координаты, что-бы обеспечить тороидальность мира.
    ///
    /// # Arguments
    ///
    /// * `coord`: Координата местоположения (x или y). Может быть отрицательной.
    /// * `max_size`: Максимальный размер мира по соответствующей координате.
    ///
    /// Returns: Координата в границах мира.
    fn clip(coord: isize, max_size: usize) -> usize {
        // Конвертируем параметры в тип со знаком.
        let max_size: isize = max_size as isize;

        if coord < 0 {
            // В процессе вычисления получилась отрицательная координата.
            // В этом случае берем смещение от максимальной границы мира,
            // на размер отрицательной координаты.
            return (max_size + coord) as usize;
        } else if coord > (max_size - 1) {
            // Если-же координата выходит за максимальные границы мира, то
            // берем остаток от деления координаты на размер мира в этом
            // направлении.
            return (coord % max_size) as usize;
        }

        // По умолчанию возвращаем переданную координату.
        coord as usize
    }

    /// Метод - обертка, конвертирует изменяемый указатель в разделяемую ссылку.
    /// Метод универсален, добавлен для сокращения unsafe блоков.
    ///
    /// # Arguments
    ///
    /// * `t`: Изменяемый указатель на тип T.
    ///
    /// Returns: &T
    fn get_agent_ref<'a, T: ?Sized>(t: *mut T) -> &'a T {
        unsafe {
            t.as_ref().expect("Обнаружен нулевой указатель на агента")
        }
    }

    /// Метод - обертка, конвертирует изменяемый указатель в изменяемую ссылку.
    /// Метод универсален, добавлен для сокращения unsafe блоков.
    ///
    /// # Arguments
    ///
    /// * `t`: Изменяемый указатель на тип T.
    ///
    /// Returns: &mut T
    fn get_agent_mut<'a, T: ?Sized>(t: *mut T) -> &'a mut T {
        unsafe {
            t.as_mut().expect("Обнаружен нулевой указатель на агента")
        }
    }

    // /// Возвращает ширину мира.
    // pub fn get_width(&self) -> usize {
    //     self.width
    // }
    //
    // /// Возвращает высоту мира.
    // pub fn get_height(&self) -> usize {
    //     self.height
    // }

    /// Возвращает состояние ячейки, т.е. информацию, которую можно отобразить
    /// для данной ячейки.
    ///
    /// # Arguments
    ///
    /// * `x`: Координата "x" местоположения.
    /// * `y`: Координата "y" местоположения.
    ///
    /// returns: Vec<CellStuff, Global>
    pub fn get_view_state(&self) -> Map {
        self.view_state.clone()
    }

    /// Find an empty spot for the agent within its particular type.
    ///
    /// # Arguments
    ///
    /// * `agent_type`: Тип агента для которого пытаемся найти "место в мире".
    ///
    /// returns: Result<(usize, usize), RecoverableError>
    pub fn find_empty_spot(&self, agent_type: AgentType) -> Result<(usize, usize), RecoverableError> {
        match agent_type {
            AgentType::Plant => {
                if self.plant_count >= self.max_plants {
                    return Err(RecoverableError::new(
                        fmt::format(format_args!(
                            "Достигнуто максимальное количество ({}) растений в мире",
                            self.max_plants,
                        ))
                    ))
                }
            }
            AgentType::Herbivore => {
                if self.animal_count.0 >= self.max_herbivore {
                    return Err(RecoverableError::new(
                        fmt::format(format_args!(
                            "Достигнуто максимальное количество ({}) травоядных в мире",
                            self.max_herbivore,
                        ))
                    ))
                }
            }
            AgentType::Carnivore => {
                if self.animal_count.1 >= self.max_carnivore {
                    return Err(RecoverableError::new(
                        fmt::format(format_args!(
                            "Достигнуто максимальное количество ({}) хищников в мире",
                            self.max_carnivore,
                        ))
                    ))
                }
            }
        }

        match agent_type {
            AgentType::Plant => {
                // Просматриваем все ячейки, но в случайном порядке.
                for test_x in &self.shuffle_width {
                    for test_y in &self.shuffle_height {
                        if let PlantInCell::None = self.landscape[*test_x][*test_y].plant {
                            // Точка свободна, берем ее.
                            return Ok((*test_x, *test_y));
                        }
                    }
                }

                // Вряд ли это случится, но если все ячейки заняты...
                return Err(RecoverableError::new(fmt::format(format_args!(
                    "Не удалось найти свободное место для растения"
                ))));
            }

            AgentType::Herbivore | AgentType::Carnivore => {
                for test_x in &self.shuffle_width {
                    for test_y in &self.shuffle_height {
                        if let AnimalInCell::None = self.landscape[*test_x][*test_y].animal {
                            return Ok((*test_x, *test_y));
                        }
                    }
                }

                // Вряд ли жто случится, но если все ячейки заняты...
                panic!("По каким-то причинам, в мире закончилось место для новых животных!");
            }
        }
    }

    /// Добавляет растение в мир.
    ///
    /// # Arguments
    ///
    /// * `x`: Координата "x" местоположения.
    /// * `y`: Координата "y" местоположения.
    /// * `plant`: Растение: тип должен реализовывать типаж PlantAlive и быть обернут в Box.
    ///
    /// returns: Result<(), WorldError>
    pub fn add_plant(
        &mut self,
        mut x: usize,
        mut y: usize,
        mut plant: Box<dyn PlantAlive>
    ) -> Result<(), AddAgentError> {
        // Если переданная точка выходит за "границы" мира.
        if x >= self.width || y >= self.height {
            return Err(
                AddAgentError::OutOfBounds((x, y))
            );
        }

        x = Self::clip(x as isize, self.width);
        y = Self::clip(y as isize, self.height);

        // Нужно проверить, не занято ли место в ячейке.
        if let PlantInCell::None = self.landscape[x][y].plant {
            // С начала в cell мы помещаем изменяемый указать на растение.
            self.landscape[x][y].plant = PlantInCell::Plant(plant.as_mut());

            // Затем переносим "бокс" с растением, в общий массив растений.
            // Порядок важен, если мы сделаем наоборот, то попытаемся получить
            // изменяемую ссылку у перемещенного объекта.
            self.plants.push(plant);
            self.plant_count += 1;
        } else {
            return Err(
                AddAgentError::TakenCell((x, y))
            );
        }

        Ok(())
    }

    /// Добавляет животное в мир.
    ///
    /// # Arguments
    ///
    /// * `x`: Координата "x" местоположения.
    /// * `y`: Координата "y" местоположения.
    /// * `animal`: Животное: тип должен реализовывать типаж AnimalAlive и быть обернут в Box.
    ///
    /// returns: Result<(), WorldError>
    pub fn add_animal(
        &mut self,
        mut x: usize,
        mut y: usize,
        mut animal: Box<dyn AnimalAlive>
    ) -> Result<(), AddAgentError> {
        // Если переданная точка выходит за "границы" мира.
        if x >= self.width || y >= self.height {
            return Err(
                AddAgentError::OutOfBounds((x, y))
            );
        }

        x = Self::clip(x as isize, self.width);
        y = Self::clip(y as isize, self.height);

        // Нужно проверить, не занято ли место в ячейке.
        if let AnimalInCell::None = self.landscape[x][y].animal {
            // Изменяемая ссылка на животное.
            let animal_ref = animal.as_mut();
            let animal_type = animal_ref.get_type();

            // С начала в cell мы помещаем изменяемый указать на животное
            // (изменяемая ссылка конвертируется в изменяемый указатель,
            // с внутренней точки зрения это одно и тоже).
            self.landscape[x][y].animal = AnimalInCell::Animal(animal_ref);

            // Затем переносим "бокс" с животным, в общий массив животных.
            // Порядок важен, если мы сделаем наоборот, то попытаемся получить
            // изменяемую ссылку у перемещенного объекта.
            self.animals.push(animal);

            match animal_type {
                AnimaType::Herbivore => {
                    self.animal_count.0 += 1;
                }
                AnimaType::Carnivore => {
                    self.animal_count.1 += 1;
                }
            }
        } else {
            return Err(
                AddAgentError::TakenCell((x, y))
            );
        }

        Ok(())
    }

    /// Одна симуляция всего мира.
    pub fn tick(&mut self) {
        // Перед каждой итерацией тасуем вектора координат. Т.к. сложность алгоритма тасовки
        // составляет 2*N, то это не представляет особых проблем с производительностью.
        self.shuffle_width.shuffle(&mut thread_rng());
        self.shuffle_height.shuffle(&mut thread_rng());

        // Перебираем ячейки в случайном порядке!
        for x in &self.shuffle_width.clone() {
            for y in &self.shuffle_height.clone() {
                // Симуляция травы.
                match self.landscape[*x][*y].plant {
                    // В точке есть растение.
                    PlantInCell::Plant(ptr) => {
                        // Получим изменяемую ссылку на значение на которое "указывает" указатель.
                        // Непосредственно работать с указателем мы не можем. Если ссылка получена,
                        // то это уже безопасный код.
                        let plant = Self::get_agent_mut(ptr);
                        self.simulate_plant(plant, *x, *y);
                    },
                    // Нет растения - ничего не делать.
                    PlantInCell::None => {},
                }

                // Симуляция животных.
                match self.landscape[*x][*y].animal {
                    // В точке есть животное.
                    AnimalInCell::Animal(ptr) => {
                        // Изменяемая ссылка на животное.
                        let animal = Self::get_agent_mut(ptr);

                        // Проверяем обработанность животного.
                        // Возможно животное уже сделало "свой ход". Как такое возможно, что в новь
                        // обрабатываемая точка уже содержит животное сделавшее свой ход? Рассмотрим
                        // пример: текущая итерация обрабатывает точку (1, 1). Животное перемещается
                        // в точку (1, 2). Когда итерация дойдет до точки (1, 2) животное повторно
                        // совершит свое действие, что неверно.
                        if animal.is_processed() == true {
                            continue;
                        }

                        // К этому моменту мертвого животного в точке быть не может (исключается
                        // параметром is_processed).
                        if animal.is_dead() {
                            panic!("Попытка симуляции мертвого животного в ячейке {}, {}.", x, y);
                        };

                        // Даем животному, своими активными действиями, шанс выжить.
                        self.simulate_animal(animal, *x, *y);
                    },
                    // Нет животного - ничего не делать.
                    AnimalInCell::None => {},
                }
            }
        }

        // Завершающая обработка.
        self.final_processing();
    }

    /// Симуляция травы в указанной точке.
    ///
    /// # Arguments
    ///
    /// * `plant`: Изменяемый указатель на текущее, симулируемое растение.
    /// * `x`: "x" координата симулируемого растения.
    /// * `y`: "y" координата симулируемого растения.
    ///
    /// Returns: ()
    fn simulate_plant(&mut self, plant: &mut dyn PlantAlive, x: usize, y: usize) {
        // Получаем то, что хочет растение.
        let action = plant.action();

        match action {
            // Растение ничего не хочет (кроме гомеостаза).
            PlantAction::None => {
                self.inactivity_plant_action(plant, x, y);
            }
            // Растение хочет расти.
            PlantAction::Grow => {
                self.grow_plant_action(plant, x, y);
            }
            // Растение решило размножиться (рассыпать семена).
            PlantAction::Reproduce => {
                self.reproduce_plant_action(plant, x, y);
            }
        }
    }

    /// Действие - нет действия.
    fn inactivity_plant_action(&mut self, plant: &mut dyn PlantAlive, x: usize, y: usize) {
        self.landscape[x][y].plant = self.landscape[x][y].plant;
        plant.inactivity_action();
    }

    /// Реализует рост растения.
    fn grow_plant_action(&mut self, plant: &mut dyn PlantAlive, x: usize, y: usize) {
        self.landscape[x][y].plant = self.landscape[x][y].plant;
        plant.grow_action(self.plant_grow_energy);
    }

    /// Реализует размножение растения.
    ///
    /// # Arguments
    ///
    /// * `plant`: Изменяемая ссылка на текущее, симулируемое растение.
    /// * `_x`: "x" координата симулируемого растения.
    /// * `_y`: "y" координата симулируемого растения.
    ///
    /// Returns: ()
    fn reproduce_plant_action(&mut self, plant: &mut dyn PlantAlive, _x: usize, _y: usize) {
        let spot = self.find_empty_spot(AgentType::Plant);

        match spot {
            // Ячейка нашлась.
            Ok(coord) => {
                let new_plant = plant.reproduce_action();
                self.add_plant(coord.0, coord.1, new_plant)
                    .expect("Не удалось добавить растение");
            }
            // Не удалось найти свободную ячейку... Пропускаем...
            Err(_) => {}
        }
    }

    /// Симуляция животного в точке.
    ///
    /// This is the main animal simulation routine. This function performs
    /// the perception phase which fills in the input cells for the animal's
    /// brain. This is based upon the particular direction of the agent.
    /// The agent brain determines the action to be taken. A function is
    /// then called based upon the action selected.
    ///
    /// # Arguments
    ///
    /// * `animal`: Изменяемая ссылка на текущее, симулируемое животное.
    /// * `x`: "x" координата симулируемого животного.
    /// * `y`: "y" координата симулируемого животного.
    ///
    /// returns: ()
    fn simulate_animal(&mut self, animal: &mut dyn AnimalAlive, x: usize, y: usize) {
        // Determine inputs for the agent brain.
        let inputs = self.percept(animal, x, y);
        let action = animal.action(&inputs);

        // Perform Action
        match action {
            AnimalAction::TurnLeft => {
                self.turn_left_animal_action(animal, x, y);
            }
            AnimalAction::TurnRight => {
                self.turn_right_animal_action(animal, x, y);
            }
            AnimalAction::Move => {
                self.movement_animal_action(animal, x, y);
            }
            AnimalAction::Eat => {
                self.eating_animal_action(animal, x, y);
            }
            AnimalAction::Reproduce => {
                self.reproduce_animal_action(animal)
            }
            AnimalAction::None => {
                self.inactivity_animal_action(animal)
            }
        }
    }

    /// Животное "должно посмотреть по сторонам" (по соответствующим областям в зависимости
    /// от направления) и заполнить структуру содержащую переменные входных сигналов для
    /// мозга животного. Животное видит текущее состояние мира, т.е. остальные агенты
    /// могли у этому моменту сделать свой шаг, а некоторые еще ждут своей очереди.
    ///
    /// TODO: В дальнейшем планирую использовать "карту восприятия", матрицу
    /// TODO: где заполнены соответствующие позиции с информацией о агентах
    /// TODO: (их наличие).
    ///
    /// # Arguments
    ///
    /// * `animal`: Изменяемая ссылка на животное.
    /// * `x`: Положение животного по "x".
    /// * `y`: Положение животного по "y".
    ///
    /// returns: AnimalInputSignal
    fn percept(&self, animal: &mut dyn AnimalAlive, x: usize, y: usize) -> AnimalInputSignal {
        let mut inputs =  AnimalInputSignal {
            plant_front: 0,
            plant_left: 0,
            plant_right: 0,
            plant_proximity: 0,
            herbivore_front: 0,
            herbivore_left: 0,
            herbivore_right: 0,
            herbivore_proximity: 0,
            carnivore_front: 0,
            carnivore_left: 0,
            carnivore_right: 0,
            carnivore_proximity: 0,
        };

        match animal.get_direction() {
            // Животное смотрит на север
            AnimalDirection::North => {
                let count = self.count_agents_in_area(&NORTH_FRONT, x, y);
                inputs.plant_front = count.0;
                inputs.herbivore_front = count.1;
                inputs.carnivore_front = count.2;

                let count = self.count_agents_in_area(&NORTH_LEFT, x, y);
                inputs.plant_left = count.0;
                inputs.herbivore_left = count.1;
                inputs.carnivore_left = count.2;

                let count = self.count_agents_in_area(&NORTH_RIGHT, x, y);
                inputs.plant_right = count.0;
                inputs.herbivore_right = count.1;
                inputs.carnivore_right = count.2;

                let count = self.count_agents_in_area(&NORTH_PROXIMITY, x, y);
                inputs.plant_proximity = count.0;
                inputs.herbivore_proximity = count.1;
                inputs.carnivore_proximity = count.2;
            }
            // Животное смотри на юг
            AnimalDirection::South => {
                let count = self.count_agents_in_area(&SOUTH_FRONT, x, y);
                inputs.plant_front = count.0;
                inputs.herbivore_front = count.1;
                inputs.carnivore_front = count.2;

                let count = self.count_agents_in_area(&SOUTH_LEFT, x, y);
                inputs.plant_left = count.0;
                inputs.herbivore_left = count.1;
                inputs.carnivore_left = count.2;

                let count = self.count_agents_in_area(&SOUTH_RIGHT, x, y);
                inputs.plant_right = count.0;
                inputs.herbivore_right = count.1;
                inputs.carnivore_right = count.2;

                let count = self.count_agents_in_area(&SOUTH_PROXIMITY, x, y);
                inputs.plant_proximity = count.0;
                inputs.herbivore_proximity = count.1;
                inputs.carnivore_proximity = count.2;
            }
            // Животное смотрит на запад
            AnimalDirection::West => {
                let count = self.count_agents_in_area(&WEST_FRONT, x, y);
                inputs.plant_front = count.0;
                inputs.herbivore_front = count.1;
                inputs.carnivore_front = count.2;

                let count = self.count_agents_in_area(&WEST_LEFT, x, y);
                inputs.plant_left = count.0;
                inputs.herbivore_left = count.1;
                inputs.carnivore_left = count.2;

                let count = self.count_agents_in_area(&WEST_RIGHT, x, y);
                inputs.plant_right = count.0;
                inputs.herbivore_right = count.1;
                inputs.carnivore_right = count.2;

                let count = self.count_agents_in_area(&WEST_PROXIMITY, x, y);
                inputs.plant_proximity = count.0;
                inputs.herbivore_proximity = count.1;
                inputs.carnivore_proximity = count.2;
            }
            // Животное смотрит на восток
            AnimalDirection::East => {
                let count = self.count_agents_in_area(&EAST_FRONT, x, y);
                inputs.plant_front = count.0;
                inputs.herbivore_front = count.1;
                inputs.carnivore_front = count.2;

                let count = self.count_agents_in_area(&EAST_LEFT, x, y);
                inputs.plant_left = count.0;
                inputs.herbivore_left = count.1;
                inputs.carnivore_left = count.2;

                let count = self.count_agents_in_area(&EAST_RIGHT, x, y);
                inputs.plant_right = count.0;
                inputs.herbivore_right = count.1;
                inputs.carnivore_right = count.2;

                let count = self.count_agents_in_area(&EAST_PROXIMITY, x, y);
                inputs.plant_proximity = count.0;
                inputs.herbivore_proximity = count.1;
                inputs.carnivore_proximity = count.2;
            }
        }

        inputs
    }

    /// Метод вычисляет количество агентов в точках которые переданы срезом.
    ///
    /// # Arguments
    ///
    /// * `offsets`: Срез смещений относительно заданной точки.
    /// * `x`: Координата "x" точки относительно которой ищутся агенты.
    /// * `y`: Координата "y" точки относительно которой ищутся агенты.
    ///
    /// Returns: (usize, usize, usize) - количество растений, травоядных, хищников.
    fn count_agents_in_area(&self, offsets: &[(i8, i8)], x: usize, y: usize) -> (usize, usize, usize) {
        let mut plants: usize = 0;
        let mut herbivores: usize = 0;
        let mut carnivores: usize = 0;

        for coord in offsets {
            let x_off = Self::clip(
                x as isize + coord.0 as isize,
                self.width
            );

            let y_off = Self::clip(
                y as isize + coord.1 as isize,
                self.height
            );

            if let PlantInCell::Plant(plant) = self.landscape[x_off][y_off].plant {
                let plant = Self::get_agent_ref(plant);

                if !plant.is_eaten() {
                    plants += 1;
                }
            }

            if let AnimalInCell::Animal(animal) = self.landscape[x_off][y_off].animal {
                let animal = Self::get_agent_ref(animal);

                if !animal.is_dead() {
                    match animal.get_type() {
                        AnimaType::Herbivore => {
                            herbivores += 1;
                        }
                        AnimaType::Carnivore => {
                            carnivores += 1;
                        }
                    }
                }
            }

        }

        (plants, herbivores, carnivores)
    }

    /// Реализует поворот животного на лево.
    fn turn_left_animal_action(&mut self, animal: &mut dyn AnimalAlive, _x: usize, _y: usize) {
        animal.turn_action(true);
    }

    /// Реализует поворот животного на право.
    fn turn_right_animal_action(&mut self, animal: &mut dyn AnimalAlive, _x: usize, _y: usize) {
        animal.turn_action(false);
    }

    /// Implements the move function.
    ///
    /// # Arguments
    ///
    /// * `animal`: Изменяемая ссылка на животное.
    /// * `x`: Положение животного по "x".
    /// * `y`: Положение животного по "y".
    ///
    /// returns: ()
    fn movement_animal_action(&mut self, animal: &mut dyn AnimalAlive, x: usize, y: usize) {
        // Определим координаты новой точки местоположения животного.
        let coords = match animal.get_direction() {
            AnimalDirection::North => {
                (x, Self::clip(y as isize - 1, self.height))
            }
            AnimalDirection::South => {
                (x, Self::clip(y as isize + 1, self.height))
            }
            AnimalDirection::West => {
                (Self::clip(x as isize - 1, self.width), y)
            }
            AnimalDirection::East => {
                (Self::clip(x as isize + 1, self.width), y)
            }
        };

        // Проверить возможность движения.
        match self.landscape[coords.0][coords.1].animal {
            AnimalInCell::Animal(_) => {
                // В точке есть другое животное.
                animal.move_action(false);
            },
            AnimalInCell::None => {
                // Точка свободна, перемещаемся.
                self.landscape[coords.0][coords.1].animal = self.landscape[x][y].animal;
                self.landscape[x][y].animal = AnimalInCell::None;

                animal.move_action(true);
            },
        }
    }

    /// Реализует функцию поедания у животного. Возможность съесть что-то определяется ранее,
    /// в методе Self::percept, где животно анализирует текущую обстановку.
    ///
    /// # Arguments
    ///
    /// * `animal`: Изменяемая ссылка на животное.
    /// * `x`: Положение животного по "x".
    /// * `y`: Положение животного по "y".
    ///
    /// Returns: ()
    fn eating_animal_action(&self, animal: &mut dyn AnimalAlive, x: usize, y: usize) {
        match animal.get_type() {
            // Травоядное ест траву
            AnimaType::Herbivore => {
                let coord = match animal.get_direction() {
                    AnimalDirection::North => {
                        self.choose_plant(x, y, &NORTH_PROXIMITY)
                    }
                    AnimalDirection::South => {
                        self.choose_plant(x, y, &SOUTH_PROXIMITY)
                    }
                    AnimalDirection::West => {
                        self.choose_plant(x, y, &WEST_PROXIMITY)
                    }
                    AnimalDirection::East => {
                        self.choose_plant(x, y, &EAST_PROXIMITY)
                    }
                };

                match coord {
                    Some(coord) => {
                        // Получить растение по координатам
                        if let PlantInCell::Plant(plant) = self.landscape[coord.0][coord.1].plant {
                            let plant = Self::get_agent_mut(plant);

                            animal.eat_action(plant.be_eaten());
                        }
                    }
                    None => {
                        // Есть нечего: животное ошиблось.
                    }
                }

            }
            // Хищник поедает травоядное
            AnimaType::Carnivore => {
                let coord = match animal.get_direction() {
                    AnimalDirection::North => {
                        self.choose_animal(AnimaType::Herbivore, x, y, &NORTH_PROXIMITY)
                    }
                    AnimalDirection::South => {
                        self.choose_animal(AnimaType::Herbivore, x, y, &SOUTH_PROXIMITY)
                    }
                    AnimalDirection::West => {
                        self.choose_animal(AnimaType::Herbivore, x, y, &WEST_PROXIMITY)
                    }
                    AnimalDirection::East => {
                        self.choose_animal(AnimaType::Herbivore, x, y, &EAST_PROXIMITY)
                    }
                };

                match coord {
                    Some(coord) => {
                        // Получить растение по координатам
                        if let AnimalInCell::Animal(herb) = self.landscape[coord.0][coord.1].animal {
                            let herb = Self::get_agent_mut(herb);

                            if herb.get_type() == AnimaType::Carnivore {
                                panic!("Хищник хочет съесть хищника!");
                            }

                            animal.eat_action(herb.be_eaten());
                        }
                    }
                    None => {
                        // Есть нечего: животное ошиблось.
                    }
                }
            }
        }
    }

    /// Метод находит растение в области, точки которой переданы срезом.
    ///
    /// # Arguments
    ///
    /// * `x`, `y`: Координаты относительно которой берутся смещения из области.
    /// * `area`: Область смещения.
    ///
    /// returns: Option<(usize, usize)>
    fn choose_plant(&self, x: usize, y: usize, area: &[(i8, i8)]) -> Option<(usize, usize)> {
        // Отсортируем срез случайным образом, что бы получить случайное растение,
        // если их несколько в ближайшей области.
        let area = randomize_coord_vector(Vec::from(area));

        for offset in area {
            let x_off = Self::clip(x as isize + offset.0 as isize, self.width);
            let y_off = Self::clip(y as isize + offset.1 as isize, self.height);

            if let PlantInCell::Plant(_) = self.landscape[x_off][y_off].plant {
                return Some((x_off, y_off));
            }
        }

        None
    }

    /// Метод находит животное в области, точки которой переданы срезом.
    ///
    /// # Arguments
    ///
    /// * `animal_type`: Тип животного которое мы ищем.
    /// * `x`, `y`: Координаты относительно которой берутся смещения из области.
    /// * `area`: Область смещения.
    ///
    /// returns: Option<(usize, usize)>
    fn choose_animal(
        &self,
        animal_type: AnimaType,
        x: usize,
        y: usize,
        area: &[(i8, i8)]
    ) -> Option<(usize, usize)> {
        // Отсортируем срез случайным образом, что бы получить случайное животное,
        // если их несколько в ближайшей области.
        let area = randomize_coord_vector(Vec::from(area));

        for offset in area {
            let x_off = Self::clip(x as isize + offset.0 as isize, self.width);
            let y_off = Self::clip(y as isize + offset.1 as isize, self.height);

            // В точке есть животное
            if let AnimalInCell::Animal(animal) = self.landscape[x_off][y_off].animal {
                // Проверим тип животного
                let animal = Self::get_agent_ref(animal);
                if animal.get_type() == animal_type {
                    return Some((x_off, y_off));
                }
            }
        }

        None
    }

    /// Метод реализует размножение животного.
    ///
    /// An animal has reached the energy level needed for reproduction. An animal
    /// is only permitted to reproduce if space is available for the new animal.
    /// The child animal is a copy of the parent, except that one of the weights
    /// of the neural network of his brain is mutated.
    ///
    /// # Arguments
    ///
    /// * `animal`: Изменяемая ссылка на животное.
    ///
    /// returns: ()
    fn reproduce_animal_action(&mut self, animal: &mut dyn AnimalAlive) {
        let agent_type = if animal.get_type() == AnimaType::Herbivore {
            AgentType::Herbivore
        } else {
            AgentType::Carnivore
        };

        let spot = self.find_empty_spot(agent_type);

        match spot {
            // Нашлось место для размножения.
            Ok(coord) => {
                let child = animal.reproduce_action();
                let generation = child.get_generation();

                self.add_animal(coord.0, coord.1, child)
                    .expect("Внутренняя ошибка программы: найденное место для животного уже занято");

                match animal.get_type() {
                    AnimaType::Herbivore => {
                        self.animal_reproductions.0 += 1;
                        if self.animal_max_generation.0 < generation {
                            self.animal_max_generation.0 = generation;
                        }
                    }
                    AnimaType::Carnivore => {
                        self.animal_reproductions.1 += 1;
                        if self.animal_max_generation.1 < generation {
                            self.animal_max_generation.1 = generation;
                        }
                    }
                }
            }
            // Если нет возможности размножится, ничего не делаем.
            Err(_) => {
                return;
            }
        }
    }

    /// Действие - нет действия.
    fn inactivity_animal_action(&mut self, animal: &mut dyn AnimalAlive) {
        animal.inactivity_action();
    }

    /// Завершающая обработка.
    /// Удаляем мертвых животных из среды обитания, обновляем статистику,
    /// определяем элементы для отображения, очищаем состояние животных.
    fn final_processing(&mut self) {
        // Очистим текущее состояние ячейки.
        self.view_state.clear();

        for x in 0..self.width {
            for y in 0..self.height {
                let mut tmp_view: Vec<CellStuff> = Vec::with_capacity(CellStuff::None as usize);

                // Если в точке есть растение
                if let PlantInCell::Plant(_) = self.landscape[x][y].plant {
                    tmp_view.push(CellStuff::Plant);
                }

                // Если в точке есть животное.
                if let AnimalInCell::Animal(ptr) = self.landscape[x][y].animal {
                    let animal = Self::get_agent_mut(ptr);

                    // Мир жестокое место, и если животное не справилось его место в раю.
                    // If energy falls to or below zero, the animal dies. Otherwise, we
                    // check to see if the agent has lived longer than any other agent
                    // of the particular type.
                    if animal.is_dead() {
                        // Отправляем животное в рай.
                        self.send_to_heaven(ptr, x, y);

                        if animal.is_eaten() {
                            tmp_view.push(CellStuff::KilledAnimal);
                        } else {
                            tmp_view.push(CellStuff::DeadAnimal);
                        }
                    } else {
                        // Очищаем состояние животного.
                        animal.clear();
                        // Обновляем статистику.
                        self.update_best_animal(ptr);

                        let stuff = match animal.get_type() {
                            AnimaType::Herbivore => match animal.get_direction() {
                                AnimalDirection::North => CellStuff::HerbBack,
                                AnimalDirection::South => CellStuff::HerbFront,
                                AnimalDirection::West => CellStuff::HerbLeft,
                                AnimalDirection::East => CellStuff::HerbRight,
                            },
                            AnimaType::Carnivore => match animal.get_direction() {
                                AnimalDirection::North => CellStuff::CarnBack,
                                AnimalDirection::South => CellStuff::CarnFront,
                                AnimalDirection::West => CellStuff::CarnLeft,
                                AnimalDirection::East => CellStuff::CarnRight,
                            },
                        };

                        tmp_view.push(stuff);
                    }
                }

                // После сбора того, что могло произойти в ячейке
                // следует упорядочить события по важности.
                tmp_view.sort();
                // Добавляем состояние ячейки в массив отображения.
                match tmp_view.first() {
                    Some(stuff) => {
                        self.view_state.push((x, y, *stuff));
                    }
                    _ => {}
                }
            }
        }
    }

    /// Метод "очищает" мир от умершего животного.
    ///
    /// # Arguments
    ///
    /// * `animal_ptr`: Изменяемый *указатель* на умершее животное.
    /// * `x`, `y`: Координаты умершего животного.
    ///
    /// returns: ()
    fn send_to_heaven(&mut self, animal_ptr: *mut dyn AnimalAlive, x: usize, y: usize) {
        // Death came to this animal (or it was eaten)...
        // Удаляем животное из ячейки.
        self.landscape[x][y].animal = AnimalInCell::None;
        // Помещаем указатель на животное в "рай". Указатель копируемый тип.
        self.dead_animals.push(animal_ptr);

        // Получим изменяемую ссылку на агента.
        let animal = Self::get_agent_mut(animal_ptr);

        match animal.get_type() {
            AnimaType::Herbivore => {
                self.animal_count.0 -= 1;
                self.animal_deaths.0 += 1;

                match self.best_death_animal.0 {
                    AnimalInCell::Animal(best_death_animal_ptr) => {
                        // Т.к. в этой ячейке точно не может быть текущего агента,
                        // текущий только что умер... Получим ссылку на лучшего агента.
                        let best_death_animal =  Self::get_agent_ref(best_death_animal_ptr);

                        // Только что умерший агент жил дольше всех.
                        if animal.get_age() > best_death_animal.get_age() {
                            self.best_death_animal.0 = AnimalInCell::Animal(animal_ptr);
                        }
                    }
                    _ => {}
                }
            }
            AnimaType::Carnivore => {
                self.animal_count.1 -= 1;
                self.animal_deaths.1 += 1;

                match self.best_death_animal.1 {
                    AnimalInCell::Animal(best_death_animal_ptr) => {
                        // Т.к. в этой ячейке точно не может быть текущего агента,
                        // текущий только что умер... Получим ссылку на лучшего агента.
                        let best_death_animal =  Self::get_agent_ref(best_death_animal_ptr);

                        // Только что умерший агент жил дольше всех.
                        if animal.get_age() > best_death_animal.get_age() {
                            self.best_death_animal.1 = AnimalInCell::Animal(animal_ptr);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Обновляет информацию о лучшем животном (живущем дольше всех).
    fn update_best_animal(&mut self, animal_ptr: *mut dyn AnimalAlive) {
        let animal =  Self::get_agent_ref(animal_ptr);
        match animal.get_type() {
            AnimaType::Herbivore => {
                // Получим текущее лучшее животное
                if let AnimalInCell::Animal(ptr) = self.best_animal.0 {
                    let best_animal = Self::get_agent_ref(ptr);

                    if animal.get_age() > best_animal.get_age() {
                        self.best_animal.0 = AnimalInCell::Animal(animal_ptr);
                    }
                }
            }
            AnimaType::Carnivore => {
                // Получим текущее лучшее животное
                if let AnimalInCell::Animal(ptr) = self.best_animal.1 {
                    let best_animal = Self::get_agent_ref(ptr);

                    if animal.get_age() > best_animal.get_age() {
                        self.best_animal.1 = AnimalInCell::Animal(animal_ptr);
                    }
                }
            }
        }
    }
}