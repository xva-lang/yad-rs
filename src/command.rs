use regex::Regex;

use crate::{config::get_config, logging::info};

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Command {
    Ping,
    Approve,

    /// Assign `users` as assignees. If `users` is `None`, the user who issued the command is assigned by default.
    Assign {
        user: Option<String>,
    },

    RemoveAssignment,
}

impl Command {}

fn is_tag_and_not_pattern(input: &str, pattern: &String) -> Option<String> {
    if input.starts_with("@") && input != pattern {
        Some(input.replace("@", ""))
    } else {
        None
    }
}

pub(crate) fn parse_command(bot_name: &str, input: &str) -> Vec<Command> {
    let config = get_config(None).unwrap();
    let bot_name_pattern = format!("@{bot_name}");

    let maybe_commands = input
        .lines()
        .map(|line| {
            if line.contains(&bot_name_pattern) && !line.trim_start().starts_with(">") {
                if let Some(replace_point) = line.find(&bot_name_pattern) {
                    let replace_range = &line[0..replace_point];
                    line.replace(replace_range, "")
                } else {
                    "".into()
                }
            } else {
                "".into()
            }
        })
        .collect::<Vec<_>>();

    let mut commands = Vec::new();
    for maybe_command in maybe_commands.iter() {
        let pieces = maybe_command.split_whitespace();
        for (i, word) in pieces.clone().enumerate() {
            if word == &bot_name_pattern || word == &format!("{bot_name_pattern}:") {
                continue;
            }

            match word {
                "hello" => commands.push(Command::Ping),
                "r+" => commands.push(Command::Approve),
                "c" | "claim" => commands.push(Command::Assign { user: None }),
                "a" | "assign" => commands.push(Command::Assign {
                    user: (pieces.clone())
                        .skip(i + 1)
                        .next()
                        .map_or(None, |name| is_tag_and_not_pattern(name, &bot_name_pattern)),
                }),
                "ra" | "remove-assignment" => commands.push(Command::RemoveAssignment),
                _ => info(format!("Unknown command: {word}"), Some(&config)),
            }
        }
    }
    commands
}

#[cfg(test)]
mod tests {
    use crate::command::parse_command;

    #[test]
    fn command() {
        let comment = r"huge command
        that may have a command. @xleat r+";

        let commands = parse_command("xleat", comment);
        println!("{commands:#?}")
    }
}
