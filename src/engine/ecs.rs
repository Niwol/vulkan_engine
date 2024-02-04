use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Display,
    sync::Arc,
};

use crate::{camera::Camera3D, vulkan_context::VulkanContext};

use super::material::{material_manager::MaterialManager, Material};

pub mod components;

pub type Entity = usize;

trait ComponentVec {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_entity(&self, index: usize) -> Option<Entity>;
    fn len(&self) -> usize;
    fn swap_remove(&mut self, index: usize);
    fn inner_type_id(&self) -> TypeId;
    fn inner_type_name(&self) -> &str;
}

impl<T: 'static> ComponentVec for Vec<(Entity, T)> {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn get_entity(&self, index: usize) -> Option<Entity> {
        if let Some((entity, _)) = self.get(index) {
            return Some(*entity);
        }

        None
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn swap_remove(&mut self, index: usize) {
        self.swap_remove(index);
    }

    fn inner_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn inner_type_name(&self) -> &str {
        std::any::type_name::<T>()
    }
}

pub struct Scene {
    entities: HashMap<Entity, Vec<(TypeId, usize)>>,
    component_vecs: HashMap<TypeId, Box<dyn ComponentVec>>,
    material_manager: MaterialManager,
    camera: Option<Camera3D>,

    vulkan_context: Arc<VulkanContext>,
}

impl Scene {
    pub(crate) fn new(vulkan_context: Arc<VulkanContext>) -> Self {
        Self {
            entities: HashMap::new(),
            component_vecs: HashMap::new(),
            material_manager: MaterialManager::new(Arc::clone(vulkan_context.device())),
            camera: None,

            vulkan_context,
        }
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn spawn_entity(&mut self) -> Entity {
        let entity = self.entities.len();
        self.entities.insert(entity, Vec::new());

        entity
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        assert!(
            self.entities.contains_key(&entity),
            "Scene does not contain entity {}",
            entity
        );

        while !self.entities[&entity].is_empty() {
            self.entity_remove_last_component(entity);
        }

        self.entities.remove(&entity);
    }

    fn entity_remove_last_component(&mut self, entity: Entity) {
        if let Some((type_id, index)) = self.entities.get_mut(&entity).unwrap().pop() {
            let component_vec = self.component_vecs.get_mut(&type_id).unwrap();

            component_vec.swap_remove(index);
            if index < component_vec.len() {
                let old_index = component_vec.len();
                let new_index = index;
                let entity_to_update = component_vec.get_entity(new_index).unwrap();
                self.update_entity(entity_to_update, type_id, old_index, new_index);
            }
        }
    }

    fn update_entity(
        &mut self,
        entity: Entity,
        type_id: TypeId,
        old_index: usize,
        new_index: usize,
    ) {
        for (self_type_id, index) in self.entities.get_mut(&entity).unwrap() {
            if *self_type_id == type_id && *index == old_index {
                *index = new_index;
                return;
            }
        }
    }

    pub fn entities(&self) -> Vec<&Entity> {
        self.entities.keys().collect()
    }

    pub fn entity_add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        assert!(
            self.entities.contains_key(&entity),
            "Entity {entity} does not exist in the scene"
        );

        if let Some(component_vec) = self.component_vecs.get_mut(&TypeId::of::<T>()) {
            let component_vec = component_vec
                .as_any_mut()
                .downcast_mut::<Vec<(Entity, T)>>()
                .unwrap();
            self.entities
                .get_mut(&entity)
                .unwrap()
                .push((TypeId::of::<T>(), component_vec.len()));
            component_vec.push((entity, component));
            return;
        }

        self.entities
            .get_mut(&entity)
            .unwrap()
            .push((TypeId::of::<T>(), 0));
        self.component_vecs
            .insert(TypeId::of::<T>(), Box::new(vec![(entity, component)]));
    }

    pub fn entity_components(&self, entity: Entity) -> &Vec<(TypeId, usize)> {
        assert!(
            self.entities.get(&entity).is_some(),
            "Entity {entity} does not exist in the scene"
        );

        self.entities.get(&entity).unwrap()
    }

    pub fn entity_components_mut(&mut self, entity: Entity) -> &mut Vec<(TypeId, usize)> {
        assert!(
            self.entities.get(&entity).is_some(),
            "Entity {entity} does not exist in the scene"
        );

        self.entities.get_mut(&entity).unwrap()
    }

    pub fn components<T: 'static>(&self) -> Option<&Vec<(Entity, T)>> {
        if let Some(component_vec) = self.component_vecs.get(&TypeId::of::<T>()) {
            component_vec.as_any().downcast_ref::<Vec<(Entity, T)>>()
        } else {
            None
        }
    }

    pub fn components_mut<T: 'static>(&mut self) -> Option<&mut Vec<(Entity, T)>> {
        if let Some(component_vec) = self.component_vecs.get_mut(&TypeId::of::<T>()) {
            component_vec
                .as_any_mut()
                .downcast_mut::<Vec<(Entity, T)>>()
        } else {
            None
        }
    }

    pub(crate) fn material_manager(&self) -> &MaterialManager {
        &self.material_manager
    }

    pub fn new_material<T: Material + 'static>(&mut self, material: T) -> u64 {
        self.material_manager
            .new_material(material, Arc::clone(&self.vulkan_context))
    }

    pub fn set_camera(&mut self, camera: Camera3D) {
        self.camera = Some(camera);
    }

    pub fn camera(&self) -> &Option<Camera3D> {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Option<Camera3D> {
        &mut self.camera
    }
}

impl Display for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        let mut entities = self.entities();
        entities.sort();

        s.push_str("Entities: {\n");
        for &entity in entities.into_iter() {
            let components = self
                .entity_components(entity)
                .iter()
                .map(|(type_id, index)| {
                    let component_name = self.component_vecs[type_id]
                        .inner_type_name()
                        .split(":")
                        .last()
                        .unwrap();
                    (component_name, index)
                })
                .collect::<Vec<_>>();

            s.push_str(format!("\t{}: {:?}\n", entity, components).as_str());
        }
        s.push_str("}\n");

        for (_, component_vec) in self.component_vecs.iter() {
            let mut entities = Vec::new();
            for i in 0..component_vec.len() {
                entities.push(component_vec.get_entity(i).unwrap());
            }
            let component_name = component_vec.inner_type_name().split(":").last().unwrap();

            s.push_str(format!("{}: {:?}\n", component_name, entities).as_str());
        }

        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use winit::{event_loop::EventLoop, window::WindowBuilder};

    use crate::vulkan_context::VulkanContext;

    use super::*;

    fn create_empty_scene() -> Scene {
        let dummy_window = WindowBuilder::new()
            .build(&EventLoop::new().unwrap())
            .unwrap();
        let vulkan_contex = VulkanContext::new(&Arc::new(dummy_window)).unwrap();
        Scene::new(Arc::new(vulkan_contex))
    }

    fn consistency_check(scene: &Scene) {
        let entities = scene.entities();

        for &entity in entities {
            consistency_check_entity_in_scene(scene, entity);
        }
        for (_, component_vec) in &scene.component_vecs {
            consistency_check_component_vec(scene, component_vec);
        }
    }

    fn consistency_check_entity_in_scene(scene: &Scene, entity: Entity) {
        for (type_id, index) in scene.entity_components(entity) {
            let component_vec = &scene.component_vecs.get(type_id).unwrap();
            let other = component_vec.get_entity(*index);

            assert!(
                other.is_some(),
                "The index should reference some component of type type_id"
            );
            assert_eq!(
                entity,
                other.unwrap(),
                "The component referenced by the entity should reference the same entity"
            );
        }
    }

    fn consistency_check_component_vec(scene: &Scene, component_vec: &Box<dyn ComponentVec>) {
        let entities = scene.entities();
        let len = component_vec.len();
        for i in 0..len {
            let entity = component_vec.get_entity(i).unwrap();
            assert_eq!(
                entities.iter().filter(|e| e == &&&entity).count(),
                1,
                "The component should reference exactly one entity"
            );
        }
    }

    // Entity adding and removing
    #[test]
    fn create_scene() {
        let scene = create_empty_scene();

        assert_eq!(
            scene.entity_count(),
            0,
            "The scene should count 0 entities when created"
        );
    }

    #[test]
    fn add_one_entity() {
        let mut scene = create_empty_scene();
        let e = scene.spawn_entity();

        assert!(
            scene.entities.contains_key(&e),
            "The scene should contain the new entity"
        );
        assert_eq!(scene.entity_count(), 1, "The scene should count 1 entity");
    }

    #[test]
    fn add_two_entities() {
        let mut scene = create_empty_scene();

        let e1 = scene.spawn_entity();
        let e2 = scene.spawn_entity();

        assert_eq!(e1, 0);
        assert_eq!(e2, 1);
        assert_eq!(scene.entity_count(), 2, "The scene should count 2 entity");
    }

    #[test]
    fn list_entities() {
        let mut scene = create_empty_scene();

        let e1 = scene.spawn_entity();
        let e2 = scene.spawn_entity();
        let e3 = scene.spawn_entity();

        [e1, e2, e3].iter().for_each(|e| {
            assert!(
                scene.entities().contains(&e),
                "Entity {e} is not present in the scene"
            )
        });
    }

    #[test]
    fn remove_one_entity() {
        let mut scene = create_empty_scene();
        let _e1 = scene.spawn_entity();
        let e2 = scene.spawn_entity();
        let _e3 = scene.spawn_entity();

        scene.remove_entity(e2);

        assert!(!scene.entities().contains(&&e2));
    }

    #[test]
    #[should_panic(expected = "Scene does not contain entity 666")]
    fn remove_non_existant_entity() {
        let mut scene = create_empty_scene();
        scene.remove_entity(666);
    }

    // Component tests
    #[derive(Debug, PartialEq, Eq, Hash)]
    struct Dummy1(i32);
    #[derive(Debug, PartialEq, Eq, Hash)]
    struct Dummy2(u64);

    #[test]
    fn non_existant_component() {
        let scene = create_empty_scene();
        assert_eq!(scene.components::<Dummy1>(), None);
    }

    #[test]
    fn add_one_component() {
        let mut scene = create_empty_scene();
        let e = scene.spawn_entity();
        scene.entity_add_component(e, Dummy1(42));

        let components = scene.components::<Dummy1>();
        assert!(
            components.is_some(),
            "The global component list should exist"
        );

        let components = components.unwrap();
        assert!(
            components.contains(&(e, Dummy1(42))),
            "The added component assosiated with its entity is not in the global component list"
        );

        let entity_components = scene.entity_components(e);
        assert_eq!(
            components[entity_components[0].1],
            (e, Dummy1(42)),
            "The entity does not reference the component"
        );
    }

    #[test]
    fn consistency_adding_entities_and_components() {
        let mut scene = create_empty_scene();
        let e1 = scene.spawn_entity();
        let e2 = scene.spawn_entity();
        let e3 = scene.spawn_entity();

        scene.entity_add_component(e1, Dummy1(42));
        scene.entity_add_component(e1, Dummy2(8));

        scene.entity_add_component(e2, Dummy1(-3));

        assert_eq!(
            scene.entity_count(),
            3,
            "There should be 3 entities in the scene"
        );

        let dummy1_vec = scene.components::<Dummy1>();
        assert!(
            dummy1_vec.is_some(),
            "There should be a list of Dummy1 components"
        );

        let dummy1_vec = dummy1_vec.unwrap();
        assert_eq!(
            dummy1_vec.len(),
            2,
            "There should be 2 components in the Dummy1 list"
        );

        let dummy2_vec = scene.components::<Dummy2>();
        assert!(
            dummy2_vec.is_some(),
            "There should be a list of Dummy2 components"
        );

        let dummy2_vec = dummy2_vec.unwrap();
        assert_eq!(
            dummy2_vec.len(),
            1,
            "There should be 1 component in the Dummy2 list"
        );

        let e1_components = scene.entity_components(e1);
        assert_eq!(e1_components.len(), 2, "Entiy e1 should have 2 components");
        for (type_id, index) in e1_components {
            if TypeId::of::<Dummy1>() == *type_id {
                assert_eq!(dummy1_vec[*index], (e1, Dummy1(42)));
            }

            if TypeId::of::<Dummy2>() == *type_id {
                assert_eq!(dummy2_vec[*index], (e1, Dummy2(8)));
            }
        }

        let e2_components = scene.entity_components(e2);
        assert_eq!(e2_components.len(), 1, "Entity e2 should have 1 component");
        assert_eq!(
            dummy1_vec[e2_components[0].1],
            (e2, Dummy1(-3)),
            "Entity e2 should reference its component"
        );

        let e3_components = scene.entity_components(e3);
        assert_eq!(
            e3_components.len(),
            0,
            "Entity e3 should have no components"
        );
    }

    fn construct_big_scene() -> Scene {
        let mut scene = create_empty_scene();

        let e1 = scene.spawn_entity();
        let e2 = scene.spawn_entity();
        let e3 = scene.spawn_entity();
        let e4 = scene.spawn_entity();
        let e5 = scene.spawn_entity();

        scene.entity_add_component(e1, Dummy1(1));
        scene.entity_add_component(e1, Dummy1(2));
        scene.entity_add_component(e1, Dummy1(3));
        scene.entity_add_component(e1, Dummy1(4));
        scene.entity_add_component(e1, Dummy1(5));

        scene.entity_add_component(e2, Dummy1(1));
        scene.entity_add_component(e2, Dummy1(2));
        scene.entity_add_component(e2, Dummy2(3));
        scene.entity_add_component(e2, Dummy1(4));
        scene.entity_add_component(e2, Dummy2(5));

        scene.entity_add_component(e3, Dummy2(1));
        scene.entity_add_component(e3, Dummy1(-2));
        scene.entity_add_component(e3, Dummy2(3));
        scene.entity_add_component(e3, Dummy1(-4));
        scene.entity_add_component(e3, Dummy2(5));

        scene.entity_add_component(e4, Dummy2(10));
        scene.entity_add_component(e4, Dummy2(20));
        scene.entity_add_component(e4, Dummy2(30));
        scene.entity_add_component(e4, Dummy2(40));
        scene.entity_add_component(e4, Dummy2(50));

        scene.entity_add_component(e5, 5);

        scene.spawn_entity();
        scene.spawn_entity();
        scene.spawn_entity();

        scene
    }

    #[test]
    fn consistency_check_only_adding() {
        let scene = construct_big_scene();

        consistency_check(&scene);
    }

    #[test]
    fn remove_two_entities() {
        let mut scene = create_empty_scene();

        let e1 = scene.spawn_entity();
        let e2 = scene.spawn_entity();

        scene.entity_add_component(e1, Dummy1(1));
        scene.entity_add_component(e1, Dummy1(2));
        scene.entity_add_component(e2, Dummy1(3));
        scene.entity_add_component(e2, Dummy1(4));

        println!("{}", scene);

        scene.remove_entity(e1);
        consistency_check(&scene);

        println!("{}", scene);

        scene.remove_entity(e2);
        consistency_check(&scene);
    }

    #[test]
    fn consistency_check_removing_entities() {
        let mut scene = construct_big_scene();
        println!("Full scene:");
        println!("{}", scene);

        let mut entities = scene.entities().iter().map(|e| **e).collect::<Vec<usize>>();
        entities.sort();

        println!("Removing {}", entities[1]);
        scene.remove_entity(entities[1]);
        println!("After remove of {}", entities[1]);
        println!("{}", scene);
        consistency_check(&scene);

        println!("Removing {}", entities[4]);
        scene.remove_entity(entities[4]);
        println!("After remove of {}", entities[4]);
        println!("{}", scene);
        consistency_check(&scene);

        println!("Removing {}", entities.last().unwrap());
        scene.remove_entity(*entities.last().unwrap());
        println!("After remove of {}", entities.last().unwrap());
        println!("{}", scene);

        consistency_check(&scene);
    }

    #[test]
    #[should_panic(expected = "Entity 666 does not exist in the scene")]
    fn add_component_to_non_existant_entity() {
        let mut scene = create_empty_scene();
        scene.entity_add_component(666, Dummy1(5));
    }

    #[test]
    #[should_panic(expected = "Entity 666 does not exist in the scene")]
    fn components_of_non_existing_entity() {
        let scene = create_empty_scene();
        let _ = scene.entity_components(666);
    }
}
