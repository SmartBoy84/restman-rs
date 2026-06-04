# Bon macro
> The *code* was AI generated, but the following write-up and the actual idea for this is *completely* human!

It is a proc-macro which simplifies the type-state config pattern using Bon that I implement manually in [hecko](https://github.com/SmartBoy84/hecko). It's important to note that this type-state config pattern is itself a simplification of the fully manual type-state implementation (no crate) that I do in [workjam-rs]().  

In both of the following cases, if the field name is `field_name` then the trait is `HasFieldName` and the getter is `field_name`. So a request part using it would be written as `request_part!(PartName, "part_name", (), HasFieldName, field_name);`. Macro magic! 

So, for example, the [`config.rs`](https://github.com/SmartBoy84/hecko/blob/main/src/echo/config.rs) can be expressed simply using the `#[bon_config]` directive:
```rust
#[bon_macro::bon_config]
pub struct EchoRequestConfig<'a> {
    section_id: Cow<'a, str>,
    lesson_descriptor: Cow<'a, str>,
    media_id: Cow<'a, str>,
}
```  

This "desugars"/expands to (src; [`config.rs`](https://github.com/SmartBoy84/hecko/blob/main/src/echo/config.rs)):
```rust
#[derive(Debug, bon::Builder)]
#[builder(builder_type(name = EchoRequestConfig, vis = "pub"), finish_fn(vis = ""))]
struct _EchoRequestConfig<'a> {
    #[builder(into, getter(name = get_section_id_internal, vis = ""))]
    section_id: Cow<'a, str>,

    #[builder(into, getter(name = get_lesson_descriptor_internal, vis = ""))]
    lesson_descriptor: Cow<'a, str>,

    #[builder(into, getter(name = get_media_id_internal, vis = ""))]
    media_id: Cow<'a, str>,
}

impl<'a, S: echo_request_config::State> RequestConfig for EchoRequestConfig<'a, S> {}

impl<'a> EchoRequestConfig<'a, echo_request_config::Empty> {
    pub fn new() -> Self {
        _EchoRequestConfig::builder()
    }
}

impl<'a> HasSectionID for EchoRequestConfig<'a, echo_request_config::SetSectionId> {
    fn section_id(&self) -> &str {
        self.get_section_id_internal().as_ref()
    }
}

impl<'a> HasLessonDescriptor for EchoRequestConfig<'a, echo_request_config::SetLessonDescriptor> {
    fn lesson_descriptor(&self) -> &str {
        self.get_lesson_descriptor_internal().as_ref()
    }
}

impl<'a> HasMediaID for EchoRequestConfig<'a, echo_request_config::SetMediaId> {
    fn media_id(&self) -> &str {
        self.get_media_id_internal().as_ref()
    }
}
```