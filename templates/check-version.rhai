//
// Check min version of 'sails-cli'
//
let sails_cli_min_version_str = "0.4.0";
let sails_cli_min_version = parse_sem_ver(sails_cli_min_version_str);

let sails_cli_version_str = if variable::is_set("sails-cli-version") {
    variable::get("sails-cli-version")
} else {
    "0"
};
let sails_cli_version = parse_sem_ver(sails_cli_version_str);

if sails_cli_version[0] < sails_cli_min_version[0]
    || sails_cli_version[1] < sails_cli_min_version[1]
    || sails_cli_version[2] < sails_cli_min_version[2] {
    abort("Sails CLI version '" + sails_cli_version_str + "' is not supported. Please upgrade to version '" + sails_cli_min_version_str + "' or higher");
}

//
// Check min version of 'sails-rs'
//
let sails_min_version_str = "0.6.3";
let sails_min_version = parse_sem_ver(sails_min_version_str);

let sails_version_str = if variable::is_set("sails-version") {
    variable::get("sails-version")
} else {
    ""
};
if sails_version_str == "" {
    sails_version_str = variable::get("sails-rs-version");
}
print(`SAILS VERSION-1: '${sails_version_str}'`);
let sails_version = parse_sem_ver(sails_version_str);

print(`Major ${sails_version[0]}, Minor ${sails_version[1]}, Patch ${sails_version[2]}`);
print(`Major ${sails_min_version[0]}, Minor ${sails_min_version[1]}, Patch ${sails_min_version[2]}`);

if sails_version[0] < sails_min_version[0]
    || sails_version[1] < sails_min_version[1]
    || sails_version[2] < sails_min_version[2] {
    abort("Sails version '" + sails_version_str + "' is not supported. Please specify version equal or higher than '" + sails_min_version_str + "'");
}

print("SAILS VERSION-2: '" + sails_version_str + "'");

fn parse_sem_ver(version) {
    let dot_idx = 0;
    let dot_idx_next = index_of(version, ".", dot_idx);
    let major = if dot_idx_next == -1 {
        version.sub_string(dot_idx)
    } else {
        version.sub_string(dot_idx..dot_idx_next)
    };
    major = parse_int(major);

    dot_idx = dot_idx_next + 1;
    dot_idx_next = index_of(version, ".", dot_idx);
    let minor = if dot_idx == 0 {
        "0"
    } else if dot_idx_next == -1 {
        version.sub_string(dot_idx)
    } else {
        version.sub_string(dot_idx..dot_idx_next)
    };
    minor = parse_int(minor);

    dot_idx = dot_idx_next + 1;
    dot_idx_next = index_of(version, ".", dot_idx);
    let patch = if dot_idx == 0 {
        "0"
    } else if dot_idx_next == -1 {
        version.sub_string(dot_idx)
    } else {
        abort("Version '" + version + "' does not conform to semantic versioning");
    };
    patch = parse_int(patch);

    [major, minor, patch]
}
