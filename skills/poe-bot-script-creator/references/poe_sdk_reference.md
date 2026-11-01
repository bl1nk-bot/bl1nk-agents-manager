This reference covers the core components of the Poe Python library used for building bots.

## Core Classes

### `poe.Message`
Represents a message in a chat.
- `text`: The content of the message.
- `sender`: Role of the sender ("user", "bot", "system", "tool").
- `attachments`: List of `poe.Attachment` objects.
- `write(text)`: Append text to an in-progress message.
- `overwrite(text)`: Replace message text.

### `poe.Attachment`
Represents a file or image attached to a message.
- `name`: Filename.
- `contents`: Bytes content.
- `url`: Source URL.
- `is_inline`: Boolean for inline rendering.

## Server Bot Integration (`fastapi_poe`)
For low-level server implementation, use `fastapi_poe`.
- `fp.PoeBot`: Base class for creating bots.
- `fp.run(bot)`: Start the bot server.
