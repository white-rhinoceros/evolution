use crate::landscape::Energy;

pub mod brains;
pub mod species;

/// Возможные виды животных.
#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub enum AnimaType {
    Herbivore,
    Carnivore,
}

/// Возможные действия для животного.
/// Действия, связанные с конфликтами внутри группы пока не рассматриваем!
#[derive(Copy, Clone)]
pub enum AnimalAction {
    TurnLeft,     // Повернуть на лево (агент остается на месте).
    TurnRight,    // Повернуть на право (агент остается на месте).
    Move,         // Сделать шаг вперед.
    Eat,          // Попытаться съесть агента в области близости.
    Reproduce,    // Размножение.
    None,         // Нет действия (животное что-то ждет).
}
const MAX_ACTIONS: usize = 6;

/// Перечисление, определяющее текущие направление животного.
#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub enum AnimalDirection {
    North, South, West, East
}

/// Структура для передачи значений входных ячеек сенсоров.
#[derive(Copy, Clone)]
pub struct AnimalInputSignal {
    pub plant_front: usize,           // Растение на переднем плане
    pub plant_left: usize,            // Растение слева
    pub plant_right: usize,           // Растение справа
    pub plant_proximity: usize,       // Растение поблизости

    pub herbivore_front: usize,       // Травоядное на переднем плане
    pub herbivore_left: usize,        // Травоядное слева
    pub herbivore_right: usize,       // Травоядное справа
    pub herbivore_proximity: usize,   // Травоядное поблизости

    pub carnivore_front: usize,       // Хищник на переднем плане.
    pub carnivore_left: usize,        // Хищник слева.
    pub carnivore_right: usize,       // Хищник справа.
    pub carnivore_proximity: usize,   // Хищник поблизости.
}

/// Типаж, определяющий животное.
pub trait AnimalAlive {
    // Методы получения состояния животного.

    /// Мертвое ли?
    fn is_dead(&self) -> bool;

    /// Было ли животное съедено?
    fn is_eaten(&self) -> bool;

    /// Признак того, что на текущей итерации животное уже "совершило свой ход".
    fn is_processed(&self) -> bool;

    /// Возвращает тип животного.
    fn get_type(&self) -> AnimaType;

    /// Возвращает текущее направление движения животного.
    fn get_direction(&self) -> AnimalDirection;

    /// Возвращает возраст животного в итерациях.
    fn get_age(&self) -> usize;

    /// Возвращает поколение животного.
    fn get_generation(&self) -> usize;

    /// Очищает состояние животное. Метод следует вызвать после прохода всех
    /// ячеек на текущей итерации.
    fn clear(&mut self);

    // Метод Action

    /// Активные действия животного ("желания" животного).
    /// Действие животного относительно того, что оно "видит" в текущий момент.
    /// Что именно "видит" животное определяется структурой AnimalInputSignal,
    /// разделяемая ссылка на которую передается в качестве параметров.
    fn action(&mut self, inputs: &AnimalInputSignal) -> AnimalAction;

    // Действия, которые реализуют "желания" животного. Эти методы лишь
    // изменяют внутреннее состояние животного и сами не влияют на мир.
    // Удовлетворится ли желание решает Мир по объективным причинам.

    /// Implement the turn action. Given a turn direction, the current facing
    /// is used to determine the new facing.
    fn turn_action(&mut self, turn_left: bool);

    /// Реализует желание двигаться вперед.
    fn move_action(&mut self, realized: bool);

    /// Реализует желание съесть другое животное или траву.
    /// energy - энергия полученная от съедания.
    fn eat_action(&mut self, energy: Energy);

    /// Реализует желание размножаться.
    fn reproduce_action(&mut self) -> Box<dyn AnimalAlive>;

    /// Действие - "нет действия". Животное может предпочесть оставаться на месте
    /// и ждать когда еда сама придет, экономя энергию.
    fn inactivity_action(&mut self);

    // Действия, которые можно совершить с животным против его воли.

    /// Попытка съедения животного.
    fn be_eaten(&mut self) -> Energy;
}


