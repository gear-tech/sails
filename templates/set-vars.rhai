let project_name = variable::get("project-name");

let app_project_name = if is_kebab_case(project_name) {
    project_name + "-app"
} else {
    project_name + "_app"
};

let client_project_name = if is_kebab_case(project_name) {
    project_name + "-client"
} else {
    project_name + "_client"
};

variable::set("app-project-name", app_project_name);

variable::set("app_crate_name", app_project_name.to_snake_case());

variable::set("client-project-name", client_project_name);

variable::set("client_crate_name", client_project_name.to_snake_case());

variable::set("program-struct-name", project_name.to_pascal_case() + "Program");

variable::set("service-struct-name", project_name.to_pascal_case() + "Service");

variable::set("service-name", project_name.to_pascal_case());

variable::set("service-name-snake", project_name.to_snake_case());

variable::set("mocks-feature-name", "mocks");

// Set versions of used crates
variable::set("mockall-version", "0.12");
variable::set("sails-rs-version", "0.6.2");
variable::set("tokio-version", "1.41");

fn is_kebab_case(name) {
    name.index_of('-') != -1 || name.index_of('_') == -1
}
