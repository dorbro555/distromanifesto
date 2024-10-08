use std::path::PathBuf;
// use std::env::current_dir;

// use clap::{Arg, Command, Parser};
use clap::{arg, command, value_parser, ArgAction, Command, Parser, Subcommand};
use inquire::{Text, Confirm, formatter::MultiOptionFormatter, MultiSelect, validator::{StringValidator, Validation}};

#[derive(Parser)]
#[command(version, about, long_about = None)]

struct Args {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
    /// Runs a wizard to create a distrobox manifest file
    Wizard {
        /// Sets output filepath
        #[arg(short, long, default_value_t = String::from("./"))]
        filepath: String,
    }
}

fn main() {

    let args = Args::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = args.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = args.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match args.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &args.command {
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
        Some(Commands::Wizard { filepath: _ }) => {
            println!("Starting the wizard!");
            run_wizard("./")
        }
        None => {}
    }

    // for _ in 0..args.count {
    //     println!("Hello {}!", args.name);
    // }
}

fn run_wizard(output: &str) {
    println!("Welcome to the Distro Manifesto Wizard!");

    // Example step: Choose a base image
    // let base_image = inquire("Enter the base image:");
    // let init_cmd = inquire("Enter the initialization command:");

    let container_name = text_prompt_for_value("What should the container be named?: ");
    let base_image = text_prompt_for_value("Enter the base image: ");
    let init_cmd = text_prompt_for_value("Enter an init command: ");
    let home_value = text_prompt_for_value("Which home directory should the container use: ");

    let enabled_flags = multiselect_prompt_for_value("Select which flags you would like to turn on: ")

    // Additional steps...

    // Generate the assemble.ini content
    let assemble_content = format!(
        "\n[{}]\nimage=\"{}\"\ninit=\"{}\"\nhome=\"{}\"\nflags=\"{:?}\"",
        container_name, base_image, init_cmd, home_value, enabled_flags
    );

    // Write to file
    // std::fs::write(output, assemble_content).expect("Unable to write file");
    println!("Assemble file created at: {}", output);
    println!("Created Manifest file contents: {}", assemble_content)
}

fn text_prompt_for_value(prompt_question: &str) -> String {
    // validation for the inquire packages text prompt
    let validator = |input: &str| if input.chars().count() > 140 {
        Ok(Validation::Invalid("You're only allowed 140 characters.".into()))
    } else {
        Ok(Validation::Valid)
    };

    let value = Text::new(prompt_question)
        .with_validator(validator)
        .prompt();

    // extract the value out of the Result
    let final_value = match value {
        Ok(value) => value,
        Err(err) => {
            println!("Error while publishing your status: {}", err);
            panic!("Encountered an error"); // early return on error
        },
    };
    final_value
}

fn confirm_prompt_for_value(prompt_question: &str) -> bool {
    let value = Confirm::new(prompt_question)
        .with_help_message("the help message")
        .with_default(false)
        .prompt();

    // extract the value out of the Result
    let final_value = match value {
        Ok(value) => value,
        Err(err) => {
            println!("Error while publishing your status: {}", err);
            panic!("Encountered an error"); // early return on error
        },
    };
    final_value
}

fn multiselect_prompt_for_value(prompt_question: &str) -> String {
    let options = vec![
        "entry",
        "start_now",
        "init",
        "nvidia",
        "pull",
        "root",
        "unshare_ipc",
        "unshare_netns",
        "unshare_process",
        "unshare_devsys",
        "unshare_all",
    ];

    let formatter: MultiOptionFormatter<'_, &str> = &|a| format!("{} different flags", a.len());

    let flags = MultiSelect::new(prompt_question, options)
        .with_formatter(formatter)
        .prompt();

    let enabled_flags = match flags {
        Ok(values) => values,
        Err(err) => {
            panic!("The enabled flags could not be processed");
        },
    };

    let mut manifest_files_flags: String = "";

    for x in enabled_flags {
        manifest_files_flags.push_str("{}=true\n")
    }

    manifest_files_flags

}

// fn inquire(prompt: &str) -> String {
//     use std::io::{self, Write};
//     print!("{} ", prompt);
//     io::stdout().flush().unwrap();
//     let mut input = String::new();
//     io::stdin().read_line(&mut input).unwrap();
//     input.trim().to_string()
// }
