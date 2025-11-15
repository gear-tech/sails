## The **{{ project-name }}** program

The program workspace includes the following packages:
- `{{ project-name }}` is the package allowing to build WASM binary for the program and IDL file for it. {% if with-client and with-gtest %} 
  The package also includes integration tests for the program in the `tests` sub-folder{% endif %}
- `{{ app-project-name }}` is the package containing business logic for the program represented by the `{{ service-struct-name }}` structure. {% if with-client %} 
- `{{ client-project-name }}` is the package containing the client for the program allowing to interact with it from another program, tests, or
  off-chain client.
{% endif %}

The `{{ app-project-name }}` package now keeps its canonicalization targets in `sails_services.in`. Both the build script and the
`sails_meta_dump` helper include that manifest, so adding or removing services only requires editing that single file. The file uses the
`sails_services! { services: [ ... ] }` syntax and may declare witness aliases before the `services` block if a generic service needs to be
instantiated with a concrete client type.
