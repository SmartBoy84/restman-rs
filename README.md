# restman
Library helper utilities for creating Rust bindings around a REST API.  
# Implementing
For example, consider implementing the API path `/v1/location/{location_id}/company/{company_id}/employee/{employee_id}/position`. 
## RequestPart
Everything prior to `position` is a `RequestPart` and must be defined using the `request_part!` macro. 

As some parts encode a configuration paramter (e.g., `location`), the macro has two forms:  
```request_part!(<struct name>, <serialised name>, <next part>)```  
```request_part!(<struct name>, <serialised name>, <config trait>, <config getter>)```   

So to implement `v1`,
```rust
request_part!(V1, "v1", ())
// let v1 = <V1>::default();
```
> Use `()` if nothing proceeds it - it is not possible to encode a URL without `()` terminating it.  

Implementing `location` is more complicated however,
```rust
trait HasLocationID {
    fn location_id(&self) -> &str;
}

struct Config {location_id: String};
impl HasLocationID for Config {
    fn location_id(&self) -> &str {
        &self.location_id
    }
}

impl restman_rs::request::RequestConfig for Config {} // needed

request_part!(Location, "location", V1, HasLocationID, location_id);
```
> It is possible to implement a single Config struct to hold all the possible states for a given API (e.g., location id present, but employee id isn't etc)
> 
> Refer to [`WorkjamRequestConfig`](https://github.com/SmartBoy84/workjam-rs/blob/main/src/config.rs) - my implementation can be further simplified using the `Bon` crate  
## Endpoint
# Example
For an indepth example and explanation, refer to [workjam-rs](https://github.com/SmartBoy84/workjam-rs)
