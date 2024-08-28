## The **{{ project-name }}** program

The program workspace includes the following packages:
- `{{ project-name }}` is the package allowing to build WASM binary for the program and IDL file for it. {% if with-client and with-gtest %} 
  The package also includes integration tests for the program in the `tests` sub-folder{% endif %}
- `{{ app-project-name }}` is the package containing business logic for the program represented by the `{{ service-struct-name }}` structure. {% if with-client %} 
- `{{ client-project-name }}` is the package containing the client for the program allowing to interact with it from another program, tests, or
  off-chain client.
{% endif %}
