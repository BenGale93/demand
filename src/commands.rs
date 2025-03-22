use nom::{
    IResult, Parser,
    character::complete::{alphanumeric1, digit1, multispace0},
    sequence::delimited,
};

#[derive(Debug)]
pub struct BuyCommand {
    pub item: String,
    pub price: u64,
}

pub fn parse_buy(input: &str) -> IResult<&str, BuyCommand> {
    let (input, item) = delimited(multispace0, alphanumeric1, multispace0).parse(input)?;
    let (input, cost) = delimited(multispace0, digit1, multispace0).parse(input)?;

    Ok((
        input,
        BuyCommand {
            item: item.into(),
            price: cost.parse().unwrap(),
        },
    ))
}
