use anyhow::Result;
use clap::{arg, Arg, Command};
use nixpacks::{
    create_docker_image, generate_build_plan,
    nixpacks::{
        builder::docker::DockerBuilderOptions, nix::pkg::Pkg, plan::generator::GeneratePlanOptions,
    },
};

fn main() -> Result<()> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let matches = Command::new("nixpacks")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .version(VERSION)
        .subcommand(
            Command::new("plan")
                .about("Generate a build plan for an app")
                .arg(arg!(<PATH> "App source")),
        )
        .subcommand(
            Command::new("build")
                .about("Create a docker image for an app")
                .arg(arg!(<PATH> "App source"))
                .arg(
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .help("Name for the built image")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("out")
                        .long("out")
                        .short('o')
                        .help("Save output directory instead of building it with Docker")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("tag")
                        .long("tag")
                        .short('t')
                        .help("Additional tags to add to the output image")
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("label")
                        .long("label")
                        .short('l')
                        .help("Additional labels to add to the output image")
                        .takes_value(true)
                        .multiple_values(true),
                )
                .arg(
                    Arg::new("buildkit")
                        .long("buildkit")
                        .help("Forces docker to use buildkit")
                        .takes_value(false),
                ),
        )
        .arg(
            Arg::new("plan")
                .long("plan")
                .help("Existing build plan file to use")
                .takes_value(true)
                .global(true),
        )
        .arg(
            Arg::new("install_cmd")
                .long("install-cmd")
                .short('i')
                .help("Specify the install command to use")
                .takes_value(true)
                .global(true),
        )
        .arg(
            Arg::new("build_cmd")
                .long("build-cmd")
                .short('b')
                .help("Specify the build command to use")
                .takes_value(true)
                .global(true),
        )
        .arg(
            Arg::new("start_cmd")
                .long("start-cmd")
                .short('s')
                .help("Specify the start command to use")
                .takes_value(true)
                .global(true),
        )
        .arg(
            Arg::new("pkgs")
                .long("pkgs")
                .short('p')
                .help("Provide additional nix packages to install in the environment")
                .takes_value(true)
                .multiple_values(true)
                .global(true),
        )
        .arg(
            Arg::new("apt")
                .long("apt")
                .help("Provide additional apt packages to install in the environment")
                .takes_value(true)
                .multiple_values(true)
                .global(true),
        )
        .arg(
            Arg::new("libs")
                .long("libs")
                .help("Provide additional nix libraries to install in the environment")
                .takes_value(true)
                .multiple_values(true)
                .global(true),
        )
        .arg(
            Arg::new("pin")
                .long("pin")
                .help("Pin the nixpkgs")
                .takes_value(false)
                .global(true),
        )
        .arg(
            Arg::new("env")
                .long("env")
                .help("Provide environment variables to your build")
                .takes_value(true)
                .multiple_values(true)
                .global(true),
        )
        .get_matches();

    let install_cmd = matches.value_of("install_cmd").map(|s| vec![s.to_string()]);
    let build_cmd = matches.value_of("build_cmd").map(|s| vec![s.to_string()]);
    let start_cmd = matches.value_of("start_cmd").map(|s| s.to_string());
    let pkgs = match matches.values_of("pkgs") {
        Some(values) => values.map(Pkg::new).collect::<Vec<_>>(),
        None => Vec::new(),
    };
    let libs = match matches.values_of("libs") {
        Some(values) => values.map(String::from).collect::<Vec<String>>(),
        None => Vec::new(),
    };
    let apt_pkgs = match matches.values_of("apt") {
        Some(values) => values.map(String::from).collect::<Vec<String>>(),
        None => Vec::new(),
    };
    let pin_pkgs = matches.is_present("pin");

    let envs: Vec<_> = match matches.values_of("env") {
        Some(envs) => envs.collect(),
        None => Vec::new(),
    };

    let plan_path = matches.value_of("plan").map(|n| n.to_string());

    let plan_options = &GeneratePlanOptions {
        custom_install_cmd: install_cmd,
        custom_start_cmd: start_cmd,
        custom_build_cmd: build_cmd,
        custom_pkgs: pkgs,
        custom_libs: libs,
        custom_apt_pkgs: apt_pkgs,
        pin_pkgs,
        plan_path,
    };

    match &matches.subcommand() {
        Some(("plan", matches)) => {
            let path = matches.value_of("PATH").expect("required");

            let plan = generate_build_plan(path, envs, plan_options)?;
            let json = serde_json::to_string_pretty(&plan)?;
            println!("{}", json);
        }
        Some(("build", matches)) => {
            let path = matches.value_of("PATH").expect("required");
            let name = matches.value_of("name").map(|n| n.to_string());
            let out_dir = matches.value_of("out").map(|n| n.to_string());

            let tags = matches
                .values_of("tag")
                .map(|values| values.map(|s| s.to_string()).collect::<Vec<_>>())
                .unwrap_or_default();

            let labels = matches
                .values_of("label")
                .map(|values| values.map(|s| s.to_string()).collect::<Vec<_>>())
                .unwrap_or_default();

            let force_buildkit = matches.is_present("buildkit");

            let build_options = &DockerBuilderOptions {
                name,
                tags,
                labels,
                out_dir,
                force_buildkit,
                quiet: false,
            };

            create_docker_image(path, envs, plan_options, build_options)?;
        }
        _ => eprintln!("Invalid command"),
    }

    Ok(())
}
