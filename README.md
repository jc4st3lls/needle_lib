# needle_lib

[![Crates.io](https://img.shields.io/crates/v/needle_lib.svg)](https://crates.io/crates/needle_lib)
[![Documentation](https://docs.rs/needle_lib/badge.svg)](https://docs.rs/needle_lib)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

A high-performance Rust inference library for the **Cactus Needle2** binary model.

`needle_lib` provides a safe, idiomatic, and simple API to run local model inference with built-in support for tool calling, reasoning/thinking steps extraction, confidence metrics, and grounding validation.

---

## Features

- **Local & Offline Inference**: Run the model fully offline using high-performance precompiled native engines.
- **Dynamic Tool (Function) Calling**: Provide any standard JSON schema of available tools, and the engine will decide when and how to call them.
- **Reasoning Steps & Confidence**: Access the model's inner reasoning chain and confidence scores directly in the response.
- **Grounding Validation**: Built-in support for validation metrics, including negation detection and ungrounded statement tracking.
- **Performance Profiling**: Retrieve precise prefill TPS (Tokens Per Second), decode TPS, and peak RAM consumption (MB).
- **Custom Weights Support**: Re-load fine-tuned weights (such as custom `.cact` LoRA files) on the fly.

---

## Installation

Add `needle_lib` to your `Cargo.toml`:

```toml
[dependencies]
needle_lib = "0.1.1"
serde_json = "1.0"
anyhow = "1.0"
```

### Pre-requisites

The native library linkage is fully managed by our `build.rs` script. To keep the crate lightweight and compliant with crates.io size constraints, the build script automatically downloads the correct precompiled native library (`libneedle.dylib` on macOS, `libneedle.so` on Linux, or `libneedle.dll` on Windows) for your target CPU architecture.

These shared libraries and models are downloaded during compilation from our official Hugging Face repository: [jc4st3lls/needle_lib](https://huggingface.co/jc4st3lls/needle_lib). They are stored under the user's home directory inside the `~/.needle/` directory. They are linked automatically using `rpath`, so you do not need to configure any complex environment variables or manually download any assets.

---

## Quick Start

The following example demonstrates how to initialize the engine with a tool definition and request a complete response.

```rust
use needle_lib::{Needle, ResponseType};
use serde_json::json;

fn main() -> anyhow::Result<()> {
    // 1. Define your tools in JSON Schema format
    let tools = json!([{
        "name": "get_weather",
        "description": "Get the current weather for a city.",
        "parameters": {
            "type": "object",
            "properties": {
                "city": { "type": "string", "description": "The city name, e.g. London" }
            },
            "required": ["city"]
        }
    }]);

    // 2. Initialize the Needle engine (with an optional system prompt)
    let system_prompt = "You are a helpful and precise assistant.";
    let needle = Needle::init(system_prompt, &tools.to_string())?;

    // 3. Complete a prompt that requires tool execution
    let response = needle.complete("What's the weather like in Lagos right now?", 256)?;

    // 4. Handle the structured response
    match response.kind {
        ResponseType::Call => {
            println!("The model decided to call a tool!");
            for call in &response.function_calls {
                println!("  Tool name: {}", call.name);
                if let Some(city) = call.arguments.get("city").and_then(|v| v.as_str()) {
                    println!("  Argument 'city': {}", city);
                }
            }
        }
        ResponseType::Respond => {
            println!("The model responded with direct text.");
        }
    }

    println!("Confidence score: {}", response.confidence);
    if let Some(reasoning) = &response.reasoning {
        println!("Thinking process: {}", reasoning);
    }

    // Print performance metrics
    println!("Prefill TPS: {}", response.prefill_tps);
    println!("Decode TPS: {}", response.decode_tps);
    println!("Peak RAM usage: {} MB", response.peak_ram_mb);

    Ok(())
}
```

---

## Advanced: Loading Custom Weights (LoRA)

By default, the Cactus Needle2 engine runs using its built-in bundled weights. If you perform fine-tuning (producing a custom `.cact` weights blob), you can load it dynamically before initializing the engine:

```rust
use needle_lib::Needle;

fn main() -> anyhow::Result<()> {
    // Read your custom fine-tuned weights blob
    let custom_weights = std::fs::read("path/to/my_needle.cact")?;

    // Load weights into the native engine
    Needle::load_weights(&custom_weights)?;

    // Now, initialization will use your custom weights!
    let needle = Needle::init("System prompt", "[]")?;
    
    Ok(())
}
```

---

## Run Examples

We include pre-packaged examples in the `examples/` directory. You can run them directly:

### Lights Agent Example

Controls room lights and brightness dynamically based on natural language commands:

```bash
cargo run --example lights_agent
```

### Weather Agent Example

Triggers weather query tool calls:

```bash
cargo run --example weather_agent
```

---

## API Reference

### `Needle`

- `Needle::init(system: &str, tools_json: &str) -> anyhow::Result<Needle>`: Initializes the context and tools schema.
- `Needle::complete(&self, text: &str, max_new_tokens: i32) -> anyhow::Result<NeedleResponse>`: Runs the local LLM generation.
- `Needle::reset(&self)`: Clears/resets the engine context.
- `Needle::load_weights(blob: &[u8]) -> anyhow::Result<()>`: Overrides bundled weights with custom `.cact` model weights.

### `NeedleResponse`

The structured output from `complete`:

- `kind`: `ResponseType` (either `Call` or `Respond`).
- `success`: `bool` indicating whether the execution succeeded.
- `error` / `error_code` / `reason`: Optional debug and failure details.
- `function_calls`: `Vec<FunctionCall>` containing requested tool executions.
- `reasoning`: `Option<String>` containing the model's thinking steps.
- `confidence`: `f64` confidence score.
- `validation`: Grounding checks such as negation and ungrounded statements.
- `prefill_tps` / `decode_tps` / `peak_ram_mb`: Key performance and resource metrics.

---

## Acknowledgements

Special thanks to **Gemma4** for assisting in the development, translation, and structuring of this library.

---

## License

This project is licensed under the Apache License, Version 2.0. See the [LICENSE](LICENSE) file for more details.
