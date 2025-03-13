use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit1, multispace0},
    sequence::delimited,
};

#[derive(Debug)]
pub enum Command {
    Buy((String, u64)),
    View,
}

pub fn parse_command(input: &str) -> IResult<&str, Command> {
    let (input, command_name) = alt((tag("view"), tag("buy"))).parse(input)?;
    let parsed = match command_name {
        "view" => (input, Command::View),
        "buy" => {
            let (input, buy) = parse_buy(input)?;
            (input, Command::Buy(buy))
        }
        _ => unreachable!(),
    };
    Ok(parsed)
}

fn parse_buy(input: &str) -> IResult<&str, (String, u64)> {
    let (input, item) = delimited(multispace0, alphanumeric1, multispace0).parse(input)?;
    let (input, cost) = delimited(multispace0, digit1, multispace0).parse(input)?;

    Ok((input, (item.into(), cost.parse().unwrap())))
}
