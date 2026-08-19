use needle_lib::{Needle, ResponseType}; // The crate name as defined in [package] name

fn main() -> anyhow::Result<()> {
    let tools = serde_json::json!([{
        "name": "set_lights",
        "description": "Turn a room's lights on or off and set brightness",
        "parameters": {
            "type": "object",
            "properties": {
                "room": { "type": "string" },
                "on": { "type": "boolean" },
                "brightness": { "type": "integer", "description": "0 to 100" }
            },
            "required": ["room", "on"]
        }
    }]);

    let needle = Needle::init("", &tools.to_string())?;
    let response = needle.complete("turn on the lights in the living room and set brightness to 30", 256)?;
    if response.kind == ResponseType::Call {
        for call in &response.function_calls {
            println!("Tool: {}", call.name);
            if let Some(room) = call.arguments.get("room").and_then(|v| v.as_str()) {
                println!("  room = {room}");
            }
            if let Some(on) = call.arguments.get("on").and_then(|v| v.as_bool()) {
                println!("  on = {on}");
            }
            if let Some(brightness) = call.arguments.get("brightness").and_then(|v| v.as_i64()) {
                println!("  brightness = {brightness}");
            }
        }
    }

    println!("confidence: {}", response.confidence);
    println!(
        "reasoning: {}",
        response.reasoning.as_deref().unwrap_or("none")
    );
    //println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}
