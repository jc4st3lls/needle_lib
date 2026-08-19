// examples/weather_agent.rs
use needle_lib::{Needle, ResponseType}; // The crate name as defined in [package] name

fn main() -> anyhow::Result<()> {
    let tools = serde_json::json!([{
        "name": "get_weather",
        "description": "Get the current weather for a city.",
        "parameters": {
            "type": "object",
            "properties": { "city": { "type": "string" } },
            "required": ["city"]
        }
    }]);

    let needle = Needle::init("", &tools.to_string())?;
    let response = needle.complete("what's it like in Lagos right now?", 256)?;
    if response.kind == ResponseType::Call {
        for call in &response.function_calls {
            println!("Tool: {}", call.name);
            if let Some(city) = call.arguments.get("city").and_then(|v| v.as_str()) {
                println!("  city = {city}");
            }
        }
    }

    println!("confidence: {}", response.confidence);
    println!("reasoning: {}", response.reasoning.as_deref().unwrap_or("none"));
    //println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}
