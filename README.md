# Restman

Library helper utilities for creating Rust bindings around a REST API.

# Implementing

For example, consider implementing the API path
`/v1/company/{company_id}/employee?id=123` where the argument string `id=123` is
_optional_, and it takes a JSON payload via POST request.

## `RequestPart`

Everything prior to `employee` is a `RequestPart` and must be defined using the
`request_part!` macro.

`request_part!(<struct name>, <serialised name>, <next part>, [<config trait>, <config getter>])`

> Argument(s) wrapped in [] are optional

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
`endpoint!(<target server>, pub <endpoint name>, <serialised name>, <previous part>, <response>, <parameters>, <json payload>, method = <GET | POST | PATCH | PUT>);`

> Note; `json payload` argument can be make `()` if not required - this will
> alter what method user can call so that they themself don't have to pass `&[]`

To implement the `employee` endpoint above,

```rust
// first must define the target server marker struct
pub struct MyServer {
    pub server: String
};
impl restman_rs::Server for MyServer {}

// provide a default backend URL
impl restman_rs::ConstServer for MyServer {
    const ROOT: &str = "https://api.myserver.com/api"; // no trailing slash!
}

// if you want to permit users to also be able to specify their own backend (i.e., to be able to call ApiRequest::<T>::new_with_server(...))
impl restman_rs::DynamicServer for MyServer {
    fn get_root(&self) -> &str {
        &self.server
    }
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

// it takes a JSON payload as well (illustrative example - obviously not real world)
#[derive(serde::Serialize)]
struct EmployeePayload {
    greet: String // idk, greet them?
}

// since the parameters are optional, we'll indicate that - now specification won't be enforced at compile time!
impl restman_rs::request::QueryParametersOptional for EmployeePara {}

// assume it requires a PATCH request
endpoint!(MyServer, pub Employee, "employee", Company, EmployeeRes, EmployeePara, EmployeePayload, restman_rs::POST);
```

> See how at this point, only `Company` was needed and no other part of the
> URL + one does not need to mention the semantics of the earlier URL such as it
> requiring a config parameter

## `ApiClient`

To get access to the request methods, implement the `ApiClient<C: Server>` and
`ApiClientBackend<C: ApiHttpClient>`.

> Note, traits split to allow user to support multiple `Server`s

```rust
struct MyApiClient<C: ApiHttpClient> {
    backend: C
}

// Configuring the backend is the library creator's job as all API backends are different
impl<C: ApiHttpClient> MyApiClient<C> {
    fn new(bearer_token: &str) {
        let backend = MyBackend::new();

        backend.set_header("authorization", bearer_token); // I provide the bearer token header name (default authentication mechanism for most API backends)
    }
}

impl<C: ApiHttpClient> ApiClientBackend<C> for WorkjamUser<C> {
    // return the *configured* backend
    fn backend(&self) -> &C {
        &self.backend
    }
}

impl<C: ApiHttpClient> ApiClientServer<MyServer> for WorkjamUser<C> {}
// impl<C: ApiHttpClient> ApiClientServer<MyServer1> for MyApiClient<C> {}
// ...
// impl<C: ApiHttpClient> ApiClientServer<MyServerN> for MyApiClient<C> {}
```

It is the **user's** responsibility to correctly configure the backend (i.e., set all the authorisation headers etc). This library is designed so that each `backend` instance represents a session - I have intentionally not allowed one backend to be used with different tokens.  

Since the backend has to be uniquely configured, that is why accessing the `ApiHttpClient`'s methods requires wrapping the backend struct - this way you can uniquely set which servers that configuration of the backend can handle. Otherwise, a backend whose cookies are configured to handle server `X` may accidently be used to handle queries for server `Y`, producing a chain of errors that becomes difficult to debug.  

## Making requests

Finally, we are ready to make requests!

```rust
const TOKEN: &str = "my_token";

let backend = restman_rs::UreqApiHttpClient::new(restman_rs::client::AGENT);

let my_client = MyApiClient {
    token: TOKEN.to_string();
    backend: 
};

let config = MyConfig { company_id: "my-company" }
let para = EmployeePara {id: "my-id", employment: None };
let payload = restman_rs::ApiPayload::new(&EmployeePayload {greeting: "hey worker!".to_string()});
let req = restman_rs::ApiRequest::<Employee>::new_with_para(&config, para);

let res: EmployeeRes = client.send_payload(&req, &payload).unwrap();
// let res: EmployeeRes = client.request(&req).unwrap(); // for when no payload required - will result in an error, if not the case
```

> You can plug in your own backend, as long as it implements the
> restman_rs::client::ApiHttpClient trait

## Custom HTTP backend

The bare minimum is to implement `restman_rs::ApiHttpClient`, then depending on
which request types implement: `restman_rs::{GET, PATCH, PUT, POST}` for synchronous functionality or the asynchronous analogues `restman_rs::{AGET, APATCH, APUT, APOST}`.  

*Note*; when implementing remember that a user can use one backend for different domains.  

### Default backends

Crate comes with two backends; `ureq` and `reqwest` which can be enabled using their respective flags (e.g., `cargo add ... -F ureq`).  

Since `reqwest` is significantly heavier than `ureq`, I have reserved it solely for asynchronous applications - it does not implement any of the synchronous methods. Use `ureq` for synchronous applications as it is significantly lighter.  

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

This quickly becomes untenable when you have a large number of endpoint, and
many parts to the URL. If the API changes at any point, you have to traverse
through every single endpoint and change the path everytime.

For example, suppose the `company` part of the example URI now resides at
`/v1/location/{location id}/city/{city id}/company/{company id}`, you have to
change the arguments of each method (there may tens or hundreds!), and change
the `format!` method!

This is illogical though, the `employee` endpoint really only needs to know the
detail that it's preceeding part is `company` - everything else should be
inherited implicitly. This is what my crate solves, among other things that
becomes obvious with use.

# Examples

For indepth, real-world example, refer to
[workjam-rs](https://github.com/SmartBoy84/workjam-rs) and
[bravia-rs](https://github.com/SmartBoy84/bravia-rs).
