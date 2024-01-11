use conductor_common::http::Bytes;
use vrl::{
  compiler::{
    state, value::kind, Context, Expression, Function, FunctionExpression, Parameter, Resolved,
    TypeDef,
  },
  stdlib::{
    Abs, Append, Array, Assert, AssertEq, Boolean, Ceil, Chunks, Compact, Contains, ContainsAll,
    DecodeBase64, DecodePercent, Del, Downcase, EncodeBase64, EncodeJson, EncodePercent, EndsWith,
    Exists, Filter, Find, Flatten, Float, Floor, ForEach, FormatInt, FormatNumber, Get, GetEnvVar,
    GetHostname, Hmac, Includes, Integer, IsArray, IsBoolean, IsEmpty, IsFloat, IsInteger, IsJson,
    IsNull, IsNullish, IsObject, IsRegex, IsString, IsTimestamp, Join, Keys, Length, Log, MapKeys,
    MapValues, Match, MatchAny, MatchArray, Md5, Merge, Mod, Now, Object, ParseFloat, ParseInt,
    ParseJson, ParseKeyValue, ParseQueryString, ParseRegex, ParseRegexAll, ParseUrl,
    ParseUserAgent, Push, RandomBool, RandomBytes, RandomFloat, RandomInt, Redact, Remove, Replace,
    Round, Set, Slice, Split, StartsWith, String as VrlString, StripAnsiEscapeCodes,
    StripWhitespace, Strlen, Timestamp, ToBool, ToFloat, ToInt, ToRegex, ToString, Truncate,
    Unique, Unnest, Upcase, UuidV4, Values,
  },
  value,
  value::Value,
};

pub fn vrl_fns() -> Vec<Box<dyn Function>> {
  vec![
    // Custom Functions
    Box::new(ShortCircuitVrlFunction),
    // Array
    Box::new(Append),
    Box::new(Chunks),
    Box::new(Push),
    // Codec
    Box::new(DecodeBase64),
    Box::new(EncodeBase64),
    Box::new(EncodeJson),
    Box::new(DecodePercent),
    Box::new(EncodePercent),
    // Coerce
    Box::new(ToBool),
    Box::new(ToFloat),
    Box::new(ToInt),
    Box::new(ToRegex),
    Box::new(ToString),
    // Debug
    Box::new(Log),
    Box::new(Assert),
    Box::new(AssertEq),
    // Enumerate
    Box::new(Compact),
    Box::new(Filter),
    Box::new(Flatten),
    Box::new(ForEach),
    Box::new(Includes),
    Box::new(Keys),
    Box::new(Length),
    Box::new(MapKeys),
    Box::new(MapValues),
    Box::new(MatchArray),
    Box::new(Strlen),
    Box::new(Unique),
    Box::new(Values),
    // Path
    Box::new(Del),
    Box::new(Exists),
    Box::new(Get),
    Box::new(Remove),
    Box::new(Set),
    // Crypto
    Box::new(Hmac),
    Box::new(Md5),
    // Number
    Box::new(Abs),
    Box::new(Ceil),
    Box::new(Floor),
    Box::new(FormatInt),
    Box::new(FormatNumber),
    Box::new(Mod),
    Box::new(Round),
    // Object
    Box::new(Merge),
    Box::new(Unnest),
    // Parse
    Box::new(ParseInt),
    Box::new(ParseFloat),
    Box::new(ParseJson),
    Box::new(ParseRegex),
    Box::new(ParseRegexAll),
    Box::new(ParseUrl),
    Box::new(ParseUserAgent),
    Box::new(ParseKeyValue),
    Box::new(ParseQueryString),
    // Random
    Box::new(RandomBool),
    Box::new(RandomBytes),
    Box::new(RandomFloat),
    Box::new(RandomInt),
    Box::new(UuidV4),
    // String
    Box::new(Contains),
    Box::new(ContainsAll),
    Box::new(Downcase),
    Box::new(EndsWith),
    Box::new(Find),
    Box::new(Join),
    Box::new(Match),
    Box::new(MatchAny),
    Box::new(Redact),
    Box::new(Replace),
    Box::new(Slice),
    Box::new(Split),
    Box::new(StartsWith),
    Box::new(StripAnsiEscapeCodes),
    Box::new(StripWhitespace),
    Box::new(Truncate),
    Box::new(Upcase),
    // System
    Box::new(GetEnvVar),
    Box::new(GetHostname),
    // Timestamp
    Box::new(Now),
    // Types
    Box::new(Array),
    Box::new(Boolean),
    Box::new(Float),
    Box::new(Integer),
    Box::new(IsArray),
    Box::new(IsBoolean),
    Box::new(IsString),
    Box::new(IsFloat),
    Box::new(IsEmpty),
    Box::new(IsInteger),
    Box::new(IsJson),
    Box::new(IsNull),
    Box::new(IsNullish),
    Box::new(IsObject),
    Box::new(IsRegex),
    Box::new(IsTimestamp),
    Box::new(Object),
    Box::new(VrlString),
    Box::new(Timestamp),
  ]
}

#[derive(Debug)]
struct ShortCircuitVrlFunction;

impl Function for ShortCircuitVrlFunction {
  fn identifier(&self) -> &'static str {
    "short_circuit"
  }

  fn parameters(&self) -> &'static [Parameter] {
    &[
      Parameter {
        keyword: "http_code",
        kind: kind::INTEGER,
        required: true,
      },
      Parameter {
        keyword: "message",
        kind: kind::BYTES,
        required: false,
      },
    ]
  }

  fn examples(&self) -> &'static [vrl::prelude::Example] {
    &[]
  }

  fn compile(
    &self,
    _state: &vrl::prelude::TypeState,
    _ctx: &mut vrl::prelude::FunctionCompileContext,
    arguments: vrl::prelude::ArgumentList,
  ) -> vrl::prelude::Compiled {
    let http_code = arguments.required("http_code");
    let message = arguments.optional("message");

    Ok(ShortCircuitFn { http_code, message }.as_expr())
  }
}

#[derive(Clone, Debug)]
pub struct ShortCircuitFn {
  http_code: Box<dyn Expression>,
  message: Option<Box<dyn Expression>>,
}

impl ShortCircuitFn {
  pub fn check_short_circuit(value: &Value) -> Option<(i64, Bytes)> {
    if let Some(Value::Boolean(short_circuit)) = value.get("short_circuit") {
      if *short_circuit {
        let http_code = value
          .get("http_code")
          .and_then(|v| v.as_integer())
          .unwrap_or(500);

        let default_error_message =
          Bytes::from_static("invalid message provided for short_circuit!".as_bytes());
        let message = value
          .get("message")
          .and_then(|v| v.as_bytes())
          .unwrap_or(&default_error_message);

        return Some((http_code, message.clone()));
      }
    }

    None
  }
}

impl FunctionExpression for ShortCircuitFn {
  fn resolve(&self, ctx: &mut Context) -> Resolved {
    let http_code = self.http_code.resolve(ctx)?;
    let message = self.message.as_ref().map(|s| s.resolve(ctx)).transpose()?;

    Ok(value!({
        short_circuit: true,
        http_code: http_code,
        message: message,
    }))
  }

  fn type_def(&self, _: &state::TypeState) -> TypeDef {
    TypeDef::bytes().fallible()
  }
}
