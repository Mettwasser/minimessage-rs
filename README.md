# minimessage

A (partial) implementation of [minimessage](https://docs.papermc.io/adventure/minimessage/) for
[Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) using a macro that checks the syntax at compile
time! The macro is fashioned just like `format!`.

### Example

```rs
minimessage!("<blue>Hello {}!", user.name);
```

## Small Demo

```rs
struct TestCommandHandler;

impl CommandHandler for TestCommandHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let number = random::<f32>();

        sender.send_message(minimessage!(
            r#"
            <blue>Hello there, <red><bold>{}</bold></red>!</blue> <yellow>Here's your shiny bold number: <bold>{number:.2}</bold></yellow>
            <click:open_url:"https://pumpkinmc.org/">Visit <red>pumpkin</red>!</click>
            "#,
            sender.get_name()
        ));

        Ok(0)
    }
}
```

![](readme_data/image.png)

# File embedding

File embedding is another feature. This will insert the file contents at compile time and you get the same,
compile time checked benefits as if the file contents were in-place!

`minimessage_demo.xml`

```xml
<blue>Hello there, <red><bold>{}</bold></red>!</blue> <yellow>Here's your shiny bold number: <bold>{number:.2}</bold></yellow>
<click:open_url:"https://pumpkinmc.org/">Visit <red>pumpkin</red>!</click>
<hover:show_text:"<blue>Hello world">Hover me!
```

Command Handler:

```rs
struct TestCommandHandler;

impl CommandHandler for TestCommandHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let number = 69;
        sender.send_message(minimessage!(file:"minimessage_demo.xml", sender.get_name()));

        Ok(0)
    }
}
```

![](readme_data/image2.png)

# Dynamic Rendering

Dynamic rendering requires a new GIT dependency.
Why? The `pumpkin-plugin-api` is a git dependency which cannot be pushed to crates.io.
This also allows this project to stay MIT licensed.

To use the dynamic renderer "properly", add this dependency:

```toml
[dependencies]
minimessage-rt-compat = { git = "https://github.com/Mettwasser/minimessage-rs", package = "minimessage-rt-compat" }
```

Please note however that this dependency is licensed as GPLv3 because of its dependency to `pumpkin-plugin-api`

If that's done, you can use the renderer as follows:

```rs
struct TestCommandHandler;

impl CommandHandler for TestCommandHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let text = "<blue>Hello! My name is <white><bold><italic>{my_name}!";
        let args = minimessage_rs::ArgumentCollection::new().named("my_name", sender.get_name());

        let component = minimessage_rs::deserialize_with_args(text, args).unwrap();

        sender.send_message(minimessage_rt_compat::convert(&component));

        Ok(0)
    }
}
```

![](readme_data/image3.png)

# Crate Structure

| Crate                   | Description                                                                                 |
| ----------------------- | ------------------------------------------------------------------------------------------- |
| `minimessage-rs`        | Re-exports for convenience                                                                  |
| `minimessage-impl`      | Core minimessage parser                                                                     |
| `minimessage-macro`     | `minimessage!` proc macro                                                                   |
| `minimessage-runtime`   | Runtime deserialization into a generic component tree                                       |
| `minimessage-rt-compat` | Converts runtime components to Pumpkin's `TextComponent` (not included in `minimessage-rs`) |

All crates are MIT licensed. `minimessage-rt-compat` is GPLv3 due to its dependency on `pumpkin-plugin-api`.
