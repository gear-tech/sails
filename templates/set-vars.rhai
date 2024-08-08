let project_name = variable::get("project-name");

let client_project_name = if project_name.to_snake_case() == project_name {
    project_name + "_client"
} else {
    project_name + "-client"
};

variable::set("client-project-name", client_project_name);

variable::set("program-name", project_name.to_pascal_case());

variable::set("program-name-snake", project_name.to_snake_case());

// Set versions of used crates
variable::set("mockall-version", "0.12");
variable::set("sails-rs-version", "0.2.1");
