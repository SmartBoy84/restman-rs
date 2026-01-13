# Restman

Library helper utilities for creating Rust bindings around a REST API.

# Implementing

For example, consider implementing the API path
`/v1/company/{company_id}/employee?id=123`.

> In the following, argument(s) wrapped in [] are optional

## `RequestPart`

Everything prior to `position` is a `RequestPart` and must be defined using the
`request_part!` macro.

`request_part!(<struct name>, <serialised name>, <next part>, [<config trait>, <config getter>])`

To implement `v1`,

```rust
request_part!(V1, "v1", ())
// let v1 = <V1>::default();
```

> Use `()` if nothing proceeds it - it is not possible to encode a URL without
> `()` terminating it.

Implementing `company` is more complicated however,

```rust
trait HasCompanyID {
    fn company_id(&self) -> &str;
}

struct MyConfig {company_id: String};

impl HasCompanyID for MyConfig {
    fn company_id(&self) -> &str {
        &self.company_id
    }
}

impl restman_rs::request::RequestConfig for MyConfig {} // needed

request_part!(Company, "company", V1, HasCompanyID, company_id);
```

> It is possible to implement a single Config struct to hold all the possible
> states for a given API (e.g., location id present, but employee id isn't etc)
>
> Refer to
> [`WorkjamRequestConfig`](https://github.com/SmartBoy84/workjam-rs/blob/main/src/config.rs) -
> my implementation can be further simplified using the `Bon` crate

## `Endpoint`

The terminating part of a URL is the `Endpoint`, and must be defined separately
using the `endpoint!` macro.\
`endpoint!(<target server>, pub <endpoint name>, <serialised name>, <previous part>, <response>, <parameters>,  method = <GET | POST | PATCH | PUT>);`

To implement the `employee` endpoint above,

```rust
// first must define the target server marker struct
pub struct MyServer;
impl restman_rs::Server for MyServer {
    const ROOT: &str = "https://api.myserver.com/api"; // no trailing slash!
}

// then it's response struct
#[derive(serde::Serialize)]
struct EmployeeRes {
    name: String,
    age: u32
}

// since it takes parameters - use bon to simplify this, as parameters can be optional
#[derive(serde::Serialize)]
struct EmployeePara {
    id: String,
    employment: Option<String> // will be omitted from URI if None
}

// assume it requires a PATCH request
endpoint!(MyServer, pub Employee, "employee", Company, EmployeeRes, EmployeePara, restman_rs::PATCH);
```

> See how at this point, only `Company` was needed and no other part of the
> URL + one does not need to mention the semantics of the earlier URL such as it
> requiring a config parameter

## `ApiClient`

Finally, you are ready to make a request!

```rust
const TOKEN: &str = "my_token";

let backened = restman_rs::UreqApiHttpClient::new(restman_rs::client::AGENT); // can use your own!
let client = restman_rs::ApiClient::new(backend, TOKEN);

let config = MyConfig { company_id: "my-company" }
let para = EmployeePara {id: "my-id", employment: None };
let req = restman_rs::ApiRequest::<Employee>::new_with_para(&config, para);

let res: EmployeeRes = client.request(&req).unwrap();
```

> You can plug in your own backend, as long as it implements the
> restman_rs::client::ApiHttpClient trait

## Custom HTTP backend
The bare minimum is to implement `restman_rs::ApiHttpClient`, then depending on which request types required `restman_rs::{GET, PATCH, PUT, POST}`.  

# Why do it this way?
Consider the naive approach:
```rust
impl MyClient {
    fn employee_req(&self, company_id: &str, para: EmployeePara) -> EmployeeRes {
        let uri = format!("{ROOT}/{COMPANY}/{company_id}/{EMPLOYEE}?id={}", para.employee.id);
        // ...
    }
}
```
This quickly becomes untenable when you have a large number of endpoint, and many parts to the URL. If the API changes at any point, you have to traverse through every single endpoint and change the path everytime.  

For example, suppose the `company` part of the example URI now resides at `/v1/location/{location id}/city/{city id}/company/{company id}`, you have to change the arguments of each method (there may tens or hundreds!), and change the `format!` method!  

This is illogical though, the `employee` endpoint really only needs to know the detail that it's preceeding part is `company` - everything else should be inherited implicitly. This is what my crate solves, among other things that becomes obvious with use.  

# Example

For an indepth, real-world example, refer to
[workjam-rs](https://github.com/SmartBoy84/workjam-rs)
