use bevy::prelude::*;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{value, verify};
use nom::error::ErrorKind;
use nom::{AsChar, IResult, InputLength, InputTake, InputTakeAtPosition, Parser};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Converts a color field number to a pair of hex digits if there is no precision loss.
fn to_hex_int(num: f64) -> Option<u8>
{
    let converted = (num * 255.0f64) as u8;
    let left = num;
    let right = (converted as f64) / (255.0f64);
    let diff = (left - right) as f32;

    if diff.is_subnormal() || diff == 0.0 {
        Some(converted)
    } else {
        None
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn write_num_as_hex(num: u8, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
{
    write!(writer, "{:02X}", num)
}

//-------------------------------------------------------------------------------------------------------------------

/// Copied from nom::hex_u32 on nom's main branch.
/// TODO: remove when nom publishes a new version
fn parse_hex_u32(input: Span) -> IResult<Span, u32>
{
    let e: ErrorKind = ErrorKind::IsA;
    let (i, o) = input.split_at_position1_complete(
        |c| {
            let c = c.as_char();
            !"0123456789abcdefABCDEF".contains(c)
        },
        e,
    )?;

    // Do not parse more than 8 characters for a u32
    let (remaining, parsed) = if o.input_len() <= 8 {
        (i, o)
    } else {
        input.take_split(8)
    };

    let res = parsed
        .as_bytes()
        .iter()
        .rev()
        .enumerate()
        .map(|(k, &v)| {
            let digit = v as char;
            digit.to_digit(16).unwrap_or(0) << (k * 4)
        })
        .sum();

    Ok((remaining, res))
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafHexColor
{
    pub fill: CafFill,
    pub color: Srgba,
}

impl CafHexColor
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        writer.write_bytes("#".as_bytes()).unwrap();
        if self.color.alpha != 1.0 {
            write_num_as_hex((self.color.alpha * 255.) as u8, writer)?;
        }
        write_num_as_hex((self.color.red * 255.) as u8, writer)?;
        write_num_as_hex((self.color.green * 255.) as u8, writer)?;
        write_num_as_hex((self.color.blue * 255.) as u8, writer)?;
        Ok(())
    }

    pub fn try_parse(fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let Ok((remaining, _)) = char::<_, ()>('#').parse(content) else { return Ok((None, fill, content)) };
        let start_len = remaining.input_len();
        let (remaining, digits) = parse_hex_u32(remaining)?;
        let end_len = remaining.input_len();

        let len = end_len - start_len;
        if len != 8 && len != 6 {
            tracing::warn!("failed parsing hex color at {}; hex length is {} but expected 6 or 8",
                get_location(content), len);
            return Err(span_verify_error(content));
        }

        let mut color = Srgba::default();
        if len == 8 {
            color.alpha = (((digits << 6) as u8) as f32) / 255.;
        }
        color.red = (((digits << 4) as u8) as f32) / 255.;
        color.green = (((digits << 2) as u8) as f32) / 255.;
        color.blue = ((digits as u8) as f32) / 255.;

        let (next_fill, remaining) = CafFill::parse(remaining);
        Ok((Some(Self { fill, color }), next_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
    }
}

impl TryFrom<Srgba> for CafHexColor
{
    type Error = ();

    /// Only succeeds if all fields can be converted to hex without precision loss.
    fn try_from(value: Srgba) -> Result<Self, ()>
    {
        if to_hex_int(value.red as f64).is_none()
            || to_hex_int(value.green as f64).is_none()
            || to_hex_int(value.blue as f64).is_none()
            || to_hex_int(value.alpha as f64).is_none()
        {
            return Err(());
        }
        Ok(Self { fill: CafFill::default(), color: value })
    }
}

/*
Parsing:
- proper hex format with optional alpha at the beginning
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafBuiltin
{
    Color(CafHexColor),
    Val
    {
        fill: CafFill,
        /// There is no number for `Val::Auto`.
        number: Option<CafNumberValue>,
        val: Val,
    },
}

impl CafBuiltin
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        match self {
            Self::Color(color) => {
                color.write_to_with_space(writer, space)?;
            }
            Self::Val { fill, number, val } => {
                fill.write_to_or_else(writer, space)?;
                if let Some(number) = number {
                    number.write_to(writer)?;
                }
                match val {
                    Val::Auto => {
                        writer.write_bytes("auto".as_bytes())?;
                    }
                    Val::Percent(..) => {
                        writer.write_bytes("%".as_bytes())?;
                    }
                    Val::Px(..) => {
                        writer.write_bytes("px".as_bytes())?;
                    }
                    Val::Vw(..) => {
                        writer.write_bytes("vw".as_bytes())?;
                    }
                    Val::Vh(..) => {
                        writer.write_bytes("vh".as_bytes())?;
                    }
                    Val::VMin(..) => {
                        writer.write_bytes("vmin".as_bytes())?;
                    }
                    Val::VMax(..) => {
                        writer.write_bytes("vmax".as_bytes())?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn try_parse(fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        // Hex color
        let fill = match CafHexColor::try_parse(fill, content)? {
            (Some(color), next_fill, remaining) => return Ok((Some(Self::Color(color)), next_fill, remaining)),
            (None, fill, _) => fill,
        };

        // Val::Auto
        if let Ok((remaining, val)) =
            value(Val::Auto, verify(snake_identifier, |i| *i.fragment() == "auto")).parse(content)
        {
            let (next_fill, remaining) = CafFill::parse(remaining);
            return Ok((Some(Self::Val { fill, number: None, val }), next_fill, remaining));
        }

        // Val::X(f32)
        let Ok((number, remaining)) = CafNumberValue::parse(content) else { return Ok((None, fill, content)) };
        let Some(num) = number.as_f32() else { return Ok((None, fill, content)) };
        let Ok((remaining, val)) = alt((
            value(Val::Percent(num), char::<_, ()>('%')),
            value(Val::Px(num), tag("px")),
            value(Val::Vw(num), tag("vw")),
            value(Val::Vh(num), tag("vh")),
            value(Val::VMin(num), tag("vmin")),
            value(Val::VMax(num), tag("vmax")),
        ))
        .parse(remaining) else {
            return Ok((None, fill, content));
        };

        let (next_fill, remaining) = CafFill::parse(remaining);
        Ok((
            Some(Self::Val { fill, number: Some(number), val }),
            next_fill,
            remaining,
        ))
    }

    pub fn try_from_unit_variant(typename: &str, variant: &str) -> CafResult<Option<Self>>
    {
        if typename == "Val" && variant == "Auto" {
            return Ok(Some(Self::Val {
                fill: CafFill::default(),
                number: None,
                val: Val::Auto,
            }));
        }

        Ok(None)
    }

    /// The value should not contain any macros/constants.
    pub fn try_from_newtype_variant(typename: &str, variant: &str, value: &CafValue) -> CafResult<Option<Self>>
    {
        if typename == "Color" && variant == "Srgba" {
            let CafValue::Map(CafMap { entries, .. }) = value else { return Ok(None) };
            let mut color = Srgba::default();
            for entry in entries.iter() {
                let CafMapEntry::KeyValue(keyval) = entry else { return Err(CafError::MalformedBuiltin) };
                let CafMapKey::FieldName { fill: _, name } = &keyval.key else {
                    return Err(CafError::MalformedBuiltin);
                };
                let CafValue::Number(num) = &keyval.value else { return Ok(None) };
                let Some(float) = num.number.as_f64() else { return Ok(None) };
                let value = float as f32;

                if name == "red" {
                    color.red = value;
                } else if name == "green" {
                    color.green = value;
                } else if name == "blue" {
                    color.blue = value;
                } else if name == "alpha" {
                    color.alpha = value;
                } else {
                    return Ok(None);
                }
            }

            return Ok(CafHexColor::try_from(color).map(|c| Self::Color(c)).ok());
        }

        if typename == "Val" {
            let CafValue::Number(num) = value else { return Ok(None) };
            let Some(float) = num.number.as_f64() else { return Ok(None) };
            let extracted = float as f32;

            let val = match variant {
                "Px" => Val::Px(extracted),
                "Percent" => Val::Percent(extracted),
                "Vw" => Val::Vw(extracted),
                "Vh" => Val::Vh(extracted),
                "VMin" => Val::VMin(extracted),
                "VMax" => Val::VMax(extracted),
                _ => return Err(CafError::MalformedBuiltin),
            };

            return Ok(Some(Self::Val {
                fill: CafFill::default(),
                number: Some(num.number.clone()),
                val,
            }));
        }

        Ok(None)
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Color(color), Self::Color(other_color)) => {
                color.recover_fill(other_color);
            }
            (Self::Val { fill, .. }, Self::Val { fill: other_fill, .. }) => {
                fill.recover(&other_fill);
            }
            _ => (),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
