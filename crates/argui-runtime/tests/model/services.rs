use argui_runtime::{Context, ModelRuntime, Render, ResourceScope, ServiceAlreadyRegistered};
use std::{cell::Cell, rc::Rc};

struct View;

#[test]
fn registration_outlives_its_registry_without_retaining_it() {
    let registration = ModelRuntime::default().register_service(42_u32).unwrap();
    let consumer = registration.service();
    assert_eq!(*consumer, 42);
    drop(registration);
    assert_eq!(*consumer, 42);
    assert_eq!(Rc::strong_count(&consumer), 1);
}
impl Render for View {
    fn render(&mut self, _: &mut Context<Self>) -> argui_ui::Element {
        argui_ui::Element::text("services")
    }
}

#[test]
fn services_are_typed_shared_and_isolated_between_domains() {
    let runtime = ModelRuntime::default();
    let other = ModelRuntime::default();
    assert!(runtime.service::<Cell<usize>>().is_none());
    let registration = runtime.register_service(Cell::new(1_usize)).unwrap();
    let service = registration.service();
    let model = runtime.entity(View);
    let first = model.mount().unwrap();
    let second = model.mount().unwrap();
    first
        .update(|_, cx| cx.service::<Cell<usize>>().unwrap().set(2))
        .unwrap();
    second
        .update(|_, cx| assert!(Rc::ptr_eq(&cx.service::<Cell<usize>>().unwrap(), &service)))
        .unwrap();
    model.update(|_, cx| assert_eq!(cx.service::<Cell<usize>>().unwrap().get(), 2));
    assert!(other.service::<Cell<usize>>().is_none());
    assert!(runtime.service::<String>().is_none());
    assert!(matches!(
        runtime.register_service(Cell::new(3_usize)),
        Err(ServiceAlreadyRegistered)
    ));
    assert_eq!(
        ServiceAlreadyRegistered.to_string(),
        "a service of this type is already registered"
    );
    assert_eq!(runtime.service::<Cell<usize>>().unwrap().get(), 2);
    first.close();
    assert!(runtime.service::<Cell<usize>>().is_some());
    drop(registration);
    assert!(runtime.service::<Cell<usize>>().is_none());
    assert_eq!(service.get(), 2);
    let replacement = runtime.register_service(Cell::new(4_usize)).unwrap();
    assert_eq!(replacement.service().get(), 4);
    assert!(!Rc::ptr_eq(&replacement.service(), &service));
}

#[test]
fn service_publication_can_be_owned_by_an_application_scope() {
    let runtime = ModelRuntime::default();
    let scope = ResourceScope::default();
    let registration = runtime
        .register_service(String::from("mail store"))
        .unwrap();
    let weak = Rc::downgrade(&registration.service());
    let lease = scope.own(registration).unwrap();
    assert!(runtime.service::<String>().is_some());
    scope.close();
    assert!(runtime.service::<String>().is_none());
    assert!(weak.upgrade().is_none());
    drop(lease);
}

#[test]
fn registry_does_not_retain_a_service_that_itself_retains_models() {
    let runtime = ModelRuntime::default();
    let model = runtime.entity(7_usize);
    let weak = model.downgrade();
    let registration = runtime.register_service(model).unwrap();
    drop(runtime);
    assert!(weak.upgrade().is_some());
    drop(registration);
    assert!(weak.upgrade().is_none());
    assert!(Context::<()>::default().service::<String>().is_none());
}
