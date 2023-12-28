#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Command {
    Ping,
    Approve,

    /// Assign `users` as assignees. If `users` is `None`, the user who issued the command is assigned by default.
    Assign {
        users: Option<Vec<String>>,
    },
}

impl Command {}

pub(crate) fn parse_command(bot_name: &str, input: &str) -> Vec<Command> {
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

            if word == "r+" {
                commands.push(Command::Approve);
            }

            if word == "c" || word == "claim" {
                commands.push(Command::Assign {
                    users: (pieces.clone()).skip(i).next().map_or(None, |users| {
                        Some(users.split(",").map(|x| x.into()).collect())
                    }),
                })
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
