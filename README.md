# minimessage

A (partial) implementation of minimessage for Pumpkin using a macro that checks the syntax at compile time!
The macro is fashioned just like `format!`.
<br>

> **NOTE:**
>
> This project is GPLv3 licensed if you use the `runtime` feature flag, as this bundles the `pumpkin-plugin-api` which is GPLv3.
> If you do not use this feature, it is licensed under MIT.

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

## File embedding

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

## Dynamic Rendering (`runtime` feature flag)

You can also render components at runtime:

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
        let args = minimessage::ArgumentCollection::new().named("my_name", sender.get_name());

        let component = minimessage::deserialize_with_args(text, args)
            .map_err(|err| CommandError::CommandFailed(TextComponent::text(&err.to_string())))?;

        sender.send_message(component);

        Ok(0)
    }
}
```

![](readme_data/image3.png)
