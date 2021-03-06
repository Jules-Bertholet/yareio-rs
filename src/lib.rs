#![doc = include_str!("../README.md")]

#[cfg(all(
    any(
        all(feature = "circles", any(feature = "squares", feature = "triangles")),
        all(feature = "squares", feature = "triangles")
    ),
    not(doc)
))]
compile_error!(
    r#"Only one of the features "circles", "squares", and "triangles" can be enabled at the same time"#
);

#[macro_use]
mod macros {
    macro_rules! try_can_from {
        (impl TryFrom<$from:ident>, Error = $error:ident for $t:ty) => {
            impl TryFrom<$from> for $t {
                type Error = $error;

                #[inline]
                fn try_from(value: $from) -> Result<Self, Self::Error> {
                    if <$t as CanFrom<$from>>::can_from(&value) {
                        Ok(wasm_bindgen::JsCast::unchecked_into(value))
                    } else {
                        Err(wasm_bindgen::JsCast::unchecked_into(value))
                    }
                }
            }

            impl<'a> TryFrom<&'a $from> for &'a $t {
                type Error = &'a $error;

                #[inline]
                fn try_from(value: &'a $from) -> Result<Self, Self::Error> {
                    if <$t as CanFrom<$from>>::can_from(value) {
                        Ok(wasm_bindgen::JsCast::unchecked_ref(value))
                    } else {
                        Err(wasm_bindgen::JsCast::unchecked_ref(value))
                    }
                }
            }
        };
    }
}

pub mod base;
pub mod console;
pub mod graphics;
pub mod memory;
pub mod outpost;
pub mod players;
pub mod spirit;
pub mod star;

#[cfg(feature = "RenderService")]
pub mod render_service;

use js_sys::{Array, Float64Array, JsString, Object, Reflect};
use players::PlayerID;
use spirit::{DeadSpirit, LivingEnemySpiritID, OperableSpiritID};
use std::{convert::TryFrom, fmt::Debug, marker::PhantomData, ops::Deref};
use wasm_bindgen::{prelude::*, JsCast};
/// The most useful items to import.
pub mod prelude {
    pub use crate::base::{base, bases, enemy_base, Base};
    pub use crate::outpost::{outpost_mdo, outposts, Outpost};
    pub use crate::players::this_player_id;
    pub use crate::spirit::{
        my_spirits, spirits, DeadFriendlySpirit, DeadFriendlySpiritID, LivingEnemySpirit,
        LivingEnemySpiritID, LivingFriendlySpirit, LivingFriendlySpiritID, OperableSpirit,
        OperableSpiritID, Spirit, SpiritID,
    };
    pub use crate::star::{star_a1c, star_p89, star_zxq, stars, Star};
    pub use crate::{
        console, graphics, log, tick, Destructible, Entity, EntityID, EnumerateByID, GetByID,
        OutpostSight, Position, Shape, Sight, TryGetByID,
    };

    #[cfg(feature = "RenderService")]
    pub use crate::render_service;
}

pub(crate) trait CanFrom<S: JsCast>: JsCast {
    fn can_from(value: &S) -> bool;
}

/// A position on the game board. Ordered pair of [`f64`].
pub struct Position(f64, f64);

impl From<Position> for Vec<f64> {
    #[inline]
    fn from(pos: Position) -> Self {
        vec![pos.0, pos.1]
    }
}

impl From<Vec<f64>> for Position {
    #[inline]
    fn from(vec: Vec<f64>) -> Self {
        Position(vec[0], vec[1])
    }
}

impl From<Position> for (f64, f64) {
    #[inline]
    fn from(pos: Position) -> Self {
        (pos.0, pos.1)
    }
}

impl From<(f64, f64)> for Position {
    #[inline]
    fn from(tuple: (f64, f64)) -> Self {
        Position(tuple.0, tuple.1)
    }
}

impl From<Position> for [f64; 2] {
    #[inline]
    fn from(pos: Position) -> Self {
        [pos.0, pos.1]
    }
}

impl From<[f64; 2]> for Position {
    #[inline]
    fn from(array: [f64; 2]) -> Self {
        Position(array[0], array[1])
    }
}

impl From<Position> for Array {
    #[inline]
    fn from(pos: Position) -> Self {
        let float_arr = Float64Array::new_with_length(2);
        float_arr.copy_from(&Vec::from(pos).into_boxed_slice());
        Array::from(float_arr.as_ref())
    }
}

impl wasm_bindgen::convert::IntoWasmAbi for Position {
    type Abi = <Vec<f64> as wasm_bindgen::convert::IntoWasmAbi>::Abi;

    #[inline]
    fn into_abi(self) -> Self::Abi {
        <Vec<f64>>::from(self).into_abi()
    }
}

impl wasm_bindgen::convert::OptionIntoWasmAbi for Position {
    #[inline]
    fn none() -> wasm_bindgen::convert::WasmSlice { wasm_bindgen::convert::WasmSlice { ptr: 0, len: 0 } }
}

impl wasm_bindgen::convert::FromWasmAbi for Position {
    type Abi = <Box<[f64]> as wasm_bindgen::convert::FromWasmAbi>::Abi;

    #[inline]
    unsafe fn from_abi(js: Self::Abi) -> Self {
        let slice = <Box<[f64]>>::from_abi(js);
        Position(slice[0], slice[1])
    }
}

impl wasm_bindgen::convert::OptionFromWasmAbi for Position {
    #[inline]
    fn is_none(abi: &wasm_bindgen::convert::WasmSlice) -> bool { abi.ptr == 0 }
}

impl wasm_bindgen::describe::WasmDescribe for Position {
    fn describe() {
        <Box<[f64]>>::describe();
    }
}

/// The possible values of a spirit or base's [`shape`](Destructible::shape) property.
#[wasm_bindgen(typescript_type = "Shape")]
pub enum Shape {
    Circles = "circles",
    Squares = "squares",
    Triangles = "triangles",
}

// OutpostSight
#[wasm_bindgen]
extern "C" {
    /// The [`sight`](Outpost::sight) of an outpost.
    ///
    /// [Yare.io Documentation](https://yare.io/documentation#doc_outpost)
    #[wasm_bindgen(extends = Object, typescript_type = "OutpostSight")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type OutpostSight;

    #[wasm_bindgen(method, getter)]
    pub fn enemies(this: &OutpostSight) -> Vec<LivingEnemySpiritID>;
}

// Sight
#[wasm_bindgen]
extern "C" {
    /// The [`sight`](Destructible::sight) of a spirit or base.
    ///
    /// [Yare.io Documentation](https://yare.io/documentation)
    #[wasm_bindgen(extends = OutpostSight, typescript_type = "Sight")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Sight;

    #[wasm_bindgen(method, getter)]
    pub fn friends(this: &Sight) -> Vec<OperableSpiritID>;

    #[wasm_bindgen(method, getter)]
    pub fn friends_beamable(this: &Sight) -> Vec<OperableSpiritID>;

    #[wasm_bindgen(method, getter)]
    pub fn enemies_beamable(this: &Sight) -> Vec<LivingEnemySpiritID>;

    #[wasm_bindgen(method, getter)]
    pub fn structures(this: &Sight) -> Vec<StructureID>;
}

// Entity
#[wasm_bindgen]
extern "C" {
    /// The ID of an [`Entity`](crate::Entity).
    #[wasm_bindgen(extends = JsString, typescript_type = "EntityID")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type EntityID;

    /// Any object potentially on the game board: can be a [`Spirit`](crate::spirit::Spirit), [`Base`](crate::structure::base::Base), [`Outpost`](crate::structure::outpost::Outpost), or [`Star`](crate::structure::star::Star).
    ///
    /// [Yare.io Documentation](https://yare.io/documentation)
    #[wasm_bindgen(extends = Object, typescript_type = "Entity")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Entity;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &Entity) -> EntityID;

    #[wasm_bindgen(method, getter)]
    pub fn position(this: &Entity) -> Position;

    #[wasm_bindgen(method, getter)]
    pub fn size(this: &Entity) -> u32;

    #[wasm_bindgen(method, getter)]
    pub fn energy(this: &Entity) -> i32;

    #[wasm_bindgen(method, getter)]
    fn _last_energized(this: &Entity) -> JsString;

    #[wasm_bindgen(method, getter)]
    pub fn energy_capacity(this: &Entity) -> i32;
}

impl Entity {
    #[inline]
    pub fn last_energized(&self) -> Option<EntityID> {
        let jsval = self._last_energized();
        if jsval.is_falsy() {
            None
        } else {
            Some(jsval.unchecked_into())
        }
    }
}

// LivingEntity
#[wasm_bindgen]
extern "C" {
    /// The ID of a [`LivingEntity`].
    #[wasm_bindgen(extends = EntityID, typescript_type = "EntityID")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type LivingEntityID;

    /// Any object on the game board (not destroyed): can be a [`Spirit`](crate::spirit::Spirit), [`Base`](crate::structure::base::Base), [`Outpost`](crate::structure::outpost::Outpost), or [`Star`](crate::structure::star::Star).
    ///
    /// [Yare.io Documentation](https://yare.io/documentation)
    #[wasm_bindgen(extends = Entity, typescript_type = "Entity")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type LivingEntity;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &LivingEntity) -> LivingEntityID;

}

impl CanFrom<Entity> for LivingEntity {
    #[inline]
    fn can_from(value: &Entity) -> bool {
        !Reflect::has(value, &"hp".into()).unwrap()
            || value.unchecked_ref::<Destructible>().hp() > 0
    }
}

try_can_from!(impl TryFrom<Entity>, Error = DeadSpirit for LivingEntity);

// Destructible
#[wasm_bindgen]
extern "C" {
    /// The ID of a [`Destructible`].
    #[wasm_bindgen(extends = EntityID, typescript_type = "EntityID")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type DestructibleID;

    /// Any [`Entity`](crate::Entity) that can potentially be destroyed: can be a [`Spirit`](crate::spirit::Spirit) or [`Base`](crate::structure::base::Base).
    ///
    /// [Yare.io Documentation](https://yare.io/documentation)
    #[wasm_bindgen(extends = Entity, typescript_type = "Destructible")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Destructible;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &Destructible) -> DestructibleID;

    #[wasm_bindgen(method, getter)]
    pub fn hp(this: &Destructible) -> i32;

    #[wasm_bindgen(method, getter)]
    pub fn sight(this: &Destructible) -> Sight;

    #[wasm_bindgen(method, getter)]
    pub fn player_id(this: &Destructible) -> PlayerID;

    #[wasm_bindgen(method, getter)]
    pub fn shape(this: &Destructible) -> Shape;

    #[wasm_bindgen(method, getter)]
    pub fn color(this: &Destructible) -> String;
}

impl CanFrom<Entity> for Destructible {
    #[inline]
    fn can_from(value: &Entity) -> bool {
        Reflect::has(value, &"hp".into()).unwrap()
    }
}

try_can_from!(impl TryFrom<Entity>, Error = Entity for Destructible);

// LivingDesctructible
#[wasm_bindgen]
extern "C" {
    /// The ID of a [`LivingDestructible`].
    #[wasm_bindgen(extends = DestructibleID, extends = LivingEntityID, typescript_type = "EntityID")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type LivingDestructibleID;

    /// Any [`Destructible`] that has not yet been destroyed.
    ///
    /// [Yare.io Documentation](https://yare.io/documentation)
    #[wasm_bindgen(extends = Destructible, extends = LivingEntity, typescript_type = "Destructible")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type LivingDestructible;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &LivingDestructible) -> LivingDestructibleID;
}

impl CanFrom<Destructible> for LivingDestructible {
    #[inline]
    fn can_from(value: &Destructible) -> bool {
        value.hp() > 0
    }
}

try_can_from!(impl TryFrom<Destructible>, Error = Destructible for LivingDestructible);

impl CanFrom<LivingEntity> for LivingDestructible {
    #[inline]
    fn can_from(value: &LivingEntity) -> bool {
        Reflect::has(value, &"hp".into()).unwrap()
    }
}

try_can_from!(impl TryFrom<LivingEntity>, Error = LivingEntity for LivingDestructible);

/// The possible [`structure_type`](Structure::structure_type)s.
///
/// [Yare.io Documentation](https://yare.io/documentation)
#[wasm_bindgen(typescript_type = "StructureType")]
pub enum StructureType {
    Base = "base",
    Outpost = "outpost",
    Star = "star",
}

// Structure
#[wasm_bindgen]
extern "C" {
    /// The ID of a [`Structure`].
    #[wasm_bindgen(extends = LivingEntityID, typescript_type = "StructureID")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type StructureID;

    /// A structure, i.e. anything with a [`structure_type`](Structure::structure_type): can be a [`Base`], [`Outpost`], or [`Star`].
    ///
    /// [Yare.io Documentation](https://yare.io/documentation)
    #[wasm_bindgen(extends = LivingEntity, typescript_type = "Structure")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Structure;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &Structure) -> StructureID;

    #[wasm_bindgen(method, getter)]
    pub fn structure_type(this: &Structure) -> StructureType;

    #[wasm_bindgen(method, getter)]
    pub fn collision_radius(this: &Structure) -> f64;
}

impl CanFrom<Entity> for Structure {
    #[inline]
    fn can_from(value: &Entity) -> bool {
        Reflect::has(value, &"structure_type".into()).unwrap()
    }
}

try_can_from!(impl TryFrom<Entity>, Error = Entity for Structure);

impl CanFrom<LivingEntity> for Structure {
    #[inline]
    fn can_from(value: &LivingEntity) -> bool {
        <Structure as CanFrom<Entity>>::can_from(value)
    }
}

try_can_from!(impl TryFrom<LivingEntity>, Error = Entity for Structure);

// GetById

/// This trait is implemented for the global objects that give mappings of [ID](EntityID)s to entities:
/// [`spirits`](spirit::spirits), [`bases`](structure::base::bases),
/// [`outposts`](structure::outpost::outposts), and [`stars`](structure::star::stars).
/// It allows fetching entities by their ID.
pub trait GetByID<ID: JsCast, V: JsCast>
where
    Self: AsRef<JsValue> + Deref<Target = Object>,
{
    /// Returns the value for this key.
    fn get(&self, id: &ID) -> V {
        Reflect::get(self.as_ref(), id.as_ref())
            .ok()
            .map(JsCast::unchecked_into)
            .unwrap()
    }
}

/// This trait is implemented for the global objects that give mappings of [ID](EntityID)s to entities:
/// [`spirits`](spirit::spirits), [`bases`](structure::base::bases),
/// [`outposts`](structure::outpost::outposts), and [`stars`](structure::star::stars).
/// It allows faillibly fetching entities by their ID.
pub trait TryGetByID<ID: JsCast, V: JsCast>
where
    Self: AsRef<JsValue> + Deref<Target = Object>,
{
    /// Returns the value for this key.
    fn get(&self, id: &ID) -> Option<V> {
        Reflect::get(self.as_ref(), id.as_ref())
            .ok()
            .map(JsCast::unchecked_into)
    }
}

impl<ID: JsCast, V: JsCast, T: GetByID<ID, V>> TryGetByID<ID, V> for T {}
/// This trait is implemented for the global objects that give mappings of [ID](EntityID)s to entities:
/// [`spirits`](spirit::spirits), [`bases`](structure::base::bases),
/// [`outposts`](structure::outpost::outposts), and [`stars`](structure::star::stars).
/// It allows iterating over entities by their ID.
pub trait EnumerateByID<ID: JsCast, V: JsCast>: GetByID<ID, V> {
    // An iterator visiting all IDs.
    fn ids(&self) -> ArrayTypedIter<ID> {
        let array = Object::keys(self);
        ArrayTypedIter::<ID> {
            range: 0..array.length(),
            array,
            phantom: PhantomData,
        }
    }

    // An iterator visiting all values.
    fn values(&self) -> ArrayTypedIter<V> {
        let array = Object::values(self);
        ArrayTypedIter::<V> {
            range: 0..array.length(),
            array,
            phantom: PhantomData,
        }
    }
}

/// Iterator returned by [`EnumerateByID`]'s methods.
pub struct ArrayTypedIter<T: JsCast> {
    range: std::ops::Range<u32>,
    array: Array,
    phantom: PhantomData<T>,
}

impl<T: JsCast> std::iter::Iterator for ArrayTypedIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.range.next()?;
        Some(self.array.get(index).unchecked_into())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<T: JsCast> std::iter::DoubleEndedIterator for ArrayTypedIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.range.next_back()?;
        Some(self.array.get(index).unchecked_into())
    }
}
impl<T: JsCast> std::iter::FusedIterator for ArrayTypedIter<T> {}

impl<T: JsCast> std::iter::ExactSizeIterator for ArrayTypedIter<T> {}

// `tick`
#[wasm_bindgen]
extern "C" {
    /// `tick` (the number of ticks since the start of the game).

    #[wasm_bindgen]
    pub static tick: u32;
}
