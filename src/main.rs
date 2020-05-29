use error::Result;
use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;
use structopt::StructOpt;

mod error {
    pub use anyhow::{Context, Result};
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum Error {}
}

#[derive(Debug, StructOpt)]
#[structopt(name = "roller", about = "A simple die roller")]
struct Opt {
    pub input: String,
    #[structopt(short, long)]
    pub crit: bool,
}

static DICE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?x)(?P<count>\d+)(?P<dtype>d\d+)\+?").expect("Failed to compile Dice Regex")
});

static CONSTANTS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\+(?P<const>\d+)(\+|$)").expect("Failed to compile Constants Regex"));

#[derive(Clone, Debug, PartialEq)]
enum Dice {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
}

impl From<&str> for Dice {
    fn from(s: &str) -> Self {
        match s {
            "d4" => Dice::D4,
            "d6" => Dice::D6,
            "d8" => Dice::D8,
            "d10" => Dice::D10,
            "d12" => Dice::D12,
            "d20" => Dice::D20,
            "d100" => Dice::D100,
            _ => unreachable!(),
        }
    }
}

impl From<Dice> for i32 {
    fn from(d: Dice) -> Self {
        let mut rng = rand::thread_rng();
        match d {
            Dice::D4 => rng.gen_range(1, 5),
            Dice::D6 => rng.gen_range(1, 7),
            Dice::D8 => rng.gen_range(1, 9),
            Dice::D10 => rng.gen_range(1, 11),
            Dice::D12 => rng.gen_range(1, 13),
            Dice::D20 => rng.gen_range(1, 21),
            Dice::D100 => rng.gen_range(1, 101),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Roll {
    pub dice: Vec<Dice>,
    pub constants: Vec<i32>,
}

impl Roll {
    pub fn new() -> Self {
        Roll {
            dice: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn cast(self, crit: i32) -> i32 {
        let dice_scores: Vec<i32> = self.dice.into_iter().map(Into::<i32>::into).collect();
        for n in &dice_scores {
            println!("{}", n);
        }
        let dice: i32 = dice_scores.iter().sum();
        let constant: i32 = self.constants.iter().sum();

        (dice * crit) + constant
    }
}

fn main() {
    let opt = Opt::from_args();
    let crit = match opt.crit {
        true => {
            println!("Critical Hit!");
            2
        }
        false => 1,
    };

    match run(opt.input, crit) {
        Ok(res) => println!("---\n{}", res),
        Err(err) => {
            println!("{:?}", err);
        }
    }
}

fn run(input: String, crit: i32) -> Result<i32> {
    let roll = parse(&input)?;

    Ok(roll.cast(crit))
}

fn parse(input: &str) -> Result<Roll> {
    let caps = DICE.captures_iter(input);
    let ccaps = CONSTANTS.captures_iter(input);
    let mut roll = Roll::new();

    for c in caps {
        for _ in 0..c["count"].parse::<i32>()? {
            roll.dice.push(Dice::from(&c["dtype"]))
        }
    }

    for c in ccaps {
        roll.constants.push(c["const"].parse::<i32>()?)
    }

    Ok(roll)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let input = "3d4+2d8+6";
        let res = parse(input).unwrap();
        assert_eq!(
            res,
            Roll {
                dice: vec![Dice::D4, Dice::D4, Dice::D4, Dice::D8, Dice::D8,],
                constants: vec![6]
            }
        )
    }

    #[test]
    fn test_const_alone() {
        let input = "+3";
        let caps = CONSTANTS.captures(input).unwrap();
        assert_eq!("3", &caps["const"]);
    }

    #[test]
    fn test_dice_regex() {
        let input = "3d4";
        let caps = DICE.captures(input).unwrap();
        assert_eq!("3", &caps["count"]);
        assert_eq!("d4", &caps["dtype"]);
    }

    #[test]
    fn test_many_dice_regex() {
        let input = "33d100";
        let caps = DICE.captures(input).unwrap();
        assert_eq!("33", &caps["count"]);
        assert_eq!("d100", &caps["dtype"]);
    }

    #[test]
    fn test_const_regex() {
        let input = "3d4+6";
        let dcaps = DICE.captures(input).unwrap();
        assert_eq!("3", &dcaps["count"]);
        assert_eq!("d4", &dcaps["dtype"]);

        let ccaps = CONSTANTS.captures(input).unwrap();
        assert_eq!("6", &ccaps["const"]);
    }

    #[test]
    fn test_multiple_dice_regex() {
        let input = "3d4+2d8";
        let mut caps = DICE.captures_iter(input);
        let c1 = &caps.next().unwrap();
        let c2 = &caps.next().unwrap();

        assert_eq!("3", &c1["count"]);
        assert_eq!("d4", &c1["dtype"]);

        assert_eq!("2", &c2["count"]);
        assert_eq!("d8", &c2["dtype"]);
    }

    #[test]
    fn test_multiple_const_regex() {
        let input = "3d4+6+2d8+9";
        let mut caps = DICE.captures_iter(input);
        let d1 = &caps.next().unwrap();
        let d2 = &caps.next().unwrap();

        assert_eq!("3", &d1["count"]);
        assert_eq!("d4", &d1["dtype"]);

        assert_eq!("2", &d2["count"]);
        assert_eq!("d8", &d2["dtype"]);

        let mut ccaps = CONSTANTS.captures_iter(input);
        let c1 = &ccaps.next().unwrap();
        let c2 = &ccaps.next().unwrap();

        assert_eq!("6", &c1["const"]);
        assert_eq!("9", &c2["const"]);
    }
}
